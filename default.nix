{ pkgs ? import <nixpkgs> {} }: with pkgs;
rustPlatform.buildRustPackage rec {
  pname = "fractal-toy";
  version = "0.1.0";

  src = builtins.filterSource (p: t: !builtins.elem (toString p) (map toString [
    ./.git
    ./target
    ./result
    ./default.nix
  ])) ./.;

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

  nativeBuildInputs = [
    pkgconfig
  ];

  LD_LIBRARY_PATH="${libGL}/lib";

  hardeningDisable = [ "all" ];
  cargoSha256 = "1mmib87jaqh7sd7x1m4pmhi56nrb21y876wjvak861k6csbsiiyg";
}
