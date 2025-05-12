{
  description = "A playground for heuristic search algorithms";

  inputs = {
    nixpkgs = {
      url = "github:NixOS/nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
    }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forEachSupportedSystem =
        f:
        nixpkgs.lib.genAttrs supportedSystems (
          system:
          f {
            pkgs = import nixpkgs {
              inherit system;
              overlays = [
                rust-overlay.overlays.default
                self.overlays.default
              ];
            };
          }
        );
    in
    {
      overlays.default = final: prev: {
        rustToolchain = prev.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      };

      devShells = forEachSupportedSystem (
        { pkgs }:
        {
          default = pkgs.mkShell {
            buildInputs = with pkgs; [
              rustToolchain

              llvmPackages.bintools
              llvmPackages.bolt
              mold
              rustc
              cargo
              rustup

              rust-jemalloc-sys
            ];

            packages = with pkgs; [
              cargo-audit
              cargo-criterion
              cargo-deny
              cargo-edit
              cargo-expand
              cargo-flamegraph
              cargo-fuzz
              cargo-outdated
              cargo-pgo
              cargo-show-asm
              cargo-spellcheck
              cargo-valgrind
              cargo-watch

              bacon

              just

              lldb
              valgrind-light  # light: without gdb

              coz
              critcmp

              gnuplot_qt
            ];

            env = {
              # Required by rust-analyzer
              RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
            };
          };
        }
      );
    };
}
