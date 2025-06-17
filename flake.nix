{
  description = "A playground for heuristic search algorithms";

  inputs = {
    nixpkgs = { url = "github:NixOS/nixpkgs"; };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay, }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" ];
      forEachSupportedSystem = f:
        nixpkgs.lib.genAttrs supportedSystems (system:
          f {
            pkgs = import nixpkgs {
              inherit system;
              overlays =
                [ rust-overlay.overlays.default self.overlays.default ];
            };
          });
    in {
      overlays.default = final: prev: {
        rustToolchain =
          prev.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      };

      devShells = forEachSupportedSystem ({ pkgs }: {
        default = pkgs.mkShell rec {
          nativeBuildInputs = with pkgs;
            [
              # Bevy (https://github.com/bevyengine/bevy/blob/main/docs/linux_dependencies.md#nix)
              pkg-config
            ];

          buildInputs = with pkgs; [
            rustToolchain

            llvmPackages.bintools
            llvmPackages.bolt
            mold
            rustc
            cargo
            rustup

            rust-jemalloc-sys

            # Bevy (https://github.com/bevyengine/bevy/blob/main/docs/linux_dependencies.md#nix)
            udev
            alsa-lib-with-plugins
            vulkan-loader

            libxkbcommon
            wayland
          ];

          packages = with pkgs; [
            hunspell
            hunspellDicts.en_GB-large

            # Nix
            nixfmt

            # Rust
            cargo-audit
            cargo-benchcmp
            cargo-criterion
            cargo-deny
            cargo-edit
            cargo-expand
            cargo-flamegraph
            cargo-fuzz
            cargo-outdated
            cargo-pgo
            cargo-public-api
            cargo-semver-checks
            cargo-show-asm
            cargo-spellcheck
            cargo-toml-lint
            cargo-valgrind
            cargo-watch

            bacon

            just

            lldb
            valgrind-light # light: without gdb

            coz
            critcmp

            gnuplot_qt

            ldtk
          ];

          env = {
            # Bevy (https://github.com/bevyengine/bevy/blob/main/docs/linux_dependencies.md#nix)
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;

            # Spelling
            DICTIONARY = "en_GB";
            DICPATH = "${pkgs.hunspell}/bin/hunspell";

            # Rust
            # RUSTFLAGS = "-C target-cpu=native";  # NOTE: This ruins reproducibility
            ## Required by rust-analyzer
            RUST_SRC_PATH =
              "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
          };
        };
      }); # ..devShells
    };
}
