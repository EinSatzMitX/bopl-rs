{pkgs ? import <nixpkgs> {}}: let
  lib = pkgs.lib;
  libPath = lib.makeLibraryPath [
    pkgs.libGL
    pkgs.libxkbcommon
    pkgs.wayland
  ];
in
  with pkgs;
    mkShell {
      buildInputs = [
        cargo
        rustc
        rust-analyzer
      ];

      shellHook = ''
        export LD_LIBRARY_PATH=${libPath}:$LD_LIBRARY_PATH
        export RUST_LOG=debug
        export RUST_SRC_PATH=${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}
      '';
    }
