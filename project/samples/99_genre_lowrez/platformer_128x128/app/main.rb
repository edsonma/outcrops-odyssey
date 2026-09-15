class Game
  attr :camera, :player, :clock

  FREEZE_TICKS = 4

  def initialize rooms
    @room_index = 0
    @rooms = rooms
    @player = Player.new player_starting_point.ordinal_x,
                         player_starting_point.ordinal_y
    @camera = {
      trauma: 0,
      x_offset: 0,
      y_offset: 0
    }
    @clock = 0
    @freeze_ticks = 0
  end

  def walls
    @rooms[@room_index].walls
  end

  def tiles
    @rooms[@room_index].tiles
  end

  def spikes
    @rooms[@room_index].spikes
  end

  def player_starting_point
    @rooms[@room_index].player
  end

  def tick inputs
    if @freeze_ticks > 0
      @freeze_ticks -= 1
      tick_camera
      return
    end

    load_next_room!

    spike = Geometry.find_intersect_rect @player, spikes, using: :rect
    if spike || @player.y < -8
      kill_and_reset_player!
    end

    @player.tick inputs, walls, @clock

    if @player.dash_at == @clock
      @camera.trauma = 0.1
      @freeze_ticks = FREEZE_TICKS
    end

    if @player.spawned_at && @player.on_ground_at == @clock
      @player.spawned_at = nil
      @camera.trauma = 0.2
    end

    @clock += 1
    tick_camera
  end

  def load_next_room!
    return if @player.y <= 128
    @room_index += 1
    if @room_index >= @rooms.length
      @room_index = 0
    end
    @player.reset player_starting_point.ordinal_x,
                  player_starting_point.ordinal_y,
                  @clock
  end

  def kill_and_reset_player!
    @camera.trauma = 0.2
    @player.reset player_starting_point.ordinal_x,
                  player_starting_point.ordinal_y,
                  @clock
  end

  def tick_camera
    next_offset       = 100.0 * @camera.trauma**2
    @camera.x_offset = next_offset.randomize(:sign, :ratio)
    @camera.y_offset = next_offset.randomize(:sign, :ratio)
    @camera.trauma *= 0.95
  end

  def primitives
    [
      room_primitives,
      @player.primitives
    ]
  end

  def room_primitives
    tiles.map do |h|
      color = if h.type == :wall
                { r: 30, g: 128, b: 255, a: 255 }
              elsif h.type == :spike
                { r: 255, g: 128, b: 30, a: 255 }
              else
                { r: 0, g: 0, b: 0, a: 0 }
              end

      {
        **h.rect,
        path: :solid,
        **color
      }
    end
  end
end

class Player
  attr :x, :y, :w, :h, :dead, :on_ground, :on_ground_grace,
       :has_dashed, :facing_x, :dx, :dy, :jump_power, :dash_power, :gravity,
       :dash_at, :spawned_at, :on_ground_at

  WALL_KICK_LOCK = 12
  DASH_FRAMES  = 12
  DASH_SUSTAIN = 2
  DASH_ACCEL   = 0.25
  APEX_SPEED   = 0.15

  def initialize ordinal_x, ordinal_y
    reset ordinal_x, ordinal_y, -1
  end

  def reset ordinal_x, ordinal_y, clock
    @reset_at = clock + 1
    @ordinal_x = ordinal_x
    @ordinal_y = ordinal_y
    @max_dx = 1
    @facing_x = 1
    @dx = 0
    @jump_power = 2
    @dash_power = 2.5
    @gravity = 0.10
    @maxfall = 2
    @dash_target_dx = 0
    @dash_target_dy = 0
    @dash_accel_x = DASH_ACCEL
    @dash_accel_y = DASH_ACCEL
    @x = @ordinal_x * 8
    @y = -8
    @on_ground = false
    @on_ground_at = nil
    @on_ground_grace = 6
    @w = 8
    @h = 8
    @dy = 0
    @dash_ticks = 0
    @wall_kick_ticks = 0
  end

  def rect
    {
      x: @x,
      y: @y,
      w: 8,
      h: 8,
    }
  end

  def solid_at? walls, ox, oy
    probe = { rect: { x: @x + ox, y: @y + oy, w: @w, h: @h } }
    Geometry.find_intersect_rect probe, walls, using: :rect
  end

  def tick inputs, walls, clock
    if @reset_at
      tick_respawn walls, clock
      return
    end

    collide_x! walls, clock
    collide_y! walls, clock

    if @dash_ticks > 0
      @dash_ticks -= 1
      @dx = @dx.towards(@dash_target_dx, @dash_accel_x)
      @dy = @dy.towards(@dash_target_dy, @dash_accel_y)
      @dash_ticks = @dash_ticks.clamp(0, DASH_FRAMES)
    else
      dx! inputs
      dy! inputs, walls
    end

    if dash_pressed?(inputs) && @dash_ticks <= 0 && !@dash_at
      dash! inputs, clock
    elsif jump_pressed?(inputs)
      jump! inputs, walls, clock
    end
  end

  def tick_respawn walls, clock
    if @reset_at == clock - 1
      @y = -8
      @dy = @jump_power
    end

    if @y > (@ordinal_y - 1.5) * 8
      g = @gravity
      @dy = @dy.towards(-@maxfall, g)
    end

    if @y > @ordinal_y * 8
      @reset_at = nil
      @spawned_at = clock
    end

    if @reset_at
      @y += @dy
    end
  end

  def collide_x! walls, clock
    @x += @dx
    collision = Geometry.find_intersect_rect self, walls, using: :rect
    if collision
      if @dx > 0
        @x = collision.rect.x - @w
      elsif @dx < 0
        @x = collision.rect.x + collision.rect.w
      end
      @dx = 0
    end

    if @x < 0
      @x = 0
    elsif @x + @w > 128
      @x = 128 - w
    end
  end

  def collide_y! walls, clock
    @y += @dy
    collision = Geometry.find_intersect_rect self, walls, using: :rect
    if collision
      if @dy > 0
        @y = collision.rect.y - @h
      elsif @dy < 0
        @y = collision.rect.y + collision.rect.h
        @on_ground = true
        @on_ground_at = clock
        @dash_ticks = 0
        @dash_at = nil
        @on_ground_grace = 6
      end
      @dy = 0
    elsif @on_ground
      lower_player = {
        rect: { x: @x, y: @y - 4, w: 8, h: 8 }
      }
      collision = Geometry.find_intersect_rect lower_player, walls, using: :rect
      if !collision
        @on_ground_grace -= 1
        if @on_ground_grace < 0
          @on_ground = false
          @on_ground_at = nil
        end
      end
    end
  end

  def dx! inputs
    if !@spawned_at
      if @wall_kick_ticks > 0
        @wall_kick_ticks -= 1
      else
        accel = 0.489
        deccel = 0.3
        if inputs.left_right == 0
          @dx = @dx.lerp(0, deccel)
        else
          @dx = @dx.lerp(@max_dx * inputs.left_right, accel)
        end
      end
    end

    @facing_x = @dx < 0 ? -1 : 1 if @dx != 0
  end

  def dy! inputs, walls
    maxfall = @maxfall
    slide_dir = inputs.left_right
    if slide_dir != 0 && !@on_ground && solid_at?(walls, slide_dir, 0)
      maxfall = 0.4
    end
    g = @gravity
    g *= 0.5 if @dy.abs <= APEX_SPEED
    @dy = @dy.towards(-maxfall, g)
  end

  def jump_pressed? inputs
    inputs.keyboard.key_down.j || inputs.controller_one.key_down.s
  end

  def dash_pressed? inputs
    inputs.keyboard.key_down.k || inputs.controller_one.key_down.e
  end

  def dash! inputs, clock
    @dash_ticks = DASH_FRAMES
    @dash_at = clock
    dv = inputs.directional_vector || { x: @facing_x, y: 0 }
    @dx = dv.x * @dash_power
    @dy = dv.y * @dash_power
    @dash_target_dx = DASH_SUSTAIN * @dx.sign
    @dash_target_dy = DASH_SUSTAIN * @dy.sign
    @dash_accel_x = DASH_ACCEL
    @dash_accel_y = DASH_ACCEL
    @dash_target_dy *= 0.5 if @dy > 0
    @dash_accel_x *= 0.7071 if @dy != 0
    @dash_accel_y *= 0.7071 if @dx != 0
  end

  def jump! inputs, walls, clock
    return if @dash_ticks > 0
    if @on_ground
      @on_ground = false
      @dy = @jump_power
    else
      wall_dir = if    solid_at?(walls, -3, 0) then -1
                 elsif solid_at?(walls,  3, 0) then  1
                 else 0
                 end
      if wall_dir != 0
        @dy = @jump_power
        @dx = -wall_dir * (@max_dx + 1)
        @facing_x = -wall_dir
        @wall_kick_ticks = WALL_KICK_LOCK
      end
    end
  end

  def primitives
    {
      **rect,
      path: :solid,
      r: 30, g: 255, b: 128
    }
  end
end

module Main
  def start
    @game = Game.new parse_rooms
  end

  def tick
    @game.tick inputs
    outputs.background_color = [30, 30, 30]
    outputs[:scene].background_color = [30, 30, 30]
    outputs[:scene].primitives << @game.primitives
    outputs.primitives << {
      x: 0 - @game.camera.x_offset,
      y: 0 - @game.camera.y_offset,
      w: 128,
      h: 128,
      path: :scene,
    }
  end

  def parse_rooms
    ascii_rooms.map do |ascii_room|
      parse_room ascii_room
    end
  end

  def parse_room ascii_room
    symbol_room = ascii_room.strip
                            .split("\n")
                            .reverse
                            .map
                            .with_index do |line, row|
                              line.chars.map do |char, col|
                                case char
                                when "0"
                                  :wall
                                when "X"
                                  :spike
                                when "P"
                                  :player
                                else
                                  :empty
                                end
                              end
                            end

    tiles = []
    walls = []
    spikes = []
    player = nil
    symbol_room.each_with_index do |xs, y|
      xs.each_with_index do |t, x|
        if t == :player
          player = { ordinal_x: x, ordinal_y: y }
        elsif t != :empty
          entry = {
            ordinal_x: x,
            ordinal_y: y,
            type: t,
            rect: {
              x: x * 8,
              y: y * 8,
              w: 8,
              h: 8
            },
          }
          tiles << entry
          if t == :wall
            walls << entry
          elsif t == :spike
            spikes << entry
          end
        end
      end
    end

    {
      player: player,
      tiles: tiles,
      walls: walls,
      spikes: spikes
    }
  end

  def ascii_rooms
    [
      <<~S,
      0000000000000  0
      0000000        0
      0000           0
      000          000
      000         0000
      000           00
      00000          0
      00             0
      0              0
      0            000
                    00
             00     00
       P 00  00  XXX00
      00000  00XX00000
      00000XX000000000
      0000000000000000
      S
      <<~S
      000000000000   0
      000000    00   0
      0000      00   0
      000       00   0
      000       0    0
      000    X  0    0
      0      0      00
      0000   0      00
      0000   0      00
      0000   0      00
      0000   0     000
      0000   0       0
        00   0       0
         0   0XXX
       P     0000
      00000000000
      S
    ]
  end
end

# DR.reset_and_replay speed: 2
DR.reset
