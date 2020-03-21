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
        python3
    ];

    libs = with pkgs; [
        # probably want to statically link SDL2
        # SDL2

        (SDL2.overrideAttrs (pkg: {
          src = fetchurl {
            url = "https://hg.libsdl.org/SDL/archive/08db6a6f6c23.tar.bz2";
            sha256 = "10y0y5qskm1v68d9jmmxh7hw7cydn6n9rdyh2zgpn326a8354z0l";
          };

          configureFlags = [ "--enable-static" ]
            # ++ ["--enable-hidapi" ]
            ++ (pkg.configureFlags or []);

          NIX_CFLAGS_COMPILE = [ "-ffunction-sections"  "-fdata-sections" "-O2" "-fPIC" ];
          hardeningDisable = [ "all" ];
          postInstall = "";
      	}))
      	sndio

      	openssl

      	# xorg.libX11
        # xorg.libXcursor
        # xorg.libXrandr
        # xorg.libXi
        xorg.libxcb
        xorg.xorgproto

		libGL
		alsaLib
		udev

        pkgs.xorg.libXext
        pkgs.xorg.libXinerama
        pkgs.xorg.libXrandr
        pkgs.xorg.libXcursor
        pkgs.xorg.libX11
        pkgs.xorg.libXi
        pkgs.xorg.libXxf86vm
        pkgs.xorg.libXScrnSaver
    ];

    env = writeScript "env.sh" ''
        #!${pkgs.stdenv.shell}
        export hardeningDisable=all
        export LD_LIBRARY_PATH=${lib.makeLibraryPath libs}
        export RUSTFLAGS='${lib.concatMapStringsSep " " (x: "-L " + x + "/lib/") libs}'
        export RUSTDOCFLAGS="$RUSTFLAGS"
        export PATH="${lib.makeBinPath path}:$PATH"
        export PKG_CONFIG_PATH='${lib.concatMapStringsSep ":" (x: (x.dev or x) + "/share/pkgconfig:" + (x.dev or x) + "/lib/pkgconfig") libs}'

        export FONT_DEJAVU='${pkgs.dejavu_fonts}/share/fonts/truetype'
        exec ${pkgs.bashInteractive}/bin/bash
    '';
}
