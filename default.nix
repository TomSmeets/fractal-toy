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
  cargoSha256 = "1pmjfi9gifzdyv6fzgzlvqlml7p8jh3hvfi75hnk77g39rngii4c";
}
