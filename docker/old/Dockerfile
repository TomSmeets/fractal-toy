FROM ubuntu

RUN apt-get update
RUN apt-get install -y curl
RUN apt-get install -y build-essential cmake
RUN apt-get install -y git python3

# rustup
WORKDIR /root
RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly -y

ENV PATH=/root/.cargo/bin:$PATH

RUN rustup target add x86_64-unknown-linux-gnu
RUN rustup target add x86_64-unknown-linux-musl
RUN rustup target add i686-unknown-linux-gnu
RUN rustup target add i686-unknown-linux-musl
RUN rustup target add x86_64-pc-windows-gnu
RUN rustup target add i686-pc-windows-gnu
RUN rustup target add asmjs-unknown-emscripten

# EMSDK
WORKDIR /opt
RUN git clone 'https://github.com/juj/emsdk' emsdk
RUN cd emsdk \
    && ./emsdk install latest \
    && ./emsdk activate latest

WORKDIR /root

# tools
RUN apt-get install -y python vim

# Libs
RUN apt-get install -y libsdl2-dev

# windows
RUN apt-get install -y mingw-w64
RUN apt-get install -y wine-stable

RUN dpkg --add-architecture i386
RUN apt-get update
RUN apt-get install -y wine32

ENV CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc
ENV CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUNNER=wine
ENV CC_x86_64_pc_windows_gnu=x86_64-w64-mingw32-gcc-posix
ENV CXX_x86_64_pc_windows_gnu=x86_64-w64-mingw32-g++-posix

ENV CARGO_TARGET_i686_PC_WINDOWS_GNU_LINKER=i686-w64-mingw32-gcc
ENV CARGO_TARGET_i686_PC_WINDOWS_GNU_RUNNER=wine
ENV CC_i686_pc_windows_gnu=i686-w64-mingw32-gcc-posix
ENV CXX_i686_pc_windows_gnu=i686-w64-mingw32-g++-posix

RUN curl 'https://www.libsdl.org/release/SDL2-devel-2.0.10-mingw.tar.gz' | tar -xz

# COPY . .
# WORKDIR minimal
# RUN cargo build
# RUN cargo build --target x86_64-unknown-linux-gnu
# RUN cargo build --target x86_64-pc-windows-gnu
# RUN cargo build --target asmjs-unknown-emscripten

# RUN cargo build --target i686-unknown-linux-gnu
# RUN cargo build --target i686-pc-windows-gnu
