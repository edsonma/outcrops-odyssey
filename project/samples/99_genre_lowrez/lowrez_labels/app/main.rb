module Main
  def tick
    outputs.primitives << {
      x: 8, y: 0, text: "Aa", size_px: 10,
      anchor_x: 0, anchor_y: 0,
      font: "fonts/lowrez.ttf",
    }

    outputs.primitives << {
      x: 32, y: 0, text: "Bb", size_px: 20,
      anchor_x: 0, anchor_y: 0,
      font: "fonts/lowrez.ttf",
    }

    outputs.primitives << {
      x: 72, y: 0, text: "Cc", size_px: 30,
      anchor_x: 0, anchor_y: 0,
      font: "fonts/lowrez.ttf",
    }

    outputs.primitives << {
      x: 8, y: 64, text: "Aa", size_px: 10,
      anchor_x: 0, anchor_y: 0,
      font: "tiny.ttf",
    }

    outputs.primitives << {
      x: 32, y: 64,
      text: "Bb", size_px: 20,
      anchor_x: 0, anchor_y: 0,
      font: "tiny.ttf",
    }

    outputs.primitives << {
      x: 64, y: 64,
      text: "Cc", size_px: 30,
      anchor_x: 0, anchor_y: 0,
      font: "tiny.ttf",
    }

    outputs.primitives << {
      x: 96, y: 64,
      text: "Dd", size_px: 40,
      anchor_x: 0, anchor_y: 0,
      font: "tiny.ttf",
    }

    outputs.primitives << {
      x: 4, y: 128 - 10,
      text: "Line 1",
      anchor_x: 0, anchor_y: 0.5,
      font: "tiny.ttf", size_px: 10
    }

    outputs.primitives << {
      x: 4, y: 128 - 10,
      text: "Line 2",
      anchor_x: 0, anchor_y: 1.5,
      font: "tiny.ttf", size_px: 10
    }

    outputs.primitives << {
      x: 4, y: 128 - 10,
      text: "Line 3",
      anchor_x: 0, anchor_y: 2.5,
      font: "tiny.ttf", size_px: 10
    }

    outputs.primitives << {
      x: 4, y: 128 - 10,
      text: "Line 4",
      anchor_x: 0, anchor_y: 3.5,
      font: "tiny.ttf", size_px: 10
    }
  end
end

DR.reset
