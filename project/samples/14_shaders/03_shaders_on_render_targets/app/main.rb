require "shaders/blue_left_side.fragment.hlsl"
require "shaders/blue_right_side.fragment.hlsl"

module Main
  def start
  end

  def tick
    outputs[:le_circle].set w: 100, h: 100, background_color: [0, 0, 0, 0]
    outputs[:le_circle].shader = {
      path: "shaders/blue_left_side.fragment.hlsl"
    }

    outputs[:le_circle].primitives << {
      x: 0,
      y: 0,
      w: 100,
      h: 100,
      path: "sprites/circle/blue.png",
      anchor_x: 0,
      anchor_y: 0
    }

    outputs[:le_square].set w: 100, h: 100, background_color: [0, 0, 0, 0]
    outputs[:le_square].shader = {
      path: "shaders/blue_right_side.fragment.hlsl"
    }

    outputs[:le_square].primitives << {
      x: 0,
      y: 0,
      w: 100,
      h: 100,
      path: "sprites/square/blue.png",
      anchor_x: 0,
      anchor_y: 0
    }

    outputs.primitives << {
      x: 640 + 100,
      y: 360,
      w: 100,
      h: 100,
      path: :le_square,
      anchor_x: 0.5,
      anchor_y: 0.5
    }

    outputs.primitives << {
      x: 640 - 100,
      y: 360,
      w: 100,
      h: 100,
      path: :le_circle,
      anchor_x: 0.5,
      anchor_y: 0.5
    }
  end
end

DR.reset
