module Main
  def start
    @player = {
      x: 64 - 2,
      y: 64 - 4,
      w: 4,
      h: 8,
      path: :solid,
      r: 128, g: 128, b: 255
    }

    @button_rect = {
      x: 64 - 36,
      y: 64 - 16,
      w: 72,
      h: 8,
    }

    @button_text = "Click Me!"
  end

  def tick
    calc
    render
  end

  def calc
    # wasd and arrow keys via inputs.left_right and inputs.up_down
    @player.x += inputs.left_right
    @player.y += inputs.up_down

    # make sure the player doesn't go outside the grid
    @player.x = @player.x.clamp(0, 128 - @player.w)
    @player.y = @player.y.clamp(0, 128 - @player.h)

    # click of the button
    if inputs.mouse.key_down.left && Geometry.inside_rect?(inputs.mouse, @button_rect)
      @button_text = "Clicked at #{Kernel.tick_count}"
    end
  end

  def render
    outputs.background_color = [30, 30, 30]

    # instructions
    outputs.primitives << {
      x: Grid.w.idiv(2),
      y: Grid.h - 5,
      text: "WASD or ARROW KEYS to move",
      font: "tiny.ttf",
      size_px: 10,
      anchor_x: 0.5,
      anchor_y: 0.5,
      r: 255, g: 255, b: 255
    }

    # label showing mouse location
    outputs.primitives << {
      x: Grid.w.idiv(2),
      y: Grid.h - 15,
      text: "Mouse: #{inputs.mouse.x}, #{inputs.mouse.y}",
      font: "tiny.ttf",
      size_px: 10,
      anchor_x: 0.5,
      anchor_y: 0.5,
      r: 255, g: 255, b: 255
    }

    # button rect and label
    outputs.primitives << {
      **@button_rect,
      path: :solid,
      r: 128, g: 128, b: 128
    }

    outputs.primitives << {
      **Geometry.rect(@button_rect).center,
      text: @button_text,
      size_px: 10,
      anchor_x: 0.5,
      anchor_y: 0.5,
      font: "tiny.ttf"
    }

    # player rendering with position
    # overlay
    outputs.primitives << {
      x: @player.x + @player.w.idiv(2),
      y: @player.y + @player.h + 5,
      text: "#{@player.x}, #{@player.y}",
      size_px: 10,
      anchor_x: 0.5,
      anchor_y: 0.5,
      r: 255, g: 255, b: 255,
      font: "tiny.ttf"
    }

    outputs.primitives << @player
  end
end

DR.reset
