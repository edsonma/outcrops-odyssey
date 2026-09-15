module Main
  def start
    @game = Game.new
  end

  def tick
    @game.inputs = inputs
    @game.tick
    outputs.primitives << @game.primitives
  end
end

class Game
  attr :inputs

  def initialize
    @game_over_render_queue = []
  end

  def tick
    new_game if !@clock || @game_over == true

    # calculate the new values of the light meter
    # (if the light meter hits zero, it's game over)
    calc_light_meter

    # capture the actions that were taken this turn so
    # that they can be "replayed" for the enemies on future
    # ticks of the simulation
    calc_action_history

    # calculate collisions for the player
    calc_entity @player

    # calculate collisions for the enemies
    calc_shadows

    # spawn a new light crystal
    calc_light_crystal

    # process "fire and forget" render queues
    # (eg particles and death animations)
    calc_render_queues

    # determine game over
    calc_game_over

    # increment the internal clocks for all entities
    # this internal clock is used to determine how
    # a player's past input is replayed. it's also
    # used to determine what animation frame the entity
    # should be performing when idle, running, and jumping
    calc_clock
  end

  def primitives
    [
      stage_primitives,
      light_meter_primitives,
      instruction_primitives,
      win_primitives,
      render_queue_primitives,
      light_meter_warning_primitives,
      light_crystal_primitives,
      entities_primitives,
    ]
  end

  # this is the input_entity function that handles
  # the movement of the player (and the enemies)
  # it's essentially your state machine for player
  # movement
  def input_entity entity, left_right, jump, fall_through
    # guard clause that ignores input processing if
    # the entity is still spawning
    return if !entity_active? entity

    # increment the dx of the entity by the magnitude of
    # the left_right input value
    entity.dx += left_right

    # if the left_right input is zero...
    if left_right == 0 && entity.action == :running
      # if the entity was originally running, then
      # set their "action" to standing
      # entity_set_action! updates the current action
      # of the entity and takes note of the frame that
      # the action occurred on
      entity_set_action! entity, :standing
    elsif entity.left_right != left_right && (entity_on_platform? entity)
      # if the entity is on a platform, and their current
      # left right value is different, mark them as running
      # this is done because we want to reset the run animation
      # if they changed directions
      entity_set_action! entity, :running
    end

    # capture the left_right input so that it can be
    # consulted on the next frame
    entity.left_right = left_right

    # capture the direction the player is facing
    # (this is used to determine the horizontal flip of the
    # sprite
    entity.facing = case left_right
                    when -1
                      :left
                    when 1
                      :right
                    else
                      entity.facing
                    end

    # if the fall_through (down) input was requested,
    # and if they are on a platform...
    if fall_through && (entity_on_platform? entity)
      entity.jumped_at      = 0
      # set their jump_down value (falling through a platform)
      entity.jumped_down_at = entity.clock
      # and increment the number of times they jumped
      # (entities get three jumps before needing to touch the ground again)
      entity.jump_count    += 1
    end

    # if the jump input was requested
    # and if they haven't reached their jump limit
    if jump && entity.jump_count < 3
      # update the player's current action to the
      # corresponding jump number (used for rendering
      # the different jump animations)
      if entity.jump_count == 0
        entity_set_action! entity, :first_jump
      elsif entity.jump_count == 1
        entity_set_action! entity, :midair_jump
      elsif entity.jump_count == 2
        entity_set_action! entity, :midair_jump
      end

      # set the entity's dy value and take note
      # of when jump occurred (also increment jump
      # count/eat one of their jumps)
      entity.dy             = entity.jump_power
      entity.jumped_at      = entity.clock
      entity.jumped_down_at = 0
      entity.jump_count    += 1
    end
  end

  # ease the light meters value up or down
  # every time the player captures a light crystal
  # the "target" light meter value is increased and
  # slowly spills over to the final light meter value
  # which is used to determine game over
  def calc_light_meter
    @light_meter -= 1
    d = @light_meter_target * 0.1
    @light_meter += d
    @light_meter_target -= d
  end

  def calc_action_history
    # keep track of the inputs the player has performed over time
    # as the inputs change for the player, mark the point in time
    # the specific input changed, and when the change occurred.
    # when enemies replay the player's actions, this history (along
    # with the enemy's interal clock) is consulted to determine
    # what action should be performed

    # the three possible input events are captured and marked
    # within the input timeline if/when the value changes

    # left right input events
    @curr_left_right     = inputs.left_right
    if @prev_left_right != @curr_left_right
      @input_timeline.push_front({ at: @clock, k: :left_right, v: @curr_left_right })
    end
    @prev_left_right = @curr_left_right

    # jump input events
    @curr_jump     = inputs.keyboard.key_down.space    ||
                     inputs.keyboard.key_down.up       ||
                     inputs.keyboard.key_down.k        ||
                     inputs.controller_one.key_down.a  ||
                     inputs.controller_one.key_down.b
    if @prev_jump != @curr_jump
      @input_timeline.unshift({ at: @clock, k: :space, v: @curr_jump })
    end
    @prev_jump = @curr_jump

    # jump down (fall through platform)
    @curr_down    = inputs.keyboard.down || inputs.keyboard.j || inputs.controller_one.down
    if @prev_down != @curr_down
      @input_timeline.unshift({ at: @clock, k: :down, v: @curr_down })
    end
    @prev_down = @curr_down
  end

  def calc_entity entity
    # process entity collision/simulation
    calc_entity_rect entity

    # return if the entity is still spawning
    return if !entity_active? entity

    # the first tick an entity is simulated is the tick it "wakes up" on.
    # entities that spawn in (ie the shadows, which have an activate_at)
    # announce themselves with a burst of light. the player has no
    # activate_at and is simply present from the start, so it gets no burst.
    if !entity.activated
      entity.activated = true

      if entity.activate_at
        @jitter_fade_out_render_queue << { x: entity.x + 5 * rand,
                                           y: entity.y + 5 * rand,
                                           w: 86 + 5 * rand, h: 86 + 5 * rand,
                                           path: "sprites/light.png",
                                           g: 0, b: 0, a: 255 }
      end
    end

    # calc collisions
    calc_entity_collision entity

    # update the state machine of the entity based on the
    # collision results
    calc_entity_action entity

    # calc actions the entity should take based on
    # input timeline
    calc_entity_movement entity

    input_entity entity,
                 find_input_timeline(at: entity.clock, key: :left_right),
                 find_input_timeline(at: entity.clock, key: :space),
                 find_input_timeline(at: entity.clock, key: :down)
  end

  def calc_entity_rect entity
    # this function calculates the entity's new
    # collision rect, render rect, hurt box, etc
    entity.render_rect = { x: entity.x, y: entity.y, w: entity.w, h: entity.h }
    entity.rect = entity.render_rect.merge x: entity.render_rect.x + entity.render_rect.w * 0.33,
                                           w: entity.render_rect.w * 0.33
    entity.next_rect = entity.rect.merge x: entity.x + entity.dx,
                                         y: entity.y + entity.dy
    entity.prev_rect = entity.rect.merge x: entity.x - entity.dx,
                                         y: entity.y - entity.dy
    orientation_shift = 0

    if entity.facing == :right
      orientation_shift = entity.rect.w  / 2
    end

    entity.hurt_rect  = entity.rect.merge y: entity.rect.y + entity.h * 0.33,
                                          x: entity.rect.x - (entity.rect.w / 2) + orientation_shift,
                                          h: entity.rect.h * 0.33
  end

  def calc_entity_collision entity
    # left right boundary collision via clamp
    entity.x = entity.x.clamp(8 - 32, entity.x)
    entity.x = entity.x.clamp(entity.x, 1280 - 8 - entity.rect.w - 32)
    calc_entity_below entity
  end

  def calc_entity_below entity
    # exit ground collision detection if they aren't falling
    return unless entity.dy < 0

    collision = @tiles.find_all { |t| t.rect.top <= entity.prev_rect.y }
                      .find { |t| t.rect.intersect_rect? entity.rect.merge(y: entity.next_rect.y) }

    # exit ground collision detection if no ground was found
    return unless collision

    # determine if the entity is allowed to fall through the platform
    # (you can only fall through a platform if you've been standing on it for 8 frames)
    can_drop = if entity.last_standing_at && (entity.clock - entity.last_standing_at) < 8
                 false
               else
                 true
               end

    # if the entity is allowed to fall through the platform,
    # and the entity requested the action, then clip them through the platform
    if can_drop && entity.jumped_down_at.elapsed_time(entity.clock) < 10 && !collision.impassable
      if (entity_on_platform? entity) && can_drop
        entity.dy = -1
      end

      entity.jump_count = 1
    else
      entity.y  = collision.rect.y + collision.rect.h
      entity.dy = 0
      entity.jump_count = 0
    end
  end

  def calc_entity_action entity
    # update the state machine of the entity
    # based on where they ended up after physics calculations
    if entity.dy < 0
      # mark the entity as falling after the jump animation frames
      # have been processed
      if entity.action == :midair_jump
        if entity_action_complete? entity, @midair_jump_duration
          entity_set_action! entity, :falling
        end
      else
        entity_set_action! entity, :falling
      end
    elsif entity.dy == 0 && !(entity_on_platform? entity)
      # if the entity's dy is zero, determine if they should
      # be marked as standing or running
      if entity.left_right == 0
        entity_set_action! entity, :standing
      else
        entity_set_action! entity, :running
      end
    end
  end

  def calc_entity_movement entity
    # increment x and y positions of the entity
    # based on dy and dx
    calc_entity_dy entity
    calc_entity_dx entity
  end

  def calc_entity_dx entity
    # horizontal movement application and friction
    entity.dx  = entity.dx.clamp(-5,  5)
    entity.dx *= 0.9
    entity.x  += entity.dx
  end

  def calc_entity_dy entity
    # vertical movement application and gravity
    entity.y  += entity.dy
    entity.dy += @gravity
    entity.dy += entity.dy * @drag ** 2 * -1
  end

  def calc_shadows
    # every 5 seconds, add a new shadow enemy/increase difficult
    add_shadow! if @clock.zmod?(300)

    # for each shadow, perform a simulation calculation
    @shadows.each do |shadow|
      calc_entity shadow

      # decrement the spawn countdown which is used to determine if
      # the enemy is finally active
      shadow.spawn_countdown -= 1 if shadow.spawn_countdown > 0
    end
  end

  def calc_light_crystal
    # determine if the player has intersected with a light crystal
    light_rect = @light_crystal
    if @player.hurt_rect.intersect_rect? light_rect
      # if they have then queue up the partical animation of the
      # light crystal being collected
      @jitter_fade_out_render_queue << { x:    @light_crystal.x,
                                         y:    @light_crystal.y,
                                         w:    @light_crystal.w,
                                         h:    @light_crystal.h,
                                         a:    255,
                                         path: 'sprites/light.png' }

      # increment the light meter target value
      @light_meter_target += 600

      # spawn a new light cristal for the player to try to get
      @light_crystal = new_light_crystal
    end
  end

  def calc_render_queues
    # render all the entries in the "fire and forget" render queues
    @jitter_fade_out_render_queue.each do |s|
      new_w = s.w * 1.02 ** 5
      ds = new_w - s.w
      s.w = new_w
      s.h = new_w
      s.x -= ds / 2
      s.y -= ds / 2
      s.a = s.a * 0.97 ** 5
    end

    @jitter_fade_out_render_queue.reject! { |s| s.a <= 1 }

    @game_over_render_queue.each { |s| s.a = s.a * 0.95 }
    @game_over_render_queue.reject! { |s| s.a <= 1 }
  end

  def calc_game_over
    # calcuate game over
    @game_over = false
    @you_win = false if @clock == 300

    # it's game over if the player intersects with any of the enemies
    @game_over ||= @shadows.find_all { |s| s.spawn_countdown <= 0 }
                           .any? { |s| s.hurt_rect.intersect_rect? @player.hurt_rect }

    # it's game over if the light_meter hits 0
    @game_over ||= @light_meter <= 1

    # update game over states and win/loss
    if @game_over
      @you_win = false
      @game_over = true
    end

    if @light_meter >= 6000
      @you_win = true
      @game_over = true
    end

    # if it's a game over, fade out all current entities in play
    if @game_over
      @game_over_render_queue.concat @shadows.map { |s| { **entity_primitives(s), a: 255 } }
      @game_over_render_queue << { **entity_primitives(@player), a: 255 }
      @game_over_render_queue << @light_crystal.merge(a: 255, path: 'sprites/light.png', b: 128)
    end
  end

  def calc_clock
    return if @game_over
    @clock += 1
    @player.clock += 1
    @shadows.each { |s| s.clock += 1 if entity_active? s }
  end

  def stage_primitives
    { x: 0,
      y: 0,
      w: 1280,
      h: 720,
      path: "sprites/stage.png",
      a: 200 }
  end

  def light_meter_primitives
    # the light meter sprite is rendered across the top
    # how much of the light meter is light vs dark is based off
    # of what the current light meter value is (which increases
    # when a crystal is collected and decreses a little bit every
    # frame
    meter_perc = @light_meter.fdiv(6000)
    light_w = (1280 * meter_perc)
    dark_w  = 1280 - light_w

    # once the light and dark partitions have been computed
    # render the meter sprite and clip its width (source_w)
    [
      { x: 0,
        y: 720 - 64,
        w: light_w,
        source_x: 0,
        source_y: 0,
        source_w: light_w,
        source_h: 128,
        h: 64,
        path: 'sprites/meter-light.png' },
      { x: 1280 * meter_perc,
        y: 720 - 64,
        w: dark_w,
        source_x: light_w,
        source_y: 0,
        source_w: dark_w,
        source_h: 128,
        h: 64,
        path: 'sprites/meter-dark.png' }
    ]
  end

  def instruction_primitives
    [
      { x: 640,
        y: 40,
        text: '[left/right] to move, [up/space] to jump, [down] to drop through platform',
        anchor_x: 0.5 },
    ]
  end

  def win_primitives
    return if !@you_win

    { x: 640,
      y: 720 - 40,
      text: 'You win!',
      alignment_enum: 0.5 }
  end

  def render_queue_primitives
    [
      @jitter_fade_out_render_queue,
      @game_over_render_queue
    ]
  end

  def light_meter_warning_primitives
    return if @light_meter >= 255

    # the screen starts to dim if they are close to having
    # a game over because of a depleated light meter
    [
      { x: 0,
        y: 0,
        w: 1280,
        h: 720,
        a: 255 - @light_meter,
        path: :solid,
        r: 0,
        g: 0,
        b: 0 },

      { x: @light_crystal.x - 32,
        y: @light_crystal.y - 32,
        w: 128,
        h: 128,
        a: 255 - @light_meter,
        path: 'sprites/spotlight.png' }
    ]
  end

  def light_crystal_primitives
    { x: @light_crystal.x + 5 * rand,
      y: @light_crystal.y + 5 * rand,
      w: @light_crystal.w + 5 * rand,
      h: @light_crystal.h + 5 * rand,
      path: 'sprites/light.png' }
  end

  def entities_primitives
    [
      entity_primitives(@player, r: 0, g: 0, b: 0),
      @shadows.map { |shadow| entity_primitives shadow, g: 0, b: 0 }
    ]
  end

  def entity_primitives entity, r: 255, g: 255, b: 255;
    # this is essentially the entity "prefab"
    # the current action of the entity is consulted to
    # determine what sprite should be rendered
    # the action_at time is consulted to determine which frame
    # of the sprite animation should be presented
    a = 255

    # an entity that hasn't spawned in yet is rendered as a
    # flickering point of light instead of its sprite
    if entity.activate_at && entity.activate_at > @clock
      return { x: entity.x + 5 * rand,
               y: entity.y + 5 * rand,
               w: 64 + 5 * rand,
               h: 64 + 5 * rand,
               path: "sprites/light.png",
               g: 0, b: 0,
               a: 255 }
    end

    sprint_index = Numeric.frame_index start_at: entity.action_at,
                                       tick_count: entity.clock,
                                       **action_lookup[entity.action]

    path = "sprites/player/#{entity.action}-#{sprint_index}.png"
    entity.render_rect.merge path: path,
                             a: a,
                             r: r,
                             g: g,
                             b: b,
                             flip_horizontally: entity.facing == :left
  end

  def action_lookup
    @action_lookup ||= {
      standing:    { frame_count: 1, hold_for: 8, repeat: true },
      running:     { frame_count: 4, hold_for: 8, repeat: true },
      first_jump:  { frame_count: 2, hold_for: 8, repeat: true, repeat_index: 1, },
      midair_jump: { frame_count: @midair_jump_frame_count, hold_for: @midair_jump_hold_for, repeat: true, repeat_index: 8 },
      falling:     { frame_count: 1, hold_for: 8, repeat: true }
    }
  end

  def new_game
    @clock                   = 0
    @game_over               = false
    @gravity                 = -0.4
    @drag                    = 0.15

    @player = new_entity(from_entity: { x: 640 - 8, y: 500 })
    @shadows  = []

    @activation_time         = 90
    @light_meter             = 600
    @light_meter_target       = 0

    @midair_jump_frame_count = 9
    @midair_jump_hold_for    = 6
    @midair_jump_duration    = @midair_jump_frame_count * @midair_jump_hold_for

    # hard coded collision tiles
    @tiles = [
      { x: 0,                        y: 0,   w: 1280, h: 8,    impassable: true },
      { x: 0,                        y: 0,   w: 8,    h: 1500, impassable: true },
      { x: 1280 - 8,                 y: 0,   w: 8,    h: 1500, impassable: true },

      { x: 80 + 320 + 80,            y: 128, w: 320,  h: 8 },
      { x: 80 + 320 + 80 + 320 + 80, y: 192, w: 320,  h: 8 },

      { x: 160,                      y: 320, w: 400,  h: 8 },
      { x: 160 + 400 + 160,          y: 400, w: 400,  h: 8 },

      { x: 320,                      y: 600, w: 320,  h: 8 },

      { x: 8,                        y: 500, w: 100,  h: 8 },

      { x: 8,                        y: 60,  w: 100,  h: 8 },
    ]

    @input_timeline = [
      { at: 0, k: :left_right, v: 0 },
      { at: 0, k: :space,      v: false },
      { at: 0, k: :down,       v: false },
    ]

    @jitter_fade_out_render_queue   = []
    @light_crystal = new_light_crystal
  end

  def new_light_crystal
    r = { x: 124 + rand(1000), y: 135 + rand(500), w: 64, h: 64 }
    return new_light_crystal if @tiles.any? { |t| t.intersect_rect? r }
    return new_light_crystal if (@player.x - r.x).abs < 200
    r
  end

  def entity_active? entity
    return true unless entity.activate_at
    return entity.activate_at <= @clock
  end

  def add_shadow!
    s = new_entity(from_entity: @player)
    s.activate_at = @clock + @activation_time * (@shadows.length + 1)
    s.spawn_countdown = @activation_time
    # the shadow is stamped from the player, who has already been simulated
    # and is therefore marked as activated. this is a brand new entity that
    # hasn't spawned in yet, so clear the flag to let it announce itself.
    s.activated = false
    @shadows << s
  end

  def find_input_timeline at:, key:;
    @input_timeline.find { |t| t.at <= at && t.k == key }.v
  end

  def new_entity(from_entity:)
    # these are all the properties of an entity
    # for "cloning" an entity/setting an entities
    # starting state, properties from_entity will
    # override these default values
    pe = {
      w: 96,
      h: 96,
      jump_power: 12,
      dy: 0,
      dx: 0,
      jumped_down_at: 0,
      jumped_at: @clock,
      jump_count: 1,
      clock: @clock,
      orientation: :right,
      action: :falling,
      action_at: @clock,
      left_right: 0
    }

    pe.merge from_entity
  end

  def entity_on_platform? entity
    entity.action == :standing || entity.action == :running
  end

  def entity_action_complete? entity, action_duration
    entity.action_at.elapsed_time(entity.clock) + 1 >= action_duration
  end

  def entity_set_action! entity, action
    entity.action = action
    entity.action_at = entity.clock
    entity.last_standing_at = entity.clock if action == :standing
  end
end

DR.reset_and_replay "replay.txt", speed: 1
