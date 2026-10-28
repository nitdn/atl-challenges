{
  description = "Description for the project";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        # To import a flake module
        # 1. Add foo to inputs
        # 2. Add foo as a parameter to the outputs function
        # 3. Add here: foo.flakeModule

      ];
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
        "x86_64-darwin"
      ];
      perSystem =
        {
          lib,
          pkgs,
          self',
          ...
        }:
        let
          buildInputs = with pkgs; [
            ffmpeg
            mediainfo
            sdl3
            wayland
            wayland-protocols
            libxkbcommon
            vulkan-loader
          ];
          ocamlPackages = pkgs.ocamlPackages_latest;
          ocamlBuildInputs = with ocamlPackages; [
            findlib
            dune_3
            batteries
            ocaml
          ];

        in
        {

          devShells.default = pkgs.mkShell.override { stdenv = pkgs.clangStdenv; } {
            nativeBuildInputs = with pkgs; [
              cargo
              wayland-scanner
              pkg-config
            ];
            packages =
              with pkgs;
              [
                bacon
                inetutils
                meld
                watchexec
                trunk
                ocamlPackages.ocaml-lsp
                ocamlPackages.ocamlformat
                ocamlPackages.utop
              ]
              ++ ocamlBuildInputs
              ++ buildInputs;
            TMPDIR = "~/Documents/atl-challenges/target";
            shellHook = ''
              NIX_CFLAGS_COMPILE="$(pkg-config --libs \
              # does not actually matter as long as it \
              # is a wayland-* in pkg-config \
              # partly because we only need the -rpath \
              # set up by the ld-wrapper \
              wayland-egl \
              xkbcommon \
              vulkan \
              ) $NIX_CFLAGS_COMPILE"
            '';
            # RUSTFLAGS = "-C link-arg=-Wl,-rpath,${lib.makeLibraryPath buildInputs}";
            # LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.libclang ];
            # LD_LIBRARY_PATH = builtins.toString (pkgs.lib.makeLibraryPath buildInputs);
          };
        };
    };
}
