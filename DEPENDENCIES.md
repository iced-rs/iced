# Dependencies

Iced requires some system dependencies to work, and not
all operating systems come with them installed.

You can follow the provided instructions for your system to
get them, if your system isn't here, add it!

## NixOS

You can add this `shell.nix` to your project and use it by running `nix-shell`:

```nix
{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell rec {
  buildInputs = with pkgs; [
    expat
    fontconfig
    freetype
    freetype.dev
    libGL
    pkg-config
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr
    wayland
    libxkbcommon
  ];

  LD_LIBRARY_PATH =
    builtins.foldl' (a: b: "${a}:${b}/lib") "${pkgs.vulkan-loader}/lib" buildInputs;
}
```

Alternatively, you can use this `flake.nix` to create a dev shell, activated by `nix develop` and build the package by `nix run`:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
        dependencies = with pkgs; [
          makeWrapper
          pkg-config
        ];
        runtimeDependencies = with pkgs;
        [ ]
        ++ lib.optionals stdenv.hostPlatform.isLinux
          libxkbcommon
          wayland
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
        ];
      in
      {
        defaultPackage = pkgs.rustPlatform.buildRustPackage {
          pname = "INSERT NAME OF APP";
          version = "0.1.0";
          src = ./.;
          nativeBuildInputs = runtimeDependencies;
          buildInputs = dependencies;
          cargoHash = pkgs.lib.fakeHash;
          postFixup = pkgs.lib.optional pkgs.stdenv.hostPlatform.isLinux (
            let
              rpathWayland = pkgs.lib.makeLibraryPath [
                pkgs.wayland
                pkgs.vulkan-loader
                pkgs.libxkbcommon
              ];
            in
            ''
              rpath=$(patchelf --print-rpath $out/bin/rust-qr)
              patchelf --set-rpath "$rpath:${rpathWayland}" $out/bin/rust-qr
            ''
          );
        };
        formatter = pkgs.nixfmt-tree;
        devShell = pkgs.mkShell {
          buildInputs = [
            dependencies
            runtimeDependencies
          ];
          LD_LIBRARY_PATH = builtins.foldl' (a: b: "${a}:${b}/lib") "${pkgs.vulkan-loader}/lib" runtimeDependencies;
        };
      }
    );
}
```
