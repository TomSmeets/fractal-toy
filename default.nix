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
    xorg.libXrandr
    xorg.libXcursor
    xorg.libX11
    xorg.libXi
  ];

  nativeBuildInputs = [
     pkgconfig
  ];

  LD_LIBRARY_PATH="${vulkan-loader}/lib";

  hardeningDisable = [ "all" ];
  cargoSha256 = "1nx8g5by631535k3y14178r4cp0qapgxarx7mb5iymfm1aficcfp";
}
