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
      {
        devShells.default = pkgs.mkShell rec {
          nativeBuildInputs = with pkgs; [
            # from https://github.com/bevyengine/bevy/blob/main/docs/linux_dependencies.md#Nix
            pkg-config

            # from https://bevyengine.org/learn/book/getting-started/setup/#enable-fast-compiles-optional
            mold-wrapped
            clang_16

            # use rust-version + components from the rust-toolchain.toml file
            (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
          ];

          buildInputs = with pkgs; [
            # common bevy dependencies
            udev
            alsa-lib
            vulkan-loader

            # bevy x11 feature
            xorg.libX11
            xorg.libXcursor
            xorg.libXi
            xorg.libXrandr

            # bevy wayland feature
            libxkbcommon
            wayland

            # often this becomes necessary sooner or later
            # openssl
          ];

          # from bevy setup as well
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
        };
      }
    );
}
