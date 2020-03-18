source /opt/emsdk/emsdk_env.sh
# cargo web start dos not work yet somehow
cargo web deploy --target=asmjs-unknown-emscripten --use-system-emscripten --release
cargo web deploy --target=wasm32-unknown-emscripten --use-system-emscripten --release
