# app/main.rb
#
# Outcrops Odyssey — "Geology Field" tech demo
# An infinite, procedurally-expanding open field, in the spirit of
# Don't Starve: the world isn't pre-built, it's generated in fixed-size
# chunks around the player as they walk, and the same chunk always
# regenerates the same way if you come back to it.
#
# This implements a working slice of the GDD's field-exploration systems:
#   - free 2D movement across an unbounded world (camera follows player)
#   - outcrops scattered across the field; walk up and press SPACE to
#     break one and collect its "card" (points + XP)
#   - rare mineral / fossil / bonus-rock drops at the GDD's flat 5% each
#   - a 10-slot inventory cap for bonus rock fragments
#   - a day/night lighting cycle (compressed to a couple of minutes for a
#     demo instead of the GDD's 5am-6pm/real-day pacing)
#
# NOT implemented here (out of scope for a tech demo -- see README):
#   truck/basecamp/extraction loop, task list, leveling curve, main menu,
#   multiplayer/shared-world persistence (outcrop state here is local to
#   this play session only, kept in memory).
#
# TWO DRAGONRUBY GOTCHAS THIS FILE DELIBERATELY WORKS AROUND
#   1) `rand` (mruby) does not accept a Range like normal Ruby's
#      `rand(1..10)`. Use `Numeric.rand(range)` instead -- used
#      throughout below.
#   2) Chaining `||=` on a freshly-accessed nested state field doesn't
#      reliably work the way it looks like it should. State is
#      initialized by assigning a whole Hash in one shot
#      (`state.thing ||= {...}`) rather than field-by-field.
#
# COORDINATES: DragonRuby's origin is bottom-left, y increases upward.
# The WORLD uses the same convention as the screen (no separate world
# space flip needed) -- only the camera offset changes what part of the
# infinite world lands on screen.

class Game
  attr_gtk

  SCREEN_W = 1280
  SCREEN_H = 720
  CHUNK = 640            # world units per chunk (matches the grass tile size)
  VIEW_RADIUS_CHUNKS = 2 # how many chunks out from the player's chunk to draw
  PLAYER_SPEED = 4.2
  BREAK_RANGE = 60
  DAY_LENGTH_TICKS = 5400 # ~90 seconds at 60fps for a full day/night cycle in this demo

  def tick
    defaults
    input
    calc
    render
  end

  # ---------------------------------------------------------------
  # DEFAULTS
  # ---------------------------------------------------------------
  def defaults
    state.player ||= { x: 0, y: 0, w: 60, h: 80, facing: 1 }
    state.outcrop_points ||= 0
    state.xp ||= 10
    state.inventory_count ||= 0
    state.popups ||= []          # floating "+N" / card-found feedback, an
                                  # Array of Hashes -- safe in args.state
    state.tick_count ||= 0

    # The chunk cache and visited-set use dynamic string keys ("cx,cy",
    # outcrop ids) and need full Hash behavior (#key?, #values, bracket
    # assignment for arbitrary keys). args.state is designed for
    # fixed-schema, dot-accessible game entities, not general-purpose
    # dictionaries, so these two live in plain Ruby globals instead --
    # deliberately outside of args.state.
    $chunks  ||= {}
    $visited ||= {}
  end

  # ---------------------------------------------------------------
  # INPUT
  # ---------------------------------------------------------------
  def input
    p = state.player
    dx = 0
    dy = 0
    dx -= 1 if inputs.keyboard.left  || inputs.keyboard.a
    dx += 1 if inputs.keyboard.right || inputs.keyboard.d
    dy -= 1 if inputs.keyboard.down  || inputs.keyboard.s
    dy += 1 if inputs.keyboard.up    || inputs.keyboard.w

    if dx != 0 || dy != 0
      mag = Math.sqrt((dx * dx) + (dy * dy))
      p.x += (dx / mag) * PLAYER_SPEED
      p.y += (dy / mag) * PLAYER_SPEED
      p.facing = dx < 0 ? -1 : 1 if dx != 0
    end

    break_nearest_outcrop if inputs.keyboard.key_down.space
  end

  # ---------------------------------------------------------------
  # CALC
  # ---------------------------------------------------------------
  def calc
    state.tick_count += 1
    ensure_nearby_chunks_generated
    calc_popups
  end

  def calc_popups
    state.popups.each do |pop|
      pop[:y] += 0.6
      pop[:life] -= 1
    end
    state.popups.reject! { |pop| pop[:life] <= 0 }
  end

  # ---------------------------------------------------------------
  # WORLD GENERATION (deterministic -- same chunk coords always produce
  # the same content, without needing to remember every chunk forever)
  # ---------------------------------------------------------------

  # A small, dependency-free integer hash so we never rely on Ruby's
  # Random class being seedable the same way across engine versions --
  # this is plain arithmetic, guaranteed portable.
  # mruby's Integer is fixed-width (unlike stock Ruby's arbitrary-precision
  # Integer/Bignum) and RAISES on overflow instead of silently promoting
  # to a bigger type. An earlier version of this hash multiplied by
  # constants in the billions, which overflowed almost immediately.
  # This version keeps every intermediate value under ~1.05e9 BY
  # CONSTRUCTION -- comfortably under the 32-bit signed ceiling of
  # 2,147,483,647 -- so it can't hit this bug regardless of how large
  # cx/cy/salt get, not just "probably fine" based on testing (plain
  # Ruby -- which this was smoke-tested under -- can't even reproduce
  # this bug, since its Integer doesn't have a fixed width; the fix
  # here is a width *proof*, not an empirical patch).
  HASH_MOD  = 104_729 # a prime, ~1e5
  HASH_MUL  = 9_973    # a prime, ~1e4 -- HASH_MOD * HASH_MUL ~= 1.04e9
  HASH_MUL2 = 7_919     # a different prime, used for the last mixing step

  def chunk_hash(cx, cy, salt)
    cxm = cx % 46_340
    cym = cy % 46_340
    saltm = salt % 46_340
    h = 17
    h = ((h * HASH_MUL) + cxm) % HASH_MOD
    h = ((h * HASH_MUL) + cym) % HASH_MOD
    h = ((h * HASH_MUL2) + saltm) % HASH_MOD
    h
  end

  def chunk_rand_int(cx, cy, salt, mod)
    chunk_hash(cx, cy, salt) % mod
  end

  def chunk_key(cx, cy)
    "#{cx},#{cy}"
  end

  def world_to_chunk(world_x, world_y)
    [(world_x / CHUNK).floor, (world_y / CHUNK).floor]
  end

  def ensure_nearby_chunks_generated
    pcx, pcy = world_to_chunk(state.player.x, state.player.y)
    (-VIEW_RADIUS_CHUNKS..VIEW_RADIUS_CHUNKS).each do |dx|
      (-VIEW_RADIUS_CHUNKS..VIEW_RADIUS_CHUNKS).each do |dy|
        cx = pcx + dx
        cy = pcy + dy
        key = chunk_key(cx, cy)
        next if $chunks.key?(key)

        $chunks[key] = generate_chunk(cx, cy)
      end
    end
  end

  # Generates the outcrops (and their fixed positions) for one chunk. Only
  # ever called once per chunk key per session -- after that the chunk's
  # content (and each outcrop's visited/raw state) is read from
  # $chunks / $visited, so walking away and back gives you the
  # same field you left, exactly like Don't Starve's chunk persistence.
  def generate_chunk(cx, cy)
    grass_variant = chunk_rand_int(cx, cy, 1, 3)
    outcrop_count = chunk_rand_int(cx, cy, 2, 4) # 0..3 outcrops per chunk
    tree_count = chunk_rand_int(cx, cy, 3, 3)    # 0..2 decorative trees

    outcrops = []
    outcrop_count.times do |i|
      local_x = chunk_rand_int(cx, cy, 10 + i, CHUNK - 40) + 20
      local_y = chunk_rand_int(cx, cy, 20 + i, CHUNK - 40) + 20
      outcrops << {
        id: "#{cx}_#{cy}_#{i}",
        x: (cx * CHUNK) + local_x,
        y: (cy * CHUNK) + local_y,
        lithology: %w[Igneous Sedimentary Metamorphic][chunk_rand_int(cx, cy, 40 + i, 3)]
      }
    end

    trees = []
    tree_count.times do |i|
      local_x = chunk_rand_int(cx, cy, 60 + i, CHUNK - 40) + 20
      local_y = chunk_rand_int(cx, cy, 70 + i, CHUNK - 40) + 20
      trees << { x: (cx * CHUNK) + local_x, y: (cy * CHUNK) + local_y }
    end

    dirt_patches = []
    if chunk_rand_int(cx, cy, 90, 100) < 35
      dirt_patches << {
        x: (cx * CHUNK) + chunk_rand_int(cx, cy, 91, CHUNK - 180) + 90,
        y: (cy * CHUNK) + chunk_rand_int(cx, cy, 92, CHUNK - 130) + 65
      }
    end

    { grass_variant: grass_variant, outcrops: outcrops, trees: trees, dirt_patches: dirt_patches }
  end

  # ---------------------------------------------------------------
  # OUTCROP INTERACTION
  # ---------------------------------------------------------------
  def nearest_outcrop
    p = state.player
    pcx, pcy = world_to_chunk(p.x, p.y)
    best = nil
    best_dist = nil

    (-1..1).each do |dx|
      (-1..1).each do |dy|
        chunk = $chunks[chunk_key(pcx + dx, pcy + dy)]
        next unless chunk

        chunk[:outcrops].each do |o|
          dist = Math.sqrt(((o[:x] - p.x)**2) + ((o[:y] - p.y)**2))
          if best_dist.nil? || dist < best_dist
            best = o
            best_dist = dist
          end
        end
      end
    end

    return nil if best.nil? || best_dist > BREAK_RANGE

    best
  end

  def break_nearest_outcrop
    o = nearest_outcrop
    return unless o

    already_visited = $visited[o[:id]]
    $visited[o[:id]] = true

    if already_visited
      spawn_popup(o[:x], o[:y], 'Already surveyed', 130, 130, 130)
      return
    end

    state.outcrop_points += 10 # card point value (placeholder scoring)
    state.xp += 20
    spawn_popup(o[:x], o[:y] + 20, "#{o[:lithology]} card! +10 pts / +20 xp", 255, 255, 255)

    # GDD-specified flat 5% independent drop chance per category
    if Numeric.rand(100) < 5
      state.xp += 10
      spawn_popup(o[:x], o[:y] + 46, 'Rare mineral found! +10 xp', 240, 210, 90)
    end
    if Numeric.rand(100) < 5
      state.xp += 10
      spawn_popup(o[:x], o[:y] + 72, 'Fossil found! +10 xp', 190, 220, 255)
    end
    if Numeric.rand(100) < 5 && state.inventory_count < 10
      state.inventory_count += 1
      spawn_popup(o[:x], o[:y] + 98, 'Bonus rock collected', 200, 255, 200)
    end
  end

  def spawn_popup(x, y, text, r, g, b)
    state.popups << { x: x, y: y, text: text, life: 70, r: r, g: g, b: b }
  end

  # ---------------------------------------------------------------
  # DAY / NIGHT
  # ---------------------------------------------------------------
  # Returns 0.0 (full daylight) .. 1.0 (deepest night), following a smooth
  # sine cycle rather than a literal 5am-6pm clock, to keep the demo's
  # cycle short enough to actually see.
  def night_amount
    phase = (state.tick_count % DAY_LENGTH_TICKS) / DAY_LENGTH_TICKS.to_f
    (Math.sin(phase * Math::PI * 2 - (Math::PI / 2)) + 1) / 2.0
  end

  # ---------------------------------------------------------------
  # RENDER
  # ---------------------------------------------------------------
  def render
    outputs.background_color = [20, 26, 18]
    render_world
    render_player
    render_popups
    render_night_overlay
    render_ui
  end

  def render_world
    p = state.player
    cam_x = p.x - SCREEN_W / 2
    cam_y = p.y - SCREEN_H / 2
    pcx, pcy = world_to_chunk(p.x, p.y)

    (-VIEW_RADIUS_CHUNKS..VIEW_RADIUS_CHUNKS).each do |dx|
      (-VIEW_RADIUS_CHUNKS..VIEW_RADIUS_CHUNKS).each do |dy|
        cx = pcx + dx
        cy = pcy + dy
        chunk = $chunks[chunk_key(cx, cy)]
        next unless chunk

        sx = (cx * CHUNK) - cam_x
        sy = (cy * CHUNK) - cam_y
        next if sx > SCREEN_W || sy > SCREEN_H || sx < -CHUNK || sy < -CHUNK

        outputs.sprites << {
          x: sx, y: sy, w: CHUNK, h: CHUNK,
          path: "sprites/grass_#{chunk[:grass_variant]}.png"
        }

        chunk[:dirt_patches].each do |dp|
          outputs.sprites << { x: dp[:x] - cam_x - 90, y: dp[:y] - cam_y - 65, w: 180, h: 130, path: 'sprites/dirt_patch.png' }
        end

        chunk[:trees].each do |t|
          outputs.sprites << { x: t[:x] - cam_x - 60, y: t[:y] - cam_y - 20, w: 120, h: 46, path: 'sprites/tree_shadow.png' }
          outputs.sprites << { x: t[:x] - cam_x - 70, y: t[:y] - cam_y - 10, w: 140, h: 160, path: 'sprites/tree.png' }
        end

        chunk[:outcrops].each do |o|
          visited = $visited[o[:id]]
          path = visited ? 'sprites/outcrop_visited.png' : 'sprites/outcrop_raw.png'
          ox = o[:x] - cam_x - 36
          oy = o[:y] - cam_y - 28
          outputs.sprites << { x: ox, y: oy, w: 72, h: 56, path: path }
          unless visited
            sparkle_bob = Math.sin((state.tick_count / 20.0) + o[:x]) * 3
            outputs.sprites << { x: ox + 22, y: oy + 44 + sparkle_bob, w: 28, h: 28, path: 'sprites/sparkle.png', a: 200 }
          end
        end
      end
    end
  end

  def render_player
    p = state.player
    sx = SCREEN_W / 2 - p.w / 2
    sy = SCREEN_H / 2 - p.h / 2
    outputs.sprites << { x: sx - 10, y: sy - 26, w: p.w + 20, h: 22, path: 'sprites/player_shadow.png' }
    outputs.sprites << {
      x: sx, y: sy, w: p.w, h: p.h, path: 'sprites/player.png',
      flip_horizontally: p.facing < 0
    }

    o = nearest_outcrop
    return unless o

    outputs.labels << {
      x: SCREEN_W / 2, y: 120, text: 'SPACE to break outcrop',
      size_enum: 1, alignment_enum: 1, r: 255, g: 255, b: 255, a: 220
    }
  end

  def render_popups
    state.popups.each do |pop|
      p = state.player
      cam_x = p.x - SCREEN_W / 2
      cam_y = p.y - SCREEN_H / 2
      alpha = (pop[:life] * 255 / 70).clamp(0, 255)
      outputs.labels << {
        x: pop[:x] - cam_x, y: pop[:y] - cam_y, text: pop[:text],
        size_enum: 0, alignment_enum: 1,
        r: pop[:r], g: pop[:g], b: pop[:b], a: alpha
      }
    end
  end

  def render_night_overlay
    n = night_amount
    return if n < 0.05

    outputs.solids << { x: 0, y: 0, w: SCREEN_W, h: SCREEN_H, r: 15, g: 20, b: 55, a: (n * 165).to_i }
  end

  def render_ui
    outputs.labels << {
      x: 24, y: SCREEN_H - 18, text: "Outcrop Points: #{state.outcrop_points}",
      size_enum: 2, r: 255, g: 255, b: 255, a: 230
    }
    outputs.labels << {
      x: 24, y: SCREEN_H - 42, text: "XP: #{state.xp}   Inventory: #{state.inventory_count}/10",
      size_enum: 0, r: 230, g: 230, b: 230, a: 220
    }
    outputs.labels << {
      x: 24, y: SCREEN_H - 64, text: "#{(night_amount * 100).to_i}% night",
      size_enum: -1, r: 200, g: 200, b: 220, a: 200
    }
    outputs.labels << {
      x: 24, y: 40, text: 'WASD/Arrows to walk  .  SPACE to break an outcrop',
      size_enum: -1, r: 220, g: 220, b: 220, a: 180
    }
  end
end

def tick(args)
  $game ||= Game.new
  $game.args = args
  $game.tick
end
