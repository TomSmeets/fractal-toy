FROM common
RUN rustup target add x86_64-pc-windows-gnu

RUN apt-get install -y \
      mingw-w64        \
      wine-stable

ENV CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc \
    CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUNNER=wine                   \
    CC_x86_64_pc_windows_gnu=x86_64-w64-mingw32-gcc-posix            \
    CXX_x86_64_pc_windows_gnu=x86_64-w64-mingw32-g++-posix           \
