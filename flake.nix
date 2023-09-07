{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    nixpkgs,
    utils,
    naersk,
    rust-overlay,
  }:
    utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [(import rust-overlay)];
      };
      rust = pkgs.rust-bin.beta.latest.default;
      naersk-lib = pkgs.callPackage naersk {
        cargo = rust;
        rustc = rust;
      };
    in {
      defaultPackage = naersk-lib.buildPackage ./.;
      devShell = with pkgs;
        mkShell {
          buildInputs = [rust pre-commit rustPackages.clippy];
          RUST_SRC_PATH = rust-bin.beta.latest.rust-src;
        };
    });
}
