require "shaders/effect.frag.hlsl"

module Main
  def start
  end

  def tick
    sprite_size = 720

    outputs[:water].set w: 1280, h: 720, background_color: [255, 255, 255]
    outputs[:water].primitives << 3.map do |i|
      [
        { x: 0,
          y: Grid.h / 2 - i * sprite_size + Kernel.tick_count % sprite_size,
          w: sprite_size, h: sprite_size,
          path: "sprites/flow-like-water/water.png", a: 128 },
        { x: 720,
          y: Grid.h / 2 - i * sprite_size + Kernel.tick_count % sprite_size,
          w: sprite_size, h: sprite_size,
          path: "sprites/flow-like-water/water.png", a: 128 },
        { x: Grid.w / 2 - i * sprite_size + Kernel.tick_count % sprite_size,
          y: 0,
          w: sprite_size, h: sprite_size,
          path: "sprites/flow-like-water/water.png", a: 128 },
      ]
    end

    outputs[:displacement].set w: 1280, h: 720, background_color: [255, 255, 255]
    outputs[:displacement].primitives << 3.map do |i|
      [
        { x: Grid.w / 2 - i * sprite_size + Kernel.tick_count % sprite_size,
          y: 0,
          w: sprite_size, h: sprite_size,
          path: "sprites/flow-like-water/water-displacement.png", a: 128 },
        { x: 0,
          y: Grid.h / 2 - i * sprite_size + Kernel.tick_count % sprite_size,
          w: sprite_size, h: sprite_size,
          path: "sprites/flow-like-water/water-displacement.png", a: 128 },
        { x: 720,
          y: Grid.h / 2 - i * sprite_size + Kernel.tick_count % sprite_size,
          w: sprite_size, h: sprite_size,
          path: "sprites/flow-like-water/water-displacement.png", a: 128 }
      ]
    end

    outputs.shader = {
      path: "shaders/effect.frag.hlsl",
      textures: [:displacement]
    }

    outputs.primitives << { x: 0, y: 0, w: 1280, h: 720, path: :water }

    outputs.primitives << { x: Kernel.tick_count % 1280,
                            y: Kernel.tick_count % 720,
                            text: "flow like water",
                            scale_quality_enum: 0,
                            anchor_x: 0.5,
                            anchor_y: 0.5,
                            size_px: 100,
                            r: 1, g: 1, b: 1 }

    outputs.primitives << { x: inputs.mouse.x,
                            y: inputs.mouse.y,
                            w: 64, h: 64,
                            path: "sprites/square/blue.png",
                            anchor_x: 0.5, anchor_y: 0.5 }
  end
end

DR.reset
