# The 3D (Bevy) build -> WebAssembly -> static site. This is the default Railway build.
#   docker build -t outcrops-bevy .
# The 2D Macroquad build lives in Dockerfile.macroquad (Railway: Settings -> Build -> Dockerfile path).

# ---- stage 1: compile to wasm and generate the JS glue ----
FROM rust:1-slim AS build
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*
RUN rustup target add wasm32-unknown-unknown
# MUST equal the `wasm-bindgen` version in bevy-prototype/Cargo.lock
ARG WASM_BINDGEN_VERSION=0.2.129
RUN cargo install wasm-bindgen-cli --version ${WASM_BINDGEN_VERSION} --locked

WORKDIR /src
# shared game logic: game.rs / world.rs (+ touch.rs)
COPY src ./src
COPY bevy-prototype ./bevy-prototype
WORKDIR /src/bevy-prototype
# Set CARGO_FEATURES="--no-default-features --features webgpu" for the WebGPU build
ARG CARGO_FEATURES=""
RUN cargo build --release --locked --target wasm32-unknown-unknown ${CARGO_FEATURES}
# No wasm-opt here: the Debian binaryen is too old for the features rustc emits and strips the
# exported `__wasm_bindgen_func_elem_*` closures ("wasm.__wasm_bindgen_func_elem_N is not a function").
RUN wasm-bindgen --target web --no-typescript --out-dir /out --out-name outcrops_bevy \
      target/wasm32-unknown-unknown/release/outcrops_bevy.wasm

# ---- stage 2: tiny static server (same server.js as the Macroquad build) ----
FROM node:22-alpine
WORKDIR /app
COPY server.js package.json ./
COPY --from=build /out ./public
COPY bevy-prototype/web/index.html ./public/index.html
COPY bevy-prototype/assets ./public/assets
ENV NODE_ENV=production
CMD ["node", "server.js"]
