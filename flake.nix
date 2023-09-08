{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    header-vim = {
      url = "github:42Paris/42header";
      flake = false;
    };
  };

  outputs = {
    self,
    nixpkgs,
    utils,
    naersk,
    rust-overlay,
    header-vim,
  }:
    utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [(import rust-overlay)];
      };
      rust = pkgs.rust-bin.nightly.latest.default;
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
          HEADER_PLUGIN_PATH = "${header-vim}/plugin/stdheader.vim";
        };
    });
}
