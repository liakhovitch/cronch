window.SIDEBAR_ITEMS = {"mod":[["cpuid","Processor core identifier Value is 0 when read from processor core 0, and 1 when read from processor core 1."],["div_csr","Control and status register for divider."],["div_quotient","Divider result quotient The result of `DIVIDEND / DIVISOR` (division). Contents undefined while CSR_READY is low. For signed calculations, QUOTIENT is negative when the signs of DIVIDEND and DIVISOR differ. This register can be written to directly, for context save/restore purposes. This halts any in-progress calculation and sets the CSR_READY and CSR_DIRTY flags. Reading from QUOTIENT clears the CSR_DIRTY flag, so should read results in the order REMAINDER, QUOTIENT if CSR_DIRTY is used."],["div_remainder","Divider result remainder The result of `DIVIDEND % DIVISOR` (modulo). Contents undefined while CSR_READY is low. For signed calculations, REMAINDER is negative only when DIVIDEND is negative. This register can be written to directly, for context save/restore purposes. This halts any in-progress calculation and sets the CSR_READY and CSR_DIRTY flags."],["div_sdividend","Divider signed dividend The same as UDIVIDEND, but starts a signed calculation, rather than unsigned."],["div_sdivisor","Divider signed divisor The same as UDIVISOR, but starts a signed calculation, rather than unsigned."],["div_udividend","Divider unsigned dividend Write to the DIVIDEND operand of the divider, i.e. the p in `p / q`. Any operand write starts a new calculation. The results appear in QUOTIENT, REMAINDER. UDIVIDEND/SDIVIDEND are aliases of the same internal register. The U alias starts an unsigned calculation, and the S alias starts a signed calculation."],["div_udivisor","Divider unsigned divisor Write to the DIVISOR operand of the divider, i.e. the q in `p / q`. Any operand write starts a new calculation. The results appear in QUOTIENT, REMAINDER. UDIVISOR/SDIVISOR are aliases of the same internal register. The U alias starts an unsigned calculation, and the S alias starts a signed calculation."],["fifo_rd","Read access to this core’s RX FIFO"],["fifo_st","Status register for inter-core FIFOs (mailboxes). There is one FIFO in the core 0 -> core 1 direction, and one core 1 -> core 0. Both are 32 bits wide and 8 words deep. Core 0 can see the read side of the 1->0 FIFO (RX), and the write side of 0->1 FIFO (TX). Core 1 can see the read side of the 0->1 FIFO (RX), and the write side of 1->0 FIFO (TX). The SIO IRQ for each core is the logical OR of the VLD, WOF and ROE fields of its FIFO_ST register."],["fifo_wr","Write access to this core’s TX FIFO"],["gpio_hi_in","Input value for QSPI pins"],["gpio_hi_oe","QSPI output enable"],["gpio_hi_oe_clr","QSPI output enable clear"],["gpio_hi_oe_set","QSPI output enable set"],["gpio_hi_oe_xor","QSPI output enable XOR"],["gpio_hi_out","QSPI output value"],["gpio_hi_out_clr","QSPI output value clear"],["gpio_hi_out_set","QSPI output value set"],["gpio_hi_out_xor","QSPI output value XOR"],["gpio_in","Input value for GPIO pins"],["gpio_oe","GPIO output enable"],["gpio_oe_clr","GPIO output enable clear"],["gpio_oe_set","GPIO output enable set"],["gpio_oe_xor","GPIO output enable XOR"],["gpio_out","GPIO output value"],["gpio_out_clr","GPIO output value clear"],["gpio_out_set","GPIO output value set"],["gpio_out_xor","GPIO output value XOR"],["interp0_accum0","Read/write access to accumulator 0"],["interp0_accum0_add","Values written here are atomically added to ACCUM0 Reading yields lane 0’s raw shift and mask value (BASE0 not added)."],["interp0_accum1","Read/write access to accumulator 1"],["interp0_accum1_add","Values written here are atomically added to ACCUM1 Reading yields lane 1’s raw shift and mask value (BASE1 not added)."],["interp0_base0","Read/write access to BASE0 register."],["interp0_base1","Read/write access to BASE1 register."],["interp0_base2","Read/write access to BASE2 register."],["interp0_base_1and0","On write, the lower 16 bits go to BASE0, upper bits to BASE1 simultaneously. Each half is sign-extended to 32 bits if that lane’s SIGNED flag is set."],["interp0_ctrl_lane0","Control register for lane 0"],["interp0_ctrl_lane1","Control register for lane 1"],["interp0_peek_full","Read FULL result, without altering any internal state (PEEK)."],["interp0_peek_lane0","Read LANE0 result, without altering any internal state (PEEK)."],["interp0_peek_lane1","Read LANE1 result, without altering any internal state (PEEK)."],["interp0_pop_full","Read FULL result, and simultaneously write lane results to both accumulators (POP)."],["interp0_pop_lane0","Read LANE0 result, and simultaneously write lane results to both accumulators (POP)."],["interp0_pop_lane1","Read LANE1 result, and simultaneously write lane results to both accumulators (POP)."],["interp1_accum0","Read/write access to accumulator 0"],["interp1_accum0_add","Values written here are atomically added to ACCUM0 Reading yields lane 0’s raw shift and mask value (BASE0 not added)."],["interp1_accum1","Read/write access to accumulator 1"],["interp1_accum1_add","Values written here are atomically added to ACCUM1 Reading yields lane 1’s raw shift and mask value (BASE1 not added)."],["interp1_base0","Read/write access to BASE0 register."],["interp1_base1","Read/write access to BASE1 register."],["interp1_base2","Read/write access to BASE2 register."],["interp1_base_1and0","On write, the lower 16 bits go to BASE0, upper bits to BASE1 simultaneously. Each half is sign-extended to 32 bits if that lane’s SIGNED flag is set."],["interp1_ctrl_lane0","Control register for lane 0"],["interp1_ctrl_lane1","Control register for lane 1"],["interp1_peek_full","Read FULL result, without altering any internal state (PEEK)."],["interp1_peek_lane0","Read LANE0 result, without altering any internal state (PEEK)."],["interp1_peek_lane1","Read LANE1 result, without altering any internal state (PEEK)."],["interp1_pop_full","Read FULL result, and simultaneously write lane results to both accumulators (POP)."],["interp1_pop_lane0","Read LANE0 result, and simultaneously write lane results to both accumulators (POP)."],["interp1_pop_lane1","Read LANE1 result, and simultaneously write lane results to both accumulators (POP)."],["spinlock","Reading from a spinlock address will:"],["spinlock_st","Spinlock state A bitmap containing the state of all 32 spinlocks (1=locked). Mainly intended for debugging."]],"struct":[["RegisterBlock","Register block"]],"type":[["CPUID","CPUID register accessor: an alias for `Reg<CPUID_SPEC>`"],["DIV_CSR","DIV_CSR register accessor: an alias for `Reg<DIV_CSR_SPEC>`"],["DIV_QUOTIENT","DIV_QUOTIENT register accessor: an alias for `Reg<DIV_QUOTIENT_SPEC>`"],["DIV_REMAINDER","DIV_REMAINDER register accessor: an alias for `Reg<DIV_REMAINDER_SPEC>`"],["DIV_SDIVIDEND","DIV_SDIVIDEND register accessor: an alias for `Reg<DIV_SDIVIDEND_SPEC>`"],["DIV_SDIVISOR","DIV_SDIVISOR register accessor: an alias for `Reg<DIV_SDIVISOR_SPEC>`"],["DIV_UDIVIDEND","DIV_UDIVIDEND register accessor: an alias for `Reg<DIV_UDIVIDEND_SPEC>`"],["DIV_UDIVISOR","DIV_UDIVISOR register accessor: an alias for `Reg<DIV_UDIVISOR_SPEC>`"],["FIFO_RD","FIFO_RD register accessor: an alias for `Reg<FIFO_RD_SPEC>`"],["FIFO_ST","FIFO_ST register accessor: an alias for `Reg<FIFO_ST_SPEC>`"],["FIFO_WR","FIFO_WR register accessor: an alias for `Reg<FIFO_WR_SPEC>`"],["GPIO_HI_IN","GPIO_HI_IN register accessor: an alias for `Reg<GPIO_HI_IN_SPEC>`"],["GPIO_HI_OE","GPIO_HI_OE register accessor: an alias for `Reg<GPIO_HI_OE_SPEC>`"],["GPIO_HI_OE_CLR","GPIO_HI_OE_CLR register accessor: an alias for `Reg<GPIO_HI_OE_CLR_SPEC>`"],["GPIO_HI_OE_SET","GPIO_HI_OE_SET register accessor: an alias for `Reg<GPIO_HI_OE_SET_SPEC>`"],["GPIO_HI_OE_XOR","GPIO_HI_OE_XOR register accessor: an alias for `Reg<GPIO_HI_OE_XOR_SPEC>`"],["GPIO_HI_OUT","GPIO_HI_OUT register accessor: an alias for `Reg<GPIO_HI_OUT_SPEC>`"],["GPIO_HI_OUT_CLR","GPIO_HI_OUT_CLR register accessor: an alias for `Reg<GPIO_HI_OUT_CLR_SPEC>`"],["GPIO_HI_OUT_SET","GPIO_HI_OUT_SET register accessor: an alias for `Reg<GPIO_HI_OUT_SET_SPEC>`"],["GPIO_HI_OUT_XOR","GPIO_HI_OUT_XOR register accessor: an alias for `Reg<GPIO_HI_OUT_XOR_SPEC>`"],["GPIO_IN","GPIO_IN register accessor: an alias for `Reg<GPIO_IN_SPEC>`"],["GPIO_OE","GPIO_OE register accessor: an alias for `Reg<GPIO_OE_SPEC>`"],["GPIO_OE_CLR","GPIO_OE_CLR register accessor: an alias for `Reg<GPIO_OE_CLR_SPEC>`"],["GPIO_OE_SET","GPIO_OE_SET register accessor: an alias for `Reg<GPIO_OE_SET_SPEC>`"],["GPIO_OE_XOR","GPIO_OE_XOR register accessor: an alias for `Reg<GPIO_OE_XOR_SPEC>`"],["GPIO_OUT","GPIO_OUT register accessor: an alias for `Reg<GPIO_OUT_SPEC>`"],["GPIO_OUT_CLR","GPIO_OUT_CLR register accessor: an alias for `Reg<GPIO_OUT_CLR_SPEC>`"],["GPIO_OUT_SET","GPIO_OUT_SET register accessor: an alias for `Reg<GPIO_OUT_SET_SPEC>`"],["GPIO_OUT_XOR","GPIO_OUT_XOR register accessor: an alias for `Reg<GPIO_OUT_XOR_SPEC>`"],["INTERP0_ACCUM0","INTERP0_ACCUM0 register accessor: an alias for `Reg<INTERP0_ACCUM0_SPEC>`"],["INTERP0_ACCUM0_ADD","INTERP0_ACCUM0_ADD register accessor: an alias for `Reg<INTERP0_ACCUM0_ADD_SPEC>`"],["INTERP0_ACCUM1","INTERP0_ACCUM1 register accessor: an alias for `Reg<INTERP0_ACCUM1_SPEC>`"],["INTERP0_ACCUM1_ADD","INTERP0_ACCUM1_ADD register accessor: an alias for `Reg<INTERP0_ACCUM1_ADD_SPEC>`"],["INTERP0_BASE0","INTERP0_BASE0 register accessor: an alias for `Reg<INTERP0_BASE0_SPEC>`"],["INTERP0_BASE1","INTERP0_BASE1 register accessor: an alias for `Reg<INTERP0_BASE1_SPEC>`"],["INTERP0_BASE2","INTERP0_BASE2 register accessor: an alias for `Reg<INTERP0_BASE2_SPEC>`"],["INTERP0_BASE_1AND0","INTERP0_BASE_1AND0 register accessor: an alias for `Reg<INTERP0_BASE_1AND0_SPEC>`"],["INTERP0_CTRL_LANE0","INTERP0_CTRL_LANE0 register accessor: an alias for `Reg<INTERP0_CTRL_LANE0_SPEC>`"],["INTERP0_CTRL_LANE1","INTERP0_CTRL_LANE1 register accessor: an alias for `Reg<INTERP0_CTRL_LANE1_SPEC>`"],["INTERP0_PEEK_FULL","INTERP0_PEEK_FULL register accessor: an alias for `Reg<INTERP0_PEEK_FULL_SPEC>`"],["INTERP0_PEEK_LANE0","INTERP0_PEEK_LANE0 register accessor: an alias for `Reg<INTERP0_PEEK_LANE0_SPEC>`"],["INTERP0_PEEK_LANE1","INTERP0_PEEK_LANE1 register accessor: an alias for `Reg<INTERP0_PEEK_LANE1_SPEC>`"],["INTERP0_POP_FULL","INTERP0_POP_FULL register accessor: an alias for `Reg<INTERP0_POP_FULL_SPEC>`"],["INTERP0_POP_LANE0","INTERP0_POP_LANE0 register accessor: an alias for `Reg<INTERP0_POP_LANE0_SPEC>`"],["INTERP0_POP_LANE1","INTERP0_POP_LANE1 register accessor: an alias for `Reg<INTERP0_POP_LANE1_SPEC>`"],["INTERP1_ACCUM0","INTERP1_ACCUM0 register accessor: an alias for `Reg<INTERP1_ACCUM0_SPEC>`"],["INTERP1_ACCUM0_ADD","INTERP1_ACCUM0_ADD register accessor: an alias for `Reg<INTERP1_ACCUM0_ADD_SPEC>`"],["INTERP1_ACCUM1","INTERP1_ACCUM1 register accessor: an alias for `Reg<INTERP1_ACCUM1_SPEC>`"],["INTERP1_ACCUM1_ADD","INTERP1_ACCUM1_ADD register accessor: an alias for `Reg<INTERP1_ACCUM1_ADD_SPEC>`"],["INTERP1_BASE0","INTERP1_BASE0 register accessor: an alias for `Reg<INTERP1_BASE0_SPEC>`"],["INTERP1_BASE1","INTERP1_BASE1 register accessor: an alias for `Reg<INTERP1_BASE1_SPEC>`"],["INTERP1_BASE2","INTERP1_BASE2 register accessor: an alias for `Reg<INTERP1_BASE2_SPEC>`"],["INTERP1_BASE_1AND0","INTERP1_BASE_1AND0 register accessor: an alias for `Reg<INTERP1_BASE_1AND0_SPEC>`"],["INTERP1_CTRL_LANE0","INTERP1_CTRL_LANE0 register accessor: an alias for `Reg<INTERP1_CTRL_LANE0_SPEC>`"],["INTERP1_CTRL_LANE1","INTERP1_CTRL_LANE1 register accessor: an alias for `Reg<INTERP1_CTRL_LANE1_SPEC>`"],["INTERP1_PEEK_FULL","INTERP1_PEEK_FULL register accessor: an alias for `Reg<INTERP1_PEEK_FULL_SPEC>`"],["INTERP1_PEEK_LANE0","INTERP1_PEEK_LANE0 register accessor: an alias for `Reg<INTERP1_PEEK_LANE0_SPEC>`"],["INTERP1_PEEK_LANE1","INTERP1_PEEK_LANE1 register accessor: an alias for `Reg<INTERP1_PEEK_LANE1_SPEC>`"],["INTERP1_POP_FULL","INTERP1_POP_FULL register accessor: an alias for `Reg<INTERP1_POP_FULL_SPEC>`"],["INTERP1_POP_LANE0","INTERP1_POP_LANE0 register accessor: an alias for `Reg<INTERP1_POP_LANE0_SPEC>`"],["INTERP1_POP_LANE1","INTERP1_POP_LANE1 register accessor: an alias for `Reg<INTERP1_POP_LANE1_SPEC>`"],["SPINLOCK","SPINLOCK register accessor: an alias for `Reg<SPINLOCK_SPEC>`"],["SPINLOCK_ST","SPINLOCK_ST register accessor: an alias for `Reg<SPINLOCK_ST_SPEC>`"]]};