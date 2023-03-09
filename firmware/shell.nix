with import <nixpkgs> {
  crossSystem = (import <nixpkgs/lib>).systems.examples.arm-embedded // {
    rustc.config = "thumbv6m-none-eabi";
  };
};
let
  inputsBuild = with pkgsBuildBuild; [
    #cargo
    gcc
    probe-run
    elf2uf2-rs
    gdb
    # cargo-binutils
    # flip-link
  ];

  inputsHost = with pkgsBuildHost; [
    (pkgs.callPackage ./nix_stuff/rp_openocd.drv {})
    #rustc
    # llvmPackages.bintools-unwrapped
    # lld
    # gcc-arm-embedded
    # llvmPackages.clangNoCompilerRt
  ];

  inputs = inputsBuild ++ inputsHost ++ [
    (pkgs.symlinkJoin {
      name = "rust-toolchain";
      paths = [
        pkgsBuildHost.rustc
        pkgsBuildBuild.cargo
        pkgsBuildHost.rustPlatform.rustcSrc
      ];
    })
  ];

  lib_path = "${lib.makeLibraryPath inputs}";

in mkShell {
  name = "rust-env";

  nativeBuildInputs = inputs;

  buildInputs = [
  ];

  LD_LIBRARY_PATH = lib_path;
}
