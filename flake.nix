{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs =
    { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" ];
      forAllSystems =
        f:
        nixpkgs.lib.genAttrs systems (
          system:
          f {
            inherit system;
            pkgs = nixpkgs.legacyPackages.${system};
          }
        );
    in
    {
      devShells = forAllSystems (
        { system, pkgs }:
        {
          default = pkgs.mkShell {
            buildInputs = with pkgs; [
              # Core Rust toolchain
              rustc
              cargo
              pkg-config
              lld
              clang

              # Wayland/X11 windowing
              wayland
              wayland-protocols
              libxkbcommon
              libxkbcommon.dev
              xorg.libX11
              xorg.libXcursor
              xorg.libXi
              xorg.libXrandr

              # Graphics and rendering
              vulkan-loader
              vulkan-headers
              mesa

              # Audio
              alsa-lib

              # System libraries
              eudev
              fontconfig
              freetype
            ];

            shellHook = ''
              export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${
                pkgs.lib.makeLibraryPath [
                  pkgs.libxkbcommon
                  pkgs.wayland
                  pkgs.vulkan-loader
                  pkgs.mesa
                  pkgs.alsa-lib
                  pkgs.xorg.libX11
                  pkgs.xorg.libXcursor
                  pkgs.xorg.libXi
                  pkgs.xorg.libXrandr
                ]
              }"

              export PKG_CONFIG_PATH="$PKG_CONFIG_PATH:${
                pkgs.lib.makeSearchPathOutput "dev" "lib/pkgconfig" [
                  pkgs.libxkbcommon.dev
                  pkgs.wayland.dev
                  pkgs.wayland-protocols
                  pkgs.vulkan-headers
                  pkgs.alsa-lib.dev
                  pkgs.xorg.libX11.dev
                  pkgs.xorg.libXcursor.dev
                  pkgs.xorg.libXi.dev
                  pkgs.xorg.libXrandr.dev
                ]
              }"

              echo "Development environment loaded!"
              echo "Libraries available in: $LD_LIBRARY_PATH"
            '';
          };
        }
      );

      packages = forAllSystems (
        { system, pkgs }:
        {
          default = pkgs.buildRustCrate rec {
            pname = "bevy-procedural";
            version = "0.1.0";
            crateName = "${pname}-${version}";

            src = ./.;

            cargoLock = ./Cargo.lock;
          };
        }
      );
    };
}
