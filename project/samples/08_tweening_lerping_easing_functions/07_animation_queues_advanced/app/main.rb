module Main
  def start
    # place a player on the far left with sprite and hp information
    @player = {
      x: 100,
      y: 360 - 50,
      w: 100,
      h: 100,
      path: "sprites/square/blue.png",
      hp: 30
    }

    # create an array of bullets
    @bullets = []

    # create a queue for handling bullet explosions
    @explosion_queue = []
  end

  def tick
    spawn_bullets
    calc_bullets
    render
  end

  def spawn_bullets
    # span a bullet in a random location on the far right every half second
    return if !Kernel.tick_count.zmod? 30
    @bullets << {
      x: 1280 - 100,
      y: rand(720 - 100),
      w: 100,
      h: 100,
      path: "sprites/square/red.png"
    }
  end

  def calc_bullets
    # for each bullet
    @bullets.each do |b|
      # move it to the left by 20 pixels
      b.x -= 20

      # determine if the bullet collides with the player
      if Geometry.intersect_rect? b, @player
        # decrement the player's health if it does
        @player.hp -= 1

        # mark the bullet as exploded
        b.exploded = true

        # queue the explosion by adding it to the explosion queue
        @explosion_queue << b.merge(exploded_at: Kernel.tick_count)
      end
    end

    # remove bullets that have exploded or off the screen
    @bullets.reject! { |b| b.exploded || (b.x + b.w) < 0 }

    # remove animations from the animation queue that have completed
    # frame index will return nil once the animation has completed
    @explosion_queue.reject! { |e| frame_for(e).completed }
  end

  def render
    outputs.watch "#{@explosion_queue.length}"
    outputs.watch "#{@bullets.length}"
    # render the player's hp above the sprite
    outputs.primitives << {
      x: @player.x + @player.w / 2,
      y: @player.y + @player.h + 4 + 16,
      text: "#{@player.hp}",
      size_px: 32,
      anchor_x: 0.5,
      anchor_y: 0.5
    }

    # render the player
    outputs.primitives << @player

    # render the bullets
    outputs.sprites << @bullets

    # process the animation queue
    outputs.primitives << @explosion_queue.map do |e|
      # use the exploded_at property and the frame_index function to determine when the animation should start
      frame = frame_for e

      # take the explosion primitive and set the path variariable
      { **e, path: "sprites/misc/explosion-#{frame.frame_index}.png" }
    end
  end

  def frame_for explosion
    Numeric.frame start_at: explosion.exploded_at,
                  hold_for: 4,
                  frame_count: 7,
                  repeat: false
  end
end
