let
  mozillaOverlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  nixpkgs = import <nixpkgs> { overlays = [ mozillaOverlay ]; };

  rust = (nixpkgs.rustChannels.stable.rust.override {
    targets = [ "x86_64-unknown-linux-musl" "wasm32-unknown-unknown" ];
  });

  rustPlatform = nixpkgs.makeRustPlatform {
    cargo = rust;
    rustc = rust;
  };
in
  nixpkgs.stdenv.mkDerivation {
    name = "thisisatest";
    src = "";
    nativeBuildInputs = [ rust ];
  }
