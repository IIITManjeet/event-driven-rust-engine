{
  description = "Rust Event-Driven Trading System";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            pkgs.rustc
            pkgs.cargo
            pkgs.rustfmt
            pkgs.clippy
            pkgs.openssl
            pkgs.pkg-config
          ];

          shellHook = ''
            echo "🚀 Rust Event-Driven Trading Dev Shell Ready"
          '';
        };
      }
    );
}
