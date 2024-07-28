{
  description = "A Rust CLI application";

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
            rustVersion
            openssl
            pkg-config
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.darwin.apple_sdk.frameworks.Security
          ];

          shellHook = ''
            export RUST_BACKTRACE=1
          '';
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
            license = licenses.mit;  # Adjust to your license
            maintainers = [ "Arno Dupont" ];
          };
        };
      }
    );
}
