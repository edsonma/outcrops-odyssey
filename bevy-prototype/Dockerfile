# Bevy prototype -> WebAssembly -> static site.  Build context = repository ROOT:
#   docker build -f bevy-prototype/Dockerfile -t outcrops-bevy .
# (On Railway: second service, Settings -> Build -> Dockerfile path = bevy-prototype/Dockerfile)

# ---- stage 1: compile to wasm and generate the JS glue ----
FROM rust:1-slim AS build
RUN apt-get update && apt-get install -y --no-install-recommends binaryen ca-certificates \
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
RUN wasm-bindgen --target web --no-typescript --out-dir /out --out-name outcrops_bevy \
      target/wasm32-unknown-unknown/release/outcrops_bevy.wasm \
 && (wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext --enable-mutable-globals \
      -o /tmp/opt.wasm /out/outcrops_bevy_bg.wasm && mv /tmp/opt.wasm /out/outcrops_bevy_bg.wasm \
      || echo "wasm-opt skipped")

# ---- stage 2: tiny static server (same server.js as the Macroquad build) ----
FROM node:22-alpine
WORKDIR /app
COPY server.js package.json ./
COPY --from=build /out ./public
COPY bevy-prototype/web/index.html ./public/index.html
COPY bevy-prototype/assets ./public/assets
ENV NODE_ENV=production
CMD ["node", "server.js"]
