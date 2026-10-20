# Outcrops Odyssey — Bevy 3D prototype

A 3D renderer for the *same game* as the Macroquad version: it reuses `../src/game.rs` and
`../src/world.rs` unchanged (included with `#[path]`), so rules, world generation, roads, the truck
loop and tap/click interaction are identical. Only drawing and input are new (`src/main.rs`).

Built on **Bevy 0.19** (0.20 needs rustc 1.97.1; switch whenever your toolchain allows).

## What it shows

| | |
|---|---|
| **Camera** | Perspective follow camera with smoothing. `Tilted` (default, ~50° down, the Macroquad view in 3D) and `Side` (low, side-on "2.5D"); press **V** to switch. Edit the two offsets in `follow_camera`. |
| **Lighting** | One `DirectionalLight` sun with 3-cascade shadows, driven by the game clock (05:00→18:00: rises east, sets west, warm at the horizon, fades at night), plus `GlobalAmbientLight`, `DistanceFog` and `TonyMcMapface` tonemapping. The campfire is a flickering `PointLight`. |
| **Materials** | Real PBR: Poly Haven (CC0) albedo + normal + roughness maps for grass, rock and moss, tiled with `uv_transform`; normal/roughness maps are loaded as *linear*, albedo as *sRGB*. Raw outcrops glow faintly, visited ones turn mossy. |
| **Assets** | Trees are a **glTF binary** (`assets/models/tree.glb#Scene0`, spawned with `WorldAssetRoot`). Replace it with any Blender export (`File → Export → glTF 2.0 → .glb`) and nothing else changes. Rocks are procedural displaced icospheres. |
| **World** | Chunks spawn/despawn around the camera focus; roads, dashes, dirt patches, camp, truck and player are built from the game state every frame. |

Scale: 1 game unit = 0.025 m, so a chunk is 16 m, the player ~2 m, a road 3 m wide.

## Run natively

```fish
cd bevy-prototype
cargo run --release          # first build compiles Bevy: ~10-40 min depending on cores
```

Controls: WASD/arrows, click/tap the ground, SPACE or click a rock to break it, **E** board / call
the truck, **V** camera. (On-screen touch buttons are not ported yet — `../src/touch.rs` is
engine-independent and can be reused; taps already work.)

## Build for the browser

```fish
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked    # must match Cargo.lock
cd bevy-prototype
cargo build --release --target wasm32-unknown-unknown        # WebGL2 (default)
# WebGPU instead (separate build, newer browsers only):
#   cargo build --release --target wasm32-unknown-unknown --no-default-features --features webgpu
wasm-bindgen --target web --no-typescript --out-dir ../public3d --out-name outcrops_bevy \
  target/wasm32-unknown-unknown/release/outcrops_bevy.wasm
cp web/index.html ../public3d/index.html
cp -r assets ../public3d/assets
cd .. ; set -x STATIC_DIR public3d ; node server.js          # http://localhost:3000
```

Optional size win: `wasm-opt -Oz` (binaryen) on `outcrops_bevy_bg.wasm`.

## Deploy to Railway (second service)

The Macroquad game keeps its service. Add another service from the **same repo** and set
*Settings → Build → Dockerfile path* to `bevy-prototype/Dockerfile` (root directory stays the
repo root; the build needs `../src`). The Docker build compiles the wasm, runs `wasm-bindgen`,
and serves it with the repo's `server.js`. Expect a long first build (Bevy + wasm-bindgen-cli).
For WebGPU, set the build arg `CARGO_FEATURES=--no-default-features --features webgpu`.

## Honest status

- Compiles and runs natively; rendering was checked here with a software Vulkan driver
  (screenshots of the camp, field, noon and sunset scenes).
- **The wasm/WebGL2 build has not been run**: the authoring sandbox cannot install the wasm
  target. CI (`.github/workflows/ci.yml`, job `bevy-wasm`) builds it on GitHub, but the
  in-browser look and performance still need a real browser and phone.
- Expect a **large download** (Bevy wasm is typically tens of MB before compression) — measure
  it on mobile data before deciding to migrate.
- WebGL2 limits apply (no compute-based effects); effects that need WebGPU should be treated as
  optional.

## Next steps if it wins the comparison

1. Environment lighting: bake the HDRI to a cubemap and add `EnvironmentMapLight` for natural reflections.
2. Real models from Blender (rocks, truck, cabins, geologist) as `.glb`; compress textures to KTX2.
3. Port the touch UI (stick + buttons) from `src/touch.rs` using Bevy UI.
4. Swap the primitive-built actors for animated glTF characters.
