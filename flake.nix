{
description = "A Rust CLI application for Acetics";

inputs = {
  nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  utils.url = "github:numtide/flake-utils";
  rust-overlay = {
    url = "github:oxalica/rust-overlay";
    inputs = {
      nixpkgs.follows = "nixpkgs";
      flake-utils.follows = "utils";
    };
  };
};

outputs = { self, nixpkgs, utils, rust-overlay }:
  utils.lib.eachDefaultSystem (system:
    let
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs {
        inherit system overlays;
      };
      rustVersion = pkgs.rust-bin.stable.latest.default;
    in
    {
      devShell = pkgs.mkShell {
        buildInputs = with pkgs; [
          # Rust toolchain
          rustVersion
          rust-analyzer
          clippy
          rustfmt

          # Build essentials
          openssl
          pkg-config

          # Development tools
          nixpkgs-fmt
          cargo-edit
          cargo-watch
          cargo-audit

          # Version control
          git

          # HTTP client for testing APIs
          curl

          # Command runner
          just
        ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
          darwin.apple_sdk.frameworks.Security
          darwin.apple_sdk.frameworks.SystemConfiguration
        ];

        shellHook = ''
          export RUST_BACKTRACE=1
          export RUST_LOG=debug
          export OPENSSL_DIR="${pkgs.openssl.dev}"
          export OPENSSL_LIB_DIR="${pkgs.openssl.out}/lib"

          echo "Acetics CLI development environment loaded!"
          echo "Rust version: $(rustc --version)"
          echo "Cargo version: $(cargo --version)"
        '';

        # Set RUST_SRC_PATH for rust-analyzer
        RUST_SRC_PATH = "${rustVersion}/lib/rustlib/src/rust/library";
      };

      defaultPackage = pkgs.rustPlatform.buildRustPackage {
        pname = "acetics-cli";
        version = "0.1.0";
        src = ./.;
        cargoLock = {
          lockFile = ./Cargo.lock;
        };

        nativeBuildInputs = [ pkgs.pkg-config ];
        buildInputs = [ pkgs.openssl ];

        meta = with pkgs.lib; {
          description = "A simple CLI for Acetics";
          homepage = "https://github.com/Catvert/AceticsCli";
          license = licenses.mit;
          maintainers = [ "Arno Dupont" ];
        };
      };
    }
  );
}
