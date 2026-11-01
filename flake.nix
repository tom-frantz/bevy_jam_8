{
  description = "Bevy Game Jam #8";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url  = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
      in {
        devShells.default = with pkgs; mkShell rec {
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [
            # Rust
            # (rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
              # extensions = ["rust-analyzer" "rust-src"];
            # }))
            (rust-bin.nightly."2026-10-31".default.override {
              extensions = ["rust-analyzer" "rust-src"];
              targets = ["wasm32-unknown-unknown"];
            })

            # Extra dev tools 
            bacon  # cargo install bacon
            cargo-expand

            # General
            openssl
          ] ++ lib.optionals stdenv.hostPlatform.isLinux [
            # General
            udev  # Device manager (not necessary????)
            alsa-lib-with-plugins  # Audio
            vulkan-loader vulkan-tools # Graphics  

            # X11 features
            libx11 libxcursor libxi libxrandr
            
            # Wayland features
            libxkbcommon wayland
          ];
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath ([
            
          ] ++ lib.optionals stdenv.hostPlatform.isLinux [
            # stolen from bevy
            # https://github.com/bevyengine/bevy/blob/latest/docs/linux_dependencies.md
            vulkan-loader
            libx11
            libxi
            libxcursor
            libxkbcommon
            wayland
          ]);
          shellHook = ''
            # Set fixed cache (only useful for multiple non-workspace crates)
            export CARGO_TARGET_DIR="$(pwd)/target";
            # add local cargo installs to the bin
            # cargo install --root .cargo ccase &
            export PATH="$(pwd)/.cargo/bin:$PATH";
          ''; 
        };
      });
}
