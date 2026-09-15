require "shaders/effect.frag.hlsl"

module Main
  def start
  end

  def tick
    outputs.shader = {
      path: "shaders/effect.frag.hlsl"
    }

    outputs.primitives << {
      x: 640,
      y: 360,
      w: 100,
      h: 100,
      path: "sprites/square/blue.png",
      anchor_x: 0.5,
      anchor_y: 0.5
    }
  end
end

DR.reset
