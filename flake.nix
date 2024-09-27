{
  description = "Nix development dependencies for unic-project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, ... } @ inputs:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            inputs.rust-overlay.overlays.default
          ];
        };
      in rec {
        devShell = pkgs.mkShell {
          packages = with pkgs; [
            (pkgs.rust-bin.nightly.latest.default.override {
              extensions = [
                "rust-src"
                "clippy"
                "rust-analyzer"
              ];
              targets = [
                stdenv.hostPlatform.rust.rustcTarget
                "wasm32-unknown-unknown"
              ];
            })
            wasm-pack
            wasm-bindgen-cli
            nodejs
          ];
        };
      }
    );
}
