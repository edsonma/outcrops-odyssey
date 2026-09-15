require "shaders/time_and_mouse.fragment.hlsl"

module Main
  def start
  end

  def tick
    outputs.background_color = [0, 0, 0]
    outputs.shader = {
      path: "shaders/time_and_mouse.fragment.hlsl",
      uniforms: [
        { type: :int, value: Kernel.tick_count },
        { type: :float, value: inputs.mouse.x.fdiv(Grid.w) },
        { type: :float, value: inputs.mouse.y.fdiv(Grid.h) },
      ]
    }
  end
end

DR.reset
