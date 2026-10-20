# Outcrops Odyssey

A geology open-world game: walk an endless field, break rock **outcrops**, and collect them as
**cards**. Each day starts at a basecamp, a truck drives you out along the main roads, and you must
be back on a road before sunset or the run is lost. Written in **Rust** with
[Macroquad](https://macroquad.rs), and playable **natively or in the browser** (WebAssembly),
on desktop and on phones.

The full design lives in `Outcrops_Odyssey_GDD.docx`.

## Play

| | Desktop | Phone / tablet (landscape) |
|---|---|---|
| Move | WASD / arrows, or click the ground | Floating stick (hold anywhere lower-left), or tap the ground |
| Break an outcrop | SPACE, or click it | Tap it (walks over first if far), or the **BREAK** button |
| Board / call the truck | **E** (or click the truck) | Tap the truck, or **BOARD** / **TRUCK** |
| Map / cards | **M** / **C** (Up/Down pages) | **MAP** / **CARDS** buttons |
| Restart after game over | ENTER | Tap |

The touch controls appear automatically on touch devices. The page blocks zoom and scrolling,
offers a fullscreen button, and asks portrait users to rotate. On iPhone Safari (no fullscreen
API) use Share → Add to Home Screen.

## The daily loop

1. Wake at the **basecamp**, walk to the truck parked on the road below camp, and board it.
2. The truck drives along the **main roads** (a world-wide grid) to today's random drop point.
3. Explore and break outcrops. Field time runs **05:00 → 18:00**; supplies arrive at **12:00**
   (+50 energy). Walking drains energy; at 0 you move at half speed.
4. Stand on a main road and call the truck, or be on a road at sunset (automatic pickup).
   **Miss the road at 18:00 and it's game over.**
5. The truck drives back, you sleep (energy restored), and the next day begins. Cards, points and
   visited outcrops carry over.

## What's implemented (from the GDD)

- **Infinite world**: generated in 640-unit chunks around you. Each chunk is derived from its
  coordinates by a hash, so coming back always gives the same field (works in all directions,
  including negative coordinates).
- **Outcrops**: the first break awards 10 points + 20 XP and marks the outcrop *visited* (it stays
  in the world; raw ones sparkle). Each first break rolls the GDD's flat **5%** independent chance
  for a rare mineral, a fossil and a bonus rock (+10 XP for mineral/fossil); bonus rocks fill a
  **10-slot inventory**.
- **Cards**: every break creates a named card (adjective + lithology) with mineral / fossil / rock
  badges, browsable in the card viewer.
- **Map**: every explored chunk, raw vs. visited outcrops, main roads, basecamp and you.
- **Basecamp, main roads and truck** with road-following truck routes, a field clock, noon supplies,
  energy, a "nearest road" compass and a sunset warning.
- Nothing spawns on roads or inside the camp.

## Not built yet

- Task list (and the "Task completed" animation)
- XP leveling with Strength / Stamina progression
- Main menu: player naming ("Dr Geologist: name"), save / load / new game, leaderboard
- Onboarding tutorial
- Shared world state across players (needs a backend; outcrop state is per session today)

## Project layout

```
src/world.rs      deterministic chunk generation, roads, basecamp, truck routing
src/game.rs       rules and state machine (Basecamp → TruckOut → Exploring → TruckBack → Sleeping / GameOver)
src/touch.rs      on-screen buttons and virtual stick (pure logic)
src/main.rs       window, fixed 60 Hz loop, input, drawing, letterboxed 1280×720 screen
assets/           sprites and fonts (DejaVu, free license)
web-template/     index.html + miniquad's gl.js loader for the browser build
server.js         zero-dependency static server used on Railway
Dockerfile        builds the wasm, then serves it with server.js
railway.json      Railway config (Dockerfile build, /healthz health check)
.github/workflows CI: cargo test + wasm build
bevy-prototype/   3D renderer prototype (Bevy) reusing src/game.rs + src/world.rs — see its README
project/          the original DragonRuby prototype, kept for reference only
```

`game.rs`, `world.rs` and `touch.rs` contain no rendering or window code, so the whole game loop
is covered by headless unit tests (`cargo test`, 34 tests): chunk determinism, drop rates,
inventory cap, truck routes staying on roads, sunset game over, the full day loop, tap
interactions and touch routing.

## Run it

**Natively**

```fish
cargo run --release
```

**In the browser, locally**

```fish
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown
mkdir -p public
cp web-template/* public/
cp target/wasm32-unknown-unknown/release/outcrops_odyssey.wasm public/
cp -r assets public/
node server.js   # http://localhost:3000
```

**Deploy to Railway**: New Project → Deploy from GitHub repo → pick this repo → Settings →
Networking → Generate Domain. Railway builds the `Dockerfile`; the health check is `/healthz`.

## Art credits

Grass, rock, dirt and canopy textures are composited from CC0 photo textures by
[Poly Haven](https://polyhaven.com). The geologist, shadows, sparkle and truck/camp shapes are
illustrated or drawn in code. The look is deliberately a hybrid of photo ground and a flat-vector
character. UI text uses DejaVu Sans.

## History

v0.1 was a DragonRuby (mruby) prototype; the game was ported to Rust in v0.3, keeping the same
chunk hash so worlds match. The prototype remains under `project/mygame`.
