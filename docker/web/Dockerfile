FROM rust-build-common

WORKDIR /opt

RUN apt-get install -y python                      \
 && git clone 'https://github.com/juj/emsdk' emsdk \
 && cd emsdk                                       \
 && ./emsdk install latest                         \
 && ./emsdk activate latest

RUN rustup target add asmjs-unknown-emscripten
RUN cargo install cargo-web
