{ pkgs ? import <nixpkgs> {} }: rec {
  lib = pkgs.lib;

  sdl2 = p: p.SDL2.overrideAttrs (pkg: {
    src = pkgs.fetchurl {
      url = "https://hg.libsdl.org/SDL/archive/08db6a6f6c23.tar.bz2";
      sha256 = "10y0y5qskm1v68d9jmmxh7hw7cydn6n9rdyh2zgpn326a8354z0l";
    };

    configureFlags = [
      "--enable-static"
      # "--enable-hidapi"
    ] ++ (pkg.configureFlags or []);

    NIX_CFLAGS_COMPILE = [ "-ffunction-sections"  "-fdata-sections" "-O2" ];
    hardeningDisable = [ "all" ];
    postInstall = "";
  });


  sdl2_win = p: (sdl2 p).override { x11Support = false; };

  windows = rec {
    sdl2Combined = pkgs.fetchzip {
      url = https://www.libsdl.org/release/SDL2-devel-2.0.9-mingw.tar.gz;
      sha256 = "1qw90044ri4w43bry98pycv0k24lkkxri9akl9alvrp08s2463wg";
    };

    sdl2 = {
      x64 =  "${sdl2Combined}/x86_64-w64-mingw32";
      x32 =  "${sdl2Combined}/i686-w64-mingw32";
    };
  };



  cargo_config = pkgs.writeText "config" ''
    [target.x86_64-unknown-linux-gnu]
    linker = "${pkgs.stdenv.cc}/bin/cc"
  '';

  basepkgs = pkgs.symlinkJoin {
    name = "basepkgs";
    paths = with pkgs; [
      coreutils
      binutils-unwrapped
      findutils
      diffutils
      gnused
      gnugrep
      gawk
      gnutar
      gzip
      bzip2.bin
      gnumake
      bash
      patch
      xz.bin
    ];
  };


  mkEnv = {
    paths,
    libs,
    linker,
  }: pkgs.writeScript "env.sh" ''
    #!${pkgs.stdenv.shell}
    export PATH="${lib.concatMapStringsSep ":" (x: x + "/bin/") (paths ++ [
      basepkgs
      pkgs.rustup
    ])}:$PATH"

    export RUSTFLAGS='${lib.concatMapStringsSep " " (x: "-L " + x + "/lib/") libs}'

    exec ${pkgs.bashInteractive}/bin/bash
  '';


  mkCC = p: pkgs.runCommand "env" { } ''
    mkdir -p $out/bin
    cd $out/bin
    ln -s ${p}/bin/*-cc cc
  '';

  mkLib = pkgs.runCommand "pt" {} ''
    mkdir -p $out/lib
    cd $out/lib
    ln -s ${builtins.fetchurl "ftp://sourceware.org/pub/pthreads-win32/dll-latest/lib/x64/libpthreadGC2.a"} libpthread.a
    ln -s ${builtins.fetchurl "ftp://sourceware.org/pub/pthreads-win32/dll-latest/dll/x64/pthreadGC2.dll"} libpthread.dll
  '';

  env = {
    musl = mkEnv {
      paths = [ pkgs.stdenv.cc pkgs.musl.dev ];
      libs = [ (sdl2 pkgs) ];
      linker = "musl-gcc";
    };

    simple = mkEnv {
      paths = [ pkgs.stdenv.cc pkgs.llvmPackages_latest.llvm ];
      libs = [ (sdl2 pkgs) ];
      linker = "cc";
    };

    win = let
      p = pkgs.pkgsCross.mingwW64;
    in mkEnv {
      paths = [ pkgs.stdenv.cc p.stdenv.cc ];
      libs = [ windows.sdl2.x64 ];
      linker = "x86_64-w64-mingw32-cc";
    };

    win32 = let
      p = pkgs.pkgsCross.mingw32;
    in mkEnv {
      paths = [ pkgs.stdenv.cc p.stdenv.cc ];
      libs = [ windows.sdl2.x32 ];
      linker = "i686-w64-mingw32-cc";
    };
  };

  cargo_win = let
    p = pkgs.pkgsCross.mingwW64;
  in pkgs.writeScript "env.sh" ''
    #!${pkgs.stdenv.shell}
    export PATH="${lib.concatMapStringsSep ":" (x: x + "/bin/") [
      p.stdenv.cc.cc
      p.musl.dev
      pkgs.coreutils
      pkgs.cargo
      pkgs.rustup
    ]}:$PATH"

    export RUSTFLAGS='-C linker=${p.stdenv.cc}/bin/x86_64-w64-mingw32-cc ${lib.concatMapStringsSep " " (x: "-L " + x + "/lib/") [
      p.stdenv.cc.cc
      windows.sdl2.x64
    ]}'

    export CC=x86_64-w64-mingw32-cc

    exec ${pkgs.bashInteractive}/bin/bash
  '';
}
