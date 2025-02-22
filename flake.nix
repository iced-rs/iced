{
  description = "Iced dev";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";  # Specify the Nixpkgs version
  };

  outputs = { self, nixpkgs }:
  let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};
  in
  {
		devShells.${system} = {
			# mkShell.override {
			# 	stdenv = pkgs.clangStdenv;
			# };

			default = pkgs.mkShell.override { stdenv = pkgs.clangStdenv; } {
    		    packages = with pkgs; [
    		      cargo
    		      rustc
    		      rust-analyzer
    		      rustfmt

				  python3
				  ninja
				  # clang
				  # clang-tools

    		      libxkbcommon
    		      wayland

				  vulkan-loader
				  vulkan-validation-layers
				  vulkan-tools

				  libappindicator

				  openssl
				  pkgs.pkg-config

				  gtk3
				  xdotool
				  libayatana-appindicator
    		    ];
				LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
					pkgs.libxkbcommon
					pkgs.wayland
					pkgs.vulkan-loader
    			
					pkgs.freetype
					pkgs.fontconfig
					pkgs.libinput
					pkgs.qt5.full


					pkgs.libayatana-appindicator
				];

    		    RUST_BACKTRACE = "full";
    		    WINIT_UNIX_BACKEND = "wayland";
    		};
		};
	};
}
