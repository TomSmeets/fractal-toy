{ pkgs ? import <nixpkgs> {} }: with pkgs; rec {
    path = [
        stdenv.cc
        binutils-unwrapped
        rustup
    ];

    libs = with pkgs.xorg; [
        libX11
        libXcursor
        libXrandr
        libXi
    ];

    env = writeScript "env.sh" ''
        #!${pkgs.stdenv.shell}
        export LD_LIBRARY_PATH=${lib.makeLibraryPath libs}
        export RUSTFLAGS='${lib.concatMapStringsSep " " (x: "-L " + x + "/lib/") libs}'
        export PATH="${lib.makeBinPath path}:$PATH"
        exec ${pkgs.bashInteractive}/bin/bash
    '';
}
