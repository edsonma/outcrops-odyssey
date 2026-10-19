# ---- stage 1: compile the game to WebAssembly ----
FROM rust:1-slim AS build
RUN rustup target add wasm32-unknown-unknown
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --target wasm32-unknown-unknown --locked
COPY web-template ./web-template
COPY assets ./assets
RUN mkdir -p /out/public \
 && cp web-template/index.html web-template/gl.js /out/public/ \
 && cp target/wasm32-unknown-unknown/release/outcrops_odyssey.wasm /out/public/ \
 && cp -r assets /out/public/assets

# ---- stage 2: tiny static server ----
FROM node:22-alpine
WORKDIR /app
COPY package.json server.js ./
COPY --from=build /out/public ./public
ENV NODE_ENV=production
CMD ["node", "server.js"]
