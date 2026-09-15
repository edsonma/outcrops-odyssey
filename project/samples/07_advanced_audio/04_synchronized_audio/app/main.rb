module Main
  def start
    audio[:bg] = {
      input: 'sounds/bg.ogg',
      gain: 1.0,
      looping: true,
      keyboard_key: :one
    }

    audio[:bass] = {
      input: 'sounds/bass.ogg',
      gain: 1.0,
      looping: true,
      keyboard_key: :two
    }

    audio[:strings] = {
      input: 'sounds/strings.ogg',
      looping: true,
      gain: 0.0,
      keyboard_key: :three
    }

    audio[:strings_2] = {
      input: 'sounds/strings_2.ogg',
      looping: true,
      gain: 0.0,
      keyboard_key: :four
    }

    audio[:alert] = {
      input: 'sounds/alert.ogg',
      looping: true,
      gain: 0.0,
      keyboard_key: :five
    }

    audio[:heartbeat] = {
      input: 'sounds/heartbeat.ogg',
      looping: true,
      gain: 0.0,
      keyboard_key: :six
    }

    @active_tracks = [
      :bg,
      :bass,
    ]
  end

  def tick
    tick_input
    tick_audio
    render
  end

  def tick_input
    audio.each do |key, track|
      if inputs.keyboard.key_down? track.keyboard_key
        if @active_tracks.include? key
          @active_tracks.delete key
        else
          @active_tracks << key
        end
      end
    end
  end

  def tick_audio
    audio.each do |name, track|
      if @active_tracks.include? name
        track.gain += 0.01
      else
        track.gain -= 0.01
      end
    end

    audio.each do |name, track|
      track.gain = track.gain.clamp(0.0, 1.0)
    end
  end

  def render
    white = { r: 255, g: 255, b: 255 }
    green = { r: 100, g: 255, b: 100 }
    gray = { r: 100, g: 100, b: 100 }
    outputs.background_color = [30, 30, 30]
    outputs.labels << { x: 8, y: 720 - 16, **white, size_px: 16, text: "Active sounds: #{@active_tracks}", anchor_y: 0.5 }
    outputs.labels << audio.map.with_index do |(key, h), i|
      starting_anchor_y = 1.5 + i * 5
      label_style = { **white, size_px: 16 }
      label_style = { **green, size_px: 16 } if h.gain == 1
      label_style = { **gray, size_px: 16 } if h.gain == 0
      [
        { x: 8,  y: 720 - 16, **white, size_px: 16,
          text: "Press [#{h.keyboard_key}] on the keyboard to toggle ~:#{key}~ track",
          anchor_y: starting_anchor_y + 0 },
        { x: 24, y: 720 - 16, **label_style,
          text: "input:    #{h.input}",
          anchor_y: starting_anchor_y + 1 },
        { x: 24, y: 720 - 16, **label_style,
          text: "gain:     #{h.gain.to_sf}",
          anchor_y: starting_anchor_y + 2 },
        { x: 24, y: 720 - 16, **label_style,
          text: "playtime: #{h.playtime.to_sf}s",
          anchor_y: starting_anchor_y + 3 },
        { x: 24, y: 720 - 16, **label_style,
          text: "",
          anchor_y: starting_anchor_y + 4 },
      ]
    end

    outputs.labels << { x: 480,
                        y: 720 - 32,
                        text: "Sounds provided under MIT by Travis Wattigney.",
                        size_px: 32,
                        **white,
                        anchor_x: 0,
                        anchor_y: 0.5 }

    outputs.labels << { x: 480,
                        y: 720 - 32,
                        text: "W:  traviswattigney.com",
                        size_px: 32,
                        **white,
                        anchor_x: 0,
                        anchor_y: 1.5 }

    outputs.labels << { x: 480,
                        y: 720 - 32,
                        text: "IG: @traviswattigney",
                        size_px: 32,
                        **white,
                        anchor_x: 0,
                        anchor_y: 2.5 }
  end
end

DR.reset
