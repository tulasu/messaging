{
  description = "Rust devShell with stable rust-analyzer support and Fish shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rust = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" ];
        };

        rustAnalyzer = pkgs.rust-bin.stable.latest.rust-analyzer;

      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            rust
            rustAnalyzer
            pkgs.openssl
            pkgs.pkg-config
            pkgs.eza
            pkgs.fd
            pkgs.fish
          ];

          shellHook = ''
            export SHELL=${pkgs.fish}/bin/fish
            alias ls=eza
            alias find=fd
            export RUST_SRC_PATH="${rust}/lib/rustlib/src/rust/library"
            exec ${pkgs.fish}/bin/fish
          '';
        };
      }
    );
}