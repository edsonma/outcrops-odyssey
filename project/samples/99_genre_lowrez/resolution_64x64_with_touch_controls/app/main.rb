# sample app shows how to display on-screen touch controls with a 64x64 canvas
# see ./metadata/game_metadata.txt for aspect_size and aspect_mode configuration.

module Main
  def start
    @on_screen_gamepad = OnScreenGamepad.new inputs, neutral_pos: { x: 14, y: 25 }

    @player = { x: 32 - 8, y: 32 - 8, w: 16, h: 16, r: 80, g: 255, b: 128, path: :solid, angle: 0 }

    @touch_a_button = Geometry.rect(x: 64 - 8 - 8 - 8 - 10,
                                    y: 25 - 14,
                                    w: 16,
                                    h: 16)

    @touch_b_button = Geometry.rect(x: 64 - 8 - 10,
                                    y: 25 - 2,
                                    w: 16,
                                    h: 16)
  end

  def tick
    @on_screen_gamepad.tick

    # outputs.watch "#{@on_screen_gamepad.dpad_vector&.x}", font: "tiny.ttf", size_px: 10
    # outputs.watch "#{@on_screen_gamepad.dpad_vector&.y}", font: "tiny.ttf", size_px: 10

    if @on_screen_gamepad.dpad_vector
      @player.x += @on_screen_gamepad.dpad_vector.x
      @player.y += @on_screen_gamepad.dpad_vector.y
    else
      @player.x += inputs.left_right
      @player.y += inputs.up_down
    end


    @player.x = @player.x.clamp(0, 64 - @player.w)
    @player.y = @player.y.clamp(0, 64 - @player.h)

    if spin_left?
      @player.angle += 2
    elsif spin_right?
      @player.angle -= 2
    end

    # game will be rendered to a Render Target with size 64x64
    outputs[:lowrez].set w: 64, h: 64, background_color: [30, 30, 30]
    outputs[:lowrez].primitives << @player

    # the render target will be positioned at the top of the portrait
    # screen
    outputs.primitives << {
      x: 0,
      y: Grid.top - 64,
      w: 64,
      h: 64,
      path: :lowrez
    }

    # touch controls will be positioned at the bottom
    outputs.primitives << button_primitives(@touch_a_button, "A", is_down: spin_left?)
    outputs.primitives << button_primitives(@touch_b_button, "B", is_down: spin_right?)
    outputs.primitives << @on_screen_gamepad.primitives
    if inputs.finger_left
      outputs.primitives << { x: inputs.finger_left.x, y: inputs.finger_left.y, w: 16, h: 16, path: :solid, r: 0, g: 0, b: 255, anchor_x: 0.5, anchor_y: 0.5, a: 128 }
    end

    if inputs.finger_right
      outputs.primitives << { x: inputs.finger_right.x, y: inputs.finger_right.y, w: 16, h: 16, path: :solid, r: 255, g: 0, b: 0, anchor_x: 0.5, anchor_y: 0.5, a: 128  }
    end
  end

  def button_primitives rect, label, is_down: false
    button_color = if is_down
                     { r: 0, g: 0, b: 0, a: 255 }
                   else
                     { r: 0, g: 0, b: 0, a: 32 }
                   end
    [
      rect.merge(path: "sprites/circle/solid.png",
                 **button_color),
      rect.center
          .merge(text: label,
                 font: "tiny.ttf",
                 size_px: 10,
                 r: 255, g: 255, b: 255,
                 anchor_x: 0.5, anchor_y: 0.5)
    ]
  end

  def spin_left?
    if inputs.last_active == :mouse
      return inputs.finger_right && Geometry.intersect_rect?(inputs.finger_right, @touch_a_button)
    else
      return inputs.keyboard.h || inputs.controller_one.south
    end
  end

  def spin_right?
    if inputs.last_active == :mouse
      return inputs.finger_right && Geometry.intersect_rect?(inputs.finger_right, @touch_b_button)
    else
      return inputs.keyboard.l || inputs.controller_one.east
    end
  end
end

class OnScreenGamepad
  attr :inputs, :directional_vector, :directional_angle, :dpad_vector

  def initialize inputs, side: :left, neutral_pos: nil
    @neutral_pos = neutral_pos
    @side = side
    @inputs = inputs
    if @neutral_pos
      @joystick = {
        center: @neutral_pos,
        a: 32
      }
    end
  end

  def tick
    if finger
      @joystick ||= {
        center: @neutral_pos || { x: finger.x, y: finger.y },
        a: 32
      }
      @joystick.distance = Geometry.distance(finger, @joystick.center)
      @joystick.angle = Geometry.angle(finger, @joystick.center)
      @joystick.vector = @joystick.angle.to_vector
      if @joystick.distance > 160
        @joystick.center.x = @joystick.center.x.lerp(finger.x + @joystick.vector.x * 160, 0.1)
        @joystick.center.y = @joystick.center.y.lerp(finger.y + @joystick.vector.y * 160, 0.1)
      end

      perc = @joystick.distance.clamp(0, 48).fdiv(48)
      @directional_angle = (@joystick.angle + 180) % 360

      @directional_vector = {
        x: @directional_angle.to_vector.x * perc ** 4 * 10,
        y: @directional_angle.to_vector.y * perc ** 4 * 10,
      }

      if perc > 0.1
        @dpad_vector = {
          x: Geometry.angle_cardinal_vec2(@directional_angle).x,
          y: Geometry.angle_cardinal_vec2(@directional_angle).y,
        }
      else
        @dpad_vector = nil
      end

      @joystick.a = @joystick.a.lerp(128, 0.01)
    elsif @joystick
      @joystick.a = @joystick.a.lerp(32, 0.25)
      @directional_vector = nil
      @dpad_vector = nil
      @joystick = nil if @joystick.a < 1
    end
  end

  def joystick_primitives
    center = if @joystick
               @joystick.center
             elsif @neutral_pos
               @neutral_pos
             else
               nil
             end

    return nil if !center

    [
      {
        **center,
        w: 24,
        h: 24,
        path: "sprites/circle/solid.png",
        anchor_x: 0.5,
        anchor_y: 0.5,
        r: 30,
        g: 30,
        b: 30,
        a: @joystick.a - 16
      },
      {
        **center,
        text: "ʘ",
        font: "tiny.ttf",
        size_px: 60,
        anchor_x: 0.5,
        anchor_y: 0.5,
        r: 0,
        g: 0,
        b: 0,
        a: @joystick.a
      },
    ]
  end

  def finger
    if @side == :left
      inputs.finger_left
    else
      inputs.finger_right
    end
  end

  def direction_indicator_primitives
    return nil if !finger
    return nil if !@joystick

    joystick_center = if @joystick.distance > 48
                        { x: @joystick.center.x + 48 * -@joystick.vector.x,
                          y: @joystick.center.y + 48 * -@joystick.vector.y }
                      else
                        finger
                      end

    {
      x: joystick_center.x,
      y: joystick_center.y,
      w: 4,
      h: 4,
      path: "sprites/circle/solid.png",
      r: 255,
      g: 0,
      b: 0,
      a: @joystick.a,
      anchor_x: 0.5,
      anchor_y: 0.5
    }
  end

  def primitives
    [
      joystick_primitives,
      direction_indicator_primitives
    ]
  end
end

DR.reset
