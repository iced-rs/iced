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
