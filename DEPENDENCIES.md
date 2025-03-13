# Dependencies

Iced requires some system dependencies to work, and not
all operating systems come with them installed.

You can follow the provided instructions for your system to
get them, if your system isn't here, add it!

## NixOS

You can add this `shell.nix` to your project and use it by running `nix-shell`:

```nix
{ pkgs ? import <nixpkgs> { } }:

let
  dlopenLibraries = with pkgs; [
    libxkbcommon

    # GPU backend
    vulkan-loader
    # libGL

    # Window system
    wayland
    # xorg.libX11
    # xorg.libXcursor
    # xorg.libXi
  ];
in pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    cargo
    rustc
  ];

  # additional libraries that your project
  # links to at build time, e.g. OpenSSL
  buildInputs = [];

  env.RUSTFLAGS = "-C link-arg=-Wl,-rpath,${pkgs.lib.makeLibraryPath dlopenLibraries}";
}
```

Alternatively, you can use this `flake.nix` to create a dev shell, activated by `nix develop`:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    systems.url = "github:nix-systems/default";
  };

  outputs = { nixpkgs, systems, ... }:
    let
      eachSystem = nixpkgs.lib.genAttrs (import systems);
      pkgsFor = nixpkgs.legacyPackages;
    in {
      devShells = eachSystem (system:
        let
          pkgs = pkgsFor.${system};
          dlopenLibraries = with pkgs; [
            libxkbcommon

            # GPU backend
            vulkan-loader
            # libGL

            # Window system
            wayland
            # xorg.libX11
            # xorg.libXcursor
            # xorg.libXi
          ];
        in {
          default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              cargo
              rustc
            ];

            # additional libraries that your project
            # links to at build time, e.g. OpenSSL
            buildInputs = [];

            env.RUSTFLAGS = "-C link-arg=-Wl,-rpath,${nixpkgs.lib.makeLibraryPath dlopenLibraries}";
          };
        });
    };
}
```
