{
  description = "Rust devshell for Iced";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      with pkgs;
      {
        devShells.default = mkShell rec {
          buildInputs = [
            bacon
            cargo-edit
            rust-analyzer
            rust-bin.stable.latest.default

            pkg-config
            dbus
            libxkbcommon
            expat
            fontconfig
            freetype
            freetype.dev
            libGL
            libxkbcommon
          ] ++ lib.optionals (stdenv.isLinux) [
            wayland
            xorg.libX11
            xorg.libXcursor
            xorg.libXi
            xorg.libXrandr
          ];
          LD_LIBRARY_PATH = lib.makeLibraryPath (buildInputs);
        };
      }
    );
}
