use rp_pico::hal::sio::{SpinlockValid, Spinlock};
use core::{
    marker::PhantomData,
    cell::UnsafeCell,
    sync::atomic::{compiler_fence, Ordering,},
};

/// Specifies which Spinlock a DoubleBufPort is currently holding
enum HeldLock<const A: usize, const B: usize> where
    Spinlock<A>: SpinlockValid,
    Spinlock<B>: SpinlockValid,
{
    A(Spinlock<A>),
    B(Spinlock<B>),
}

/// A single data buffer. Each DoubleBuf has two of these.
struct DataBuf<LTR, RTL>{
    /// A buffer for transferring data from the *Left* side to the *Right* side
    ltr: LTR,
    /// True if 'ltr' has not yet been read by the *Right* side
    ltr_fresh: bool,
    /// A buffer for transferring data from the *Right* side to the *Left* side
    rtl: RTL,
    /// True if 'rtl' has not yet been read by the *Left* side
    rtl_fresh: bool,
}

/// An asymmetrical, asynchronous, bidirectional double buffer.
/// Can be split into a *Left* and a *Right* to transfer data between the RP2040 cores.
/// Uses HW spinlocks for synchronization.
/// "A" and "B" parameters are integers from 0-31 specifying which spinlocks should be used.
pub struct DoubleBuf<LTR, RTL, const A: usize, const B: usize> where
    Spinlock<A>: SpinlockValid,
    Spinlock<B>: SpinlockValid,
    LTR: Send, LTR: Sync, RTL: Send, RTL: Sync
{
    buf_a: UnsafeCell<DataBuf<LTR, RTL>>,
    buf_b: UnsafeCell<DataBuf<LTR, RTL>>,
    is_split: bool,
    lock_a: PhantomData<Spinlock<A>>,
    lock_b: PhantomData<Spinlock<B>>,
}

impl <LTR, RTL, const A: usize, const B: usize>DoubleBuf<LTR, RTL, A, B> where
    Spinlock<A>: SpinlockValid,
    Spinlock<B>: SpinlockValid,
    LTR: Send, LTR: Sync, RTL: Send, RTL: Sync
{
    /// Create a new DoubleBuffer.
    /// This function is *unsafe* because the caller must assert that the provided spinlocks are not used for anything else.
    ///
    /// # Generic Parameters
    /// * 'LTR' - Whatever type of data will be transferred from the *Left* side of the buffer to the *Right* side. Usually a struct.
    /// * 'RTL' - Whatever type of data will be transferred from the *Right* side of the buffer to the *Left* side. Usually a struct.
    /// * 'A' and 'B' are integers from 0-31 specifying which spinlocks should be used. Must be explicitly specified.
    ///
    /// # Arguments
    ///
    /// * 'new_ltr' - Closure that constructs the left-to-right buffer.
    /// * 'new_rtl' - Closure that constructs the right-to-left buffer.
    // Note: We need to take closures as arguments so that we can make two copies of the initial
    //       values without requiring that the types be Copy.
    pub unsafe fn new<F, G>(new_ltr: F, new_rtl: G)->Self where
        F: Fn() -> LTR, G: Fn() -> RTL,
    {
        DoubleBuf{
            buf_a: DataBuf{
                ltr: new_ltr(),
                ltr_fresh: false,
                rtl: new_rtl(),
                rtl_fresh: false,
            }.into(),
            buf_b: DataBuf{
                ltr: new_ltr(),
                ltr_fresh: false,
                rtl: new_rtl(),
                rtl_fresh: false,
            }.into(),
            is_split: false,
            lock_a: PhantomData,
            lock_b: PhantomData,
        }
    }

    /// Splits the DoubleBuffer into a *Left* side and a *Right* side.
    /// Can only be called once; duplicate invocations will return None.
    /// Note that the original DoubleBuffer still owns the data and must live longer than either *Left* or *Right*.
    pub fn split(&mut self) -> Option<(Left<'_, LTR, RTL, A, B>, Right<'_, LTR, RTL, A, B>)>{
        // Make sure that spinlock B is actually free
        let _foo = Spinlock::<B>::try_claim().unwrap();
        drop(_foo);
        match self.is_split{
            true => {None}
            false => {
                self.is_split = true;
                Some((
                        Left{
                            buf_a: &self.buf_a,
                            lock_a: PhantomData,
                            buf_b: &self.buf_b,
                            lock_b: PhantomData,
                            // Claim the original spinlock, panicking if it fails
                            held_lock: HeldLock::A(Spinlock::<A>::try_claim().unwrap())
                        },
                        Right{
                            buf_a: &self.buf_a,
                            lock_a: PhantomData,
                            buf_b: &self.buf_b,
                            lock_b: PhantomData,
                        },
                ))
            }
        }

    }
}

/// One half of an asynchronous asymmetrical bidirectional double buffer.
/// This can be either a *Left* or a *Right*.
pub trait DoubleBufPort{
    type R;
    type W;

/// Read and write to a DoubleBuf.
/// # Arguments
///
/// * 'write' - Closure which writes data to the buffer. This will run every single time.
///
/// * 'read' - Closure which reads data from the buffer. This will *not* run if no fresh data is available!
///
/// When called on a *Left*, rw may briefly block waiting for the other side of the buffer to release.
/// When called on a *Right*, rw should never block.
/// No matter what, the write operation always happens first.
    fn rw<F, G>(&mut self, write: F, read: G) where
      F: FnMut(&mut Self::W), G: FnMut(& Self::R);
}

/// The *Left* side of a DoubleBuffer. Implements DoubleBufPort for reading and writing.
pub struct Left<'a, LTR, RTL, const A: usize, const B: usize> where
Spinlock<A>: SpinlockValid,
Spinlock<B>: SpinlockValid,
{
    buf_a: &'a UnsafeCell<DataBuf<LTR, RTL>>,
    lock_a: PhantomData<Spinlock<A>>,
    buf_b: &'a UnsafeCell<DataBuf<LTR, RTL>>,
    lock_b: PhantomData<Spinlock<B>>,
    held_lock: HeldLock<A, B>,
}

unsafe impl <LTR, RTL, const A: usize, const B: usize> Send for Left <'_, LTR, RTL, A, B> where
    Spinlock<A>: SpinlockValid,
    Spinlock<B>: SpinlockValid,
{}

unsafe impl <LTR, RTL, const A: usize, const B: usize> Sync for Left<'_, LTR, RTL, A, B> where
    Spinlock<A>: SpinlockValid,
    Spinlock<B>: SpinlockValid,
{}

impl <'a, LTR, RTL, const A: usize, const B: usize> DoubleBufPort for Left<'a, LTR, RTL, A, B> where
Spinlock<A>: SpinlockValid,
Spinlock<B>: SpinlockValid,
{
    type R = RTL;
    type W = LTR;

    fn rw<F, G>(&mut self, mut write: F, mut read: G) where
        F: FnMut(&mut Self::W), G: FnMut(& Self::R)
    {
        // Determine which lock is currently held and which buffer it corresponds to
        let (current_buf, next_buf) = match self.held_lock {
            HeldLock::A(_) => (self.buf_a, self.buf_b),
            HeldLock::B(_) => (self.buf_b, self.buf_a),
        };
        // Write data to held_lock buffer
          // Acquire a mut reference to the buffer from our UnsafeCell
        let dat: &mut DataBuf<LTR, RTL> = unsafe {&mut *current_buf.get()};
          // Call the provided closure
        write(&mut dat.ltr);
        dat.ltr_fresh = true;
          // Explicitly drop our mut reference to ensure it never coexists with
          // another mut reference in Right
        drop(dat);
        compiler_fence(Ordering::SeqCst); // Make sure the compiler doesn't pull tricks

        // Lock the other spinlock and release the currently held lock
        match self.held_lock {
            HeldLock::A(_) => {
                let new_lock = Spinlock::<B>::claim(); // FIRST, acquire the new lock
                compiler_fence(Ordering::SeqCst); // Make sure the compiler doesn't pull tricks
                self.held_lock = HeldLock::B(new_lock); // THEN, release the old lock
            }
            HeldLock::B(_) => {
                let new_lock = Spinlock::<A>::claim();
                compiler_fence(Ordering::SeqCst);
                self.held_lock = HeldLock::A(new_lock);
            }
        };
        // Check read flag and read
        let dat: &mut DataBuf<LTR, RTL> = unsafe {&mut *next_buf.get()};
        if dat.rtl_fresh == true {
            dat.rtl_fresh = false;
            read(& dat.rtl);
        }
        drop(dat);
    }
}

/// The *Right* side of a DoubleBuffer. Implements DoubleBufPort for reading and writing.
pub struct Right<'a, LTR, RTL, const A: usize, const B: usize> where
    Spinlock<A>: SpinlockValid,
    Spinlock<B>: SpinlockValid,
{
    buf_a: &'a UnsafeCell<DataBuf<LTR, RTL>>,
    lock_a: PhantomData<Spinlock<A>>,
    buf_b: &'a UnsafeCell<DataBuf<LTR, RTL>>,
    lock_b: PhantomData<Spinlock<B>>,
}

unsafe impl <LTR, RTL, const A: usize, const B: usize> Send for Right <'_, LTR, RTL, A, B> where
    Spinlock<A>: SpinlockValid,
    Spinlock<B>: SpinlockValid,
{}

unsafe impl <LTR, RTL, const A: usize, const B: usize> Sync for Right<'_, LTR, RTL, A, B> where
    Spinlock<A>: SpinlockValid,
    Spinlock<B>: SpinlockValid,
{}

impl <'a, LTR, RTL, const A: usize, const B: usize> DoubleBufPort for Right<'a, LTR, RTL, A, B> where
    Spinlock<A>: SpinlockValid,
    Spinlock<B>: SpinlockValid,
{
    type R = LTR;
    type W = RTL;

    fn rw<F, G>(&mut self, mut write: F, mut read: G) where
        F: FnMut(&mut Self::W), G: FnMut(& Self::R)
    {
        // Lock whichever buffer is available
        let (lock, buf) = loop{
            if let Some(i) = Spinlock::<A>::try_claim(){
                break (HeldLock::A(i), self.buf_a);
            }
            if let Some(i) = Spinlock::<B>::try_claim(){
                break (HeldLock::B(i), self.buf_b);
            }
        };
        compiler_fence(Ordering::SeqCst); // Make sure the compiler doesn't pull tricks
        // Write
        let dat: &mut DataBuf<LTR, RTL> = unsafe {&mut *buf.get()};
        write(&mut dat.rtl);
        dat.rtl_fresh = true;
        // Check read flag and read
        if dat.ltr_fresh == true {
            dat.ltr_fresh = false;
            read(& dat.ltr);
        }
        // Drop our mutable buffer reference explicitly, for clarity
        drop(dat);
        // Drop lock explicitly, for clarity
        drop(lock);
    }
}
