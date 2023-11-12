# based on https://github.com/oxalica/rust-overlay#use-in-devshell-for-nix-develop
{
  description = "A Nix-devShell to build/develop this project";

  inputs = {
    # `nixpkgs-unstable` is fully ok for an application (i.e., not a NixOS-system)
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    # `rust-overlay` can give us a rust-version that is in-sync with rust-toolchain.toml
    rust-overlay.url = "github:oxalica/rust-overlay";

    # `flake-utils` for easier nix-system handling
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      with pkgs;
      {
        devShells.default = mkShell {
          buildInputs = [
            # openssl
            # pkg-config

            # use rust-version + components from the rust-toolchain.toml file
            (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
          ];

          # shellHook = ''
          #   alias ls=eza
          #   alias find=fd
          # '';
        };
      }
    );
}
