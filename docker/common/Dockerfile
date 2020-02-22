FROM ubuntu

RUN apt-get update && apt-get install -y \
   curl                  \
   build-essential cmake \
   git python3

RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly -y
ENV PATH=/root/.cargo/bin:$PATH
