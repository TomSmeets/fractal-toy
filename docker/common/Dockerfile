FROM ubuntu

RUN apt-get update && apt-get install -y \
   curl                  \
   build-essential cmake \
   git python3           \
   fonts-dejavu          \
   m4 libsndio-dev

RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly -y

ENV PATH=/root/.cargo/bin:$PATH \
    FONT_DEJAVU=/usr/share/fonts/truetype/dejavu/
