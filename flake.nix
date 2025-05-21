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
            cargo-udeps
            cargo-edit
            rust-analyzer
            cargo-deny
            rust-bin.stable.latest.default
            pkg-config
            dbus
            libxkbcommon
          ] ++ lib.optionals (stdenv.isLinux) [
            wayland
          ];
          LD_LIBRARY_PATH = lib.makeLibraryPath (nativeBuildInputs ++ buildInputs ++ runtimeDependencies);
        };
      }
    );
}
