# flake.nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };
  outputs = { self, nixpkgs, rust-overlay }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; overlays = [ rust-overlay.overlays.default ]; };
      rust = pkgs.rust-bin.stable.latest.default.override {
        extensions = [ "rust-src" "rust-analyzer" "clippy" "rustfmt" ];
      };
      runtimeLibs = with pkgs; [
        vulkan-loader libxkbcommon wayland
        xorg.libX11 xorg.libXcursor xorg.libXi xorg.libXrandr
      ];
    in {
      devShells.${system}.default = pkgs.mkShell {
        nativeBuildInputs = [ rust pkgs.pkg-config pkgs.mold pkgs.clang
          pkgs.sccache pkgs.cargo-nextest pkgs.bacon pkgs.just
          pkgs.mangohud pkgs.vulkan-tools pkgs.mesa-demos pkgs.intel-gpu-tools ];
        buildInputs = runtimeLibs;
        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath runtimeLibs;
        RUSTC_WRAPPER = "sccache";
      };
    };
}
