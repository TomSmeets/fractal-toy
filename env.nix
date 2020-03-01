{ pkgs ? import <nixpkgs> {} }: with pkgs; rec {

    cc = pkgs.runCommand "cc" { } ''
      mkdir -p $out/bin
      ln -s ${stdenv.cc.cc}/bin/gcc $out/bin/cc
    '';

    path = [
        stdenv.cc
        binutils-unwrapped
        rustup
    ];

    libs = with pkgs; [
        SDL2
        xorg.libX11
        xorg.libXcursor
        xorg.libXrandr
        xorg.libXi
    ];

    env = writeScript "env.sh" ''
        #!${pkgs.stdenv.shell}
        export hardeningDisable=all
        export LD_LIBRARY_PATH=${lib.makeLibraryPath libs}
        export RUSTFLAGS='${lib.concatMapStringsSep " " (x: "-L " + x + "/lib/") libs}'
        export PATH="${lib.makeBinPath path}:$PATH"
        exec ${pkgs.bashInteractive}/bin/bash
    '';
}
