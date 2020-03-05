{ pkgs ? import <nixpkgs> {} }: with pkgs; rec {

    cc = pkgs.runCommand "cc" { } ''
      mkdir -p $out/bin
      ln -s ${stdenv.cc.cc}/bin/gcc $out/bin/cc
    '';

    path = [
        stdenv.cc
        binutils-unwrapped
        rustup
        pkgconfig
        cmake
        m4
    ];

    libs = with pkgs; [
        SDL2
        xorg.libX11
        xorg.libXcursor
        xorg.libXrandr
        xorg.libXi
        xorg.libxcb
        xorg.xorgproto
    ];

    env = writeScript "env.sh" ''
        #!${pkgs.stdenv.shell}
        export hardeningDisable=all
        export LD_LIBRARY_PATH=${lib.makeLibraryPath libs}
        export RUSTFLAGS='${lib.concatMapStringsSep " " (x: "-L " + x + "/lib/") libs}'
        export PATH="${lib.makeBinPath path}:$PATH"
        export PKG_CONFIG_PATH='${lib.concatMapStringsSep ":" (x: (x.dev or x) + "/share/pkgconfig:" + (x.dev or x) + "/lib/pkgconfig") libs}'
        exec ${pkgs.bashInteractive}/bin/bash
    '';
}
