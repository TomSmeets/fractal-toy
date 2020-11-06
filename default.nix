{ pkgs ? import <nixpkgs> {} }: with pkgs; stdenv.mkDerivation rec {
  pname = "fractal-toy";
  version = "0.1.0";

  src = "/dev/null";

  buildInputs = [
    ocl-icd
    udev
    SDL2

    libGL
    xorg.libXrandr
    xorg.libXcursor
    xorg.libX11
    xorg.libXi
  ];

  LD_LIBRARY_PATH="${libGL}/lib";

  nativeBuildInputs = [
    cargo
    pkgconfig
  ];

  hardningDisable = [ "all" ];

  # CARGO_TARGET_DIR="/tmp/cargo/${pname}/";
}
