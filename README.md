# Outcrops Odyssey — "Geology Field" Tech Demo (DragonRuby)

## Player character redesign

The player sprite (`player.png` / `player_shadow.png`) was redrawn in a
bold-outline, flat cel-shaded illustration style (angular silhouette,
dramatic swept hair, clean black linework) based on a style reference the
person provided, then adapted into an original geologist character — not
a reproduction of the reference character's specific design — holding a
rock hammer and a field notebook. It's drawn at 60×80 (up from the
original 40×56) since the held tools need the extra width to read
clearly; `state.player.w/h` and the shadow sprite in `render_player` were
updated to match.

Note this is a deliberate style mix: a flat-vector illustrated character
walking across the photorealistic photo-textured ground from the realism
pass below. That contrast is intentional here (and is exactly how many
published games — including Don't Starve itself — pair a stylized
character against a more detailed world), but if a more unified look is
wanted later, the two directions would need to move toward each other.

## Realism upgrade (real CC0 photo textures)

The grass, rock outcrops, dirt patches, and tree canopy now use real
CC0-licensed photo textures (Poly Haven), composited into the same
silhouette shapes and lighting the earlier version painted by hand:

- **Grass** (`grass_0/1/2.png`): Poly Haven's Sparse Grass, whole-image
  resized (not cropped) so each variant stays seamlessly self-tileable,
  then color-graded for three distinguishable field patches
- **Outcrops raw/visited**: Poly Haven's Cliff Side (visible rock strata)
  and Rock Face 03, with a moss overlay from Aerial Rocks 04 blended into
  the "visited" variant — directional lighting and a dark outline are
  added on top so they still read clearly as objects against the ground
- **Dirt patches**: Poly Haven's Brown Mud Leaves 01
- **Tree canopy**: Poly Haven's Forest Leaves 02 (a leaf-litter/forest-floor
  texture, recolored greener) cut into canopy-blob shapes — this is the
  one asset where "photorealistic" is a stretch, noted below
- **Player, shadows, sparkle, card popup**: unchanged/procedural — there's
  no relevant CC0 photo source for a small stylized top-down character or
  UI cue, so these stay illustrated

**Honest limitation**: this is now a *hybrid* look — real photo ground/rock
textures next to an illustrated player character and canopy. True
photorealism across every asset would need photo-sourced (or AI-generated)
art for the character and UI elements too, which wasn't available for this
pass.

An infinite, procedurally-expanding open field, Don't-Starve style: the
world isn't pre-built, it's generated in fixed-size chunks around you as
you walk, and the same chunk always regenerates the same way if you come
back to it. This implements a working slice of the GDD's field-exploration
systems, not the full game.

`preview.png` is a composited frame built from the game's own sprites,
showing the player mid-field with scattered outcrops (sparkles mark
un-broken ones), dirt patches, trees, and the live HUD layout.

## What's implemented (from the GDD)

- **Infinite exploration**: free 2D movement, camera follows the player,
  the world expands forever in every direction (tested into negative
  coordinates too, not just up-and-to-the-right)
- **Deterministic chunks**: each chunk's content (outcrop count and
  positions, tree placement, terrain variant) is derived from its
  coordinates via a hash, not stored forever — so walking away and back
  gives you the exact same field, without needing to remember every
  chunk you've ever visited forever
- **Outcrop breaking**: walk up to an outcrop (raw ones sparkle) and
  press SPACE — awards 10 points + 20 XP the first time, and permanently
  marks it "visited" (it stays in the world, just no longer sparkling,
  and re-breaking it gives no repeat reward) — exactly the raw/visited
  behavior from the GDD
- **Drop rates**: the GDD's flat 5% independent chance each for a rare
  mineral, a fossil, and a bonus rock fragment on every first-time break
- **10-slot inventory cap** for bonus rock fragments
- **Day/night cycle**: a smooth lighting overlay cycling from full
  daylight to night and back (compressed to ~90 seconds per cycle here
  instead of the GDD's 5am-6pm pacing, so you can actually see it happen
  during a demo session)

## What's NOT in this demo (see the GDD for the full design)

- The truck/basecamp/extraction loop and the "miss sunset = game over"
  failure condition
- The Task List system
- XP leveling curve / Strength & Stamina progression
- Main menu (save/load/new game/leaderboard) and player naming
- True shared-world persistence across players — outcrop visited-state
  here lives only in memory for this play session and resets when the
  game restarts. The GDD calls for this to be global/shared across all
  players; that needs a real backend and is out of scope for a
  single-player tech demo.

## A DragonRuby-specific design choice worth knowing about

The chunk cache and the visited-outcrop set use **dynamic string keys**
(`"cx,cy"`, outcrop ids) and need full Hash behavior — `#key?`, `#values`,
arbitrary bracket assignment. `args.state` is built for fixed-schema,
dot-accessible game entities (like `state.player.x`), not general-purpose
dictionaries, so this demo keeps those two collections in plain Ruby
globals (`$chunks`, `$visited`) instead of inside `args.state`. Everything
else (player, points, XP, inventory count, popups) lives in `args.state`
as usual.

## Another DragonRuby gotcha this code works around: fixed-width integers

An earlier version of `chunk_hash` (the function that deterministically
generates each chunk's content from its coordinates) multiplied by
constants in the billions, the way you'd write a hash function in normal
Ruby without a second thought. It crashed immediately with `EXCEPTION:
integer overflow`.

The reason: mruby's `Integer` (which DragonRuby runs on) is **fixed-width**
and **raises on overflow**, unlike stock Ruby/MRI, where `Integer`
automatically promotes to arbitrary-precision `Bignum` and this class of
bug simply can't happen. `chunk_hash` now keeps every intermediate value
under ~1.05 billion by construction — proven by the arithmetic bounds, not
just "seemed fine when I tried it" — comfortably under the 32-bit signed
ceiling of 2,147,483,647, regardless of how large or far-flung the chunk
coordinates get.

**This also means my plain-Ruby smoke testing has a real blind spot**:
since regular Ruby's `Integer` never overflows, my test harness literally
cannot reproduce this bug — it only surfaced once you ran it in the real
engine. I added a regression test that checks the *bound* instead (that
no intermediate value ever gets within even 2x of the overflow ceiling,
across deliberately extreme coordinates), which is the best a Ruby-side
test can do for this class of issue. Anything else added to this file
that does arithmetic on values that could grow large should keep the same
discipline: work out the worst-case product on paper, not just "it ran
fine locally."

## Controls

- **WASD / Arrow keys** — walk
- **SPACE** — break the nearest outcrop (only works within range; a
  prompt appears when one's close enough)

## Running it

1. Download DragonRuby GTK from **https://dragonruby.org**.
2. Copy this project's `mygame/app` and `mygame/sprites` into a DragonRuby
   install's `mygame/` folder (or replace the whole folder).
3. Run it:
   - macOS/Linux: `./dragonruby mygame`
   - Windows: `dragonruby.exe mygame`

## How I verified this without a DragonRuby window

Same approach as the other prototypes in this project: I stubbed out
DragonRuby's `args`/`state`/`inputs` API in plain Ruby and ran the real
game code hundreds of times, checking:

- the world actually expands (new chunks appear) as the player walks, in
  **all four directions**, including negative coordinates
- calling the chunk generator twice with the same coordinates produces
  **identical** content (determinism, the whole point of a Don't
  Starve-style persistent world)
- different chunk coordinates produce **different** content (it's not
  secretly generating the same thing everywhere)
- breaking an outcrop awards points/XP and marks it visited; breaking the
  same one again awards nothing extra
- the 10-slot inventory cap holds even when drops are forced to always
  succeed (not just hoping random rolls happened to test it)
- the day/night cycle actually swings between near-0 and near-1
  (full day and full night)

That's real logic verification, not a guarantee about rendering or
DragonRuby-version-specific behavior — only running it in the actual
engine can confirm those.

## Natural next steps

- A real extraction/truck loop with the sunset deadline and its
  game-over failure state
- Persisting `$chunks`/`$visited` to disk (or a backend, for the GDD's
  shared-world requirement) instead of only living in memory
- Replacing the flat rectangular chunk grass tiles with actual terrain
  variety at the tile level (rivers, forests, farms per the GDD's world
  description) rather than only varying at the whole-chunk level
- Real collision so trees are obstacles instead of decoration
- The full card UI (the "Pokémon card" collection screen) triggered from
  breaking an outcrop, instead of a text popup
