{
  description = "Cross compiling a rust program using cargo-zigbuild";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          targets = [
            "aarch64-unknown-linux-gnu"
            "x86_64-unknown-linux-gnu"
          ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            cargo-zigbuild
            zig
            systemd
            pkg-config
            rust-analyzer
          ];

        };
      }
    );
}
