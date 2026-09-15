module Main
  def tick
    outputs.background_color = [199, 240, 216]

    # uncomment the methods below on at a time to see the examples in action

    # ==== HELLO WORLD ======================================================
    # Steps to get started:
    # 1. ~def tick~ is the entry point for your game.
    # 2. There are quite a few code samples below, remove the "##"
    #    before each line and save the file to see the changes.
    # 3. 0,  0 is in bottom left and 63, 63 is in top right corner.
    # 4. Be sure to come to the discord channel if you need
    #    more help: [[http://discord.dragonruby.org]].

    # Commenting and uncommenting code:
    # - Add a "#" infront of lines to comment out code
    # - Remove the "#" infront of lines to comment out code

    # Invoke the hello_world subroutine/method
    hello_world # <---- add a "#" to the beginning of the line to stop running this subroutine/method.
    # =======================================================================

    # ==== HOW TO RENDER A LABEL ============================================
    # Uncomment the line below to invoke the how_to_render_a_label subroutine/method.
    # Note: The method is defined in this file with the signature ~def how_to_render_a_label~
    #       Scroll down to the method to see the details.
    #
    # Remove the "#" at the beginning of the line below
    # how_to_render_a_label # <---- remove the "#" at the begging of this line to run the method
    # =======================================================================

    # ==== HOW TO RENDER A FILLED SQUARE (SOLID) ============================
    # Remove the "#" at the beginning of the line below
    # how_to_render_solids
    # =======================================================================

    # == HOW TO RENDER A SPRITE =============================================
    # Remove the "#" at the beginning of the line below
    # how_to_render_sprites
    # =======================================================================

    # ==== HOW TO ANIMATE A SPRITE (SEPERATE PNGS) ==========================
    # Remove the "#" at the beginning of the line below
    # how_to_animate_a_sprite
    # =======================================================================

    # ==== HOW TO ANIMATE A SPRITE (SPRITE SHEET) ===========================
    # Remove the "#" at the beginning of the line below
    # how_to_animate_a_sprite_sheet
    # =======================================================================

    # ==== HOW TO DETERMINE COLLISION =======================================
    # Remove the "#" at the beginning of the line below
    # how_to_determine_collision
    # =======================================================================

    # ==== HOW TO CREATE BUTTONS ============================================
    # Remove the "#" at the beginning of the line below
    # how_to_create_buttons
    # =======================================================================

    # ==== SHOOTER GAME EXAMPLE =============================================
    # Remove the "#" at the beginning of the line below
    # shooter_game
    # =======================================================================
  end

  def hello_world
    # your canvas is 84x48

    # render a label at center x, near the top (centered horizontally is done by setting anchor_x: 0.5)
    outputs.primitives << {
      x: 84.idiv(2),
      y: 48 - 10,
      text: "Nokia 3310 Jam",
      size_px: 10, # size_px of 5 is a small font size, 10 is medium, 15 is large, 20 is extra large
      font: "tiny.ttf",
      anchor_x: 0.5,
      anchor_y: 0
    }

    # render a sprite at the center of the screen
    # and make it rotate
    outputs.primitives << {
      x: 84 / 2 - 10,
      y: 48 / 2 - 10,
      w: 20,
      h: 20,
      path: "sprites/monochrome-ship.png",
      angle: Kernel.tick_count.fdiv(2),
    }
  end

  def how_to_render_a_label
    # Render a small label (size_px: 10)
    outputs.labels << { x: 1,
                        y: 0,
                        text: "SMALL",
                        anchor_x: 0,
                        anchor_y: 0,
                        size_px: 10,
                        font: "tiny.ttf" }

    # Render a medium label (size_px: 20)
    outputs.labels << { x: 1,
                        y: 5,
                        text: "MEDIUM",
                        anchor_x: 0,
                        anchor_y: 0,
                        size_px: 20,
                        font: "tiny.ttf" }

    # Render a large label (size_px: 30)
    outputs.labels << { x: 1,
                        y: 14,
                        text: "LARGE",
                        anchor_x: 0,
                        anchor_y: 0,
                        size_px: 30,
                        font: "tiny.ttf" }

    # Render an extra large label (size_px: 40)
    outputs.labels << { x: 1,
                        y: 27,
                        text: "EXTRA LARGE",
                        anchor_x: 0,
                        anchor_y: 0,
                        size_px: 40,
                        font: "tiny.ttf" }
  end

  def how_to_render_solids
    # Render a square at 0, 0 with a width and height of 1 (setting path to :solid will render a solid color)
    outputs.sprites << { x: 0, y: 0, w: 1, h: 1, path: :solid, r: 0, g: 0, b: 0 }

    # Render a square at 1, 1 with a width and height of 2
    outputs.sprites << { x: 1, y: 1, w: 2, h: 2, path: :solid, r: 0, g: 0, b: 0 }

    # Render a square at 3, 3 with a width and height of 3
    outputs.sprites << { x: 3, y: 3, w: 3, h: 3, path: :solid, r: 0, g: 0, b: 0 }

    # Render a square at 6, 6 with a width and height of 4
    outputs.sprites << { x: 6, y: 6, w: 4, h: 4, path: :solid, r: 0, g: 0, b: 0 }
  end

  def how_to_render_sprites
    # add a sprite to the screen 10 times
    10.times do |i|
      outputs.primitives << {
        x: i * 8,
        y: i * 4,
        w: 5,
        h: 5,
        path: 'sprites/monochrome-ship.png'
      }
    end

    # add a sprite based on a position
    positions = [
      { x: 20, y: 32 },
      { x: 45, y: 15 },
      { x: 72, y: 23 },
    ]

    positions.each do |position|
      # use Ruby's ~Hash#merge~ function to create a sprite
      outputs.primitives << position.merge(path: 'sprites/monochrome-ship.png',
                                      w: 5,
                                      h: 5)
    end
  end

  def how_to_animate_a_sprite
    start_animation_on_tick = 180


    # Get the frame_index given start_at, frame_count, hold_for, and repeat
    sprite_index = Numeric.frame_index start_at: start_animation_on_tick,  # when to start the animation?
                                       frame_count: 7,                     # how many sprites?
                                       hold_for: 8,                        # how long to hold each sprite?
                                       repeat: true                        # should it repeat?

    # render the current tick and the resolved sprite index
    outputs.primitives  << sm_label.merge(x: 84 / 2,
                                          y: 48 - 10,
                                          text: "Tick: #{Kernel.tick_count}",
                                          anchor_x: 0.5)

    outputs.primitives  << sm_label.merge(x: 84 / 2,
                                          y: 48 - 20,
                                          text: "sprite_index: #{sprite_index || 'nil'}",
                                          anchor_x: 0.5)

    # Numeric.frame_index will return nil if the frame hasn't arrived yet
    if sprite_index
      # if the sprite_index is populated, use it to determine the sprite path and render it
      sprite_path  = "sprites/explosion-#{sprite_index}.png"
      outputs.primitives << { x: 84 / 2 - 16,
                              y: 0,
                              w: 32,
                              h: 32,
                              path: sprite_path }
    else
      # if the sprite_index is nil, render a countdown instead
      countdown_in_seconds = ((start_animation_on_tick - Kernel.tick_count) / 60).round(1)

      outputs.primitives  << sm_label.merge(x: 84 / 2,
                                      y: 48 / 2,
                                      text: "Count Down: #{countdown_in_seconds.to_sf}",
                                      anchor_x: 0.5,
                                      anchor_y: 0.5)
    end
  end

  def how_to_animate_a_sprite_sheet
    start_animation_on_tick = 180


    # Get the frame_index given start_at, frame_count, hold_for, and repeat
    sprite_index = Numeric.frame_index start_at: start_animation_on_tick,  # when to start the animation?
                                       frame_count: 7,                     # how many sprites?
                                       hold_for: 8,                        # how long to hold each sprite?
                                       repeat: true                        # should it repeat?

    # render the current tick and the resolved sprite index
    outputs.primitives  << sm_label.merge(x: 84 / 2,
                                          y: 48 - 10,
                                          text: "Tick: #{Kernel.tick_count}",
                                          anchor_x: 0.5)

    outputs.primitives  << sm_label.merge(x: 84 / 2,
                                          y: 48 - 20,
                                          text: "sprite_index: #{sprite_index || 'nil'}",
                                          anchor_x: 0.5)

    # Numeric.frame_index will return nil if the frame hasn't arrived yet
    if sprite_index
      # if the sprite_index is populated, use it to determine the sprite path and render it
      outputs.primitives << {
        x: 84 / 2 - 16,
        y: 0,
        w: 32,
        h: 32,
        path:  "sprites/explosion-sheet.png",
        source_x: 32 * sprite_index,
        source_y: 0,
        source_w: 32,
        source_h: 32
      }
    else
      # if the sprite_index is nil, render a countdown instead
      countdown_in_seconds = ((start_animation_on_tick - Kernel.tick_count) / 60).round(1)

      outputs.primitives  << sm_label.merge(x: 84 / 2,
                                      y: 48 / 2,
                                      text: "Count Down: #{countdown_in_seconds.to_sf}",
                                      anchor_x: 0.5,
                                      anchor_y: 0.5)
    end
  end

  def how_to_determine_collision
    # game state is stored in the state variable

    # Render the instructions
    if !state.ship_one
      # if game state's ship one isn't initialized, render the instructions to place ship one
      outputs.primitives << sm_label.merge(x: 42,
                                           y: 48 - 10,
                                           text: "CLICK: PLACE SHIP 1",
                                           anchor_x: 0.5)
    elsif !state.ship_two
      # if game state's ship one isn't initialized, render the instructions to place ship one
      outputs.primitives << sm_label.merge(x: 42,
                                           y: 48 - 10,
                                           text: "CLICK: PLACE SHIP 2",
                                           anchor_x: 0.5)
    else
      # otherwise, render the instructions to reset the ships
      outputs.primitives << sm_label.merge(x: 42,
                                           y: 48 - 10,
                                           text: "CLICK: RESET SHIPS",
                                           anchor_x: 0.5)
    end

    # if a mouse click occurs:
    # - set ship_one if it isn't set
    # - set ship_two if it isn't set
    # - otherwise reset ship one and ship two
    if inputs.mouse.click
      # is ship_one set?
      if !state.ship_one
        # set ship_one to the mouse position
        state.ship_one = { x: inputs.mouse.x - 5,
                           y: inputs.mouse.y - 5,
                           w: 10,
                           h: 10 }
      # is ship_one set?
      elsif !state.ship_two
        # set ship_two to the mouse position
        state.ship_two = { x: inputs.mouse.x - 5,
                           y: inputs.mouse.y - 5,
                           w: 10,
                           h: 10 }
      # should we reset?
      else
        state.ship_one = nil
        state.ship_two = nil
      end
    end

    # render ship one if it's set
    if state.ship_one
      # use Ruby's .merge method which is available on ~Hash~ to set the sprite
      # render ship one
      outputs.primitives << state.ship_one.merge(path: 'sprites/monochrome-ship.png')
    end

    if state.ship_two
      # use Ruby's .merge method which is available on ~Hash~ to set the sprite
      # render ship two
      outputs.primitives << state.ship_two.merge(path: 'sprites/monochrome-ship.png')
    end

    # if both ship one and ship two are set, then determine collision
    if state.ship_one && state.ship_two
      # collision is determined using the intersect_rect? method
      if Geometry.intersect_rect?(state.ship_one, state.ship_two)
        # if collision occurred, render the words collision!
        outputs.primitives << sm_label.merge(x: 84 / 2,
                                             y: 0,
                                             text: "Collision!",
                                             anchor_x: 0.5)
      else
        # if collision occurred, render the words no collision.
        outputs.primitives << sm_label.merge(x: 84 / 2,
                                             y: 0,
                                             text: "No Collision.",
                                             anchor_x: 0.5)
      end
    else
      # render overlay sprite
      outputs.primitives << { x: inputs.mouse.x - 5,
                              y: inputs.mouse.y - 5,
                              w: 10,
                              h: 10,
                              path: :solid,
                              r: 0,
                              g: 0,
                              b: 0,
                              a: 128 }

      # if both ship one and ship two aren't set, then render -- (waiting for input before collision can be determined)
      outputs.primitives << sm_label.merge(x: 84 / 2,
                                           y: 0,
                                           text: "--",
                                           anchor_x: 0.5)
    end
  end

  def how_to_create_buttons
    # Render instructions
    outputs.primitives << sm_label.merge(x: 84 / 2,
                                         y: 48 - 3,
                                         text: "Press a Button!",
                                         anchor_x: 0.5,
                                         anchor_y: 0.5)


    # Create button one using a border and a label
    state.button_one_border ||= { x: 1, y: 28, w: 82, h: 10 }
    outputs.borders << state.button_one_border
    outputs.primitives << sm_label.merge(x: state.button_one_border.x + state.button_one_border.w / 2,
                                         y: state.button_one_border.y + state.button_one_border.h / 2,
                                         anchor_x: 0.5,
                                         anchor_y: 0.5,
                                         text: "Button One")

    # Create button two using a border and a label
    state.button_two_border ||= { x: 1, y: 12, w: 82, h: 10 }
    outputs.borders << state.button_two_border
    outputs.primitives << sm_label.merge(x: state.button_two_border.x + state.button_two_border.w / 2,
                                         y: state.button_two_border.y + state.button_two_border.h / 2,
                                         anchor_x: 0.5,
                                         anchor_y: 0.5,
                                         text: "Button Two")

    # Initialize the state variable that tracks which button was clicked to "" (empty stringI
    state.last_button_clicked ||= "--"

    # If a click occurs, check to see if either button one, or button two was clicked
    # using the inside_rect? method of the mouse
    # set state.last_button_clicked accordingly
    if inputs.mouse.click
      if Geometry.inside_rect?(inputs.mouse, state.button_one_border)
        state.last_button_clicked = "Button One Clicked!"
      elsif Geometry.inside_rect?(inputs.mouse, state.button_two_border)
        state.last_button_clicked = "Button Two Clicked!"
      else
        state.last_button_clicked = "--"
      end
    end

    # Render the current value of state.last_button_clicked
    outputs.primitives << sm_label.merge(x: 84 / 2,
                                         y: 0,
                                         text: state.last_button_clicked,
                                         anchor_x: 0.5)
  end

  def shooter_game
    # render instructions
    outputs.primitives << sm_label.merge(x: 84 / 2,
                                         y: 0,
                                         text: "Move: WASD/ARROWS",
                                         anchor_y: 0,
                                         anchor_x: 0.5)

    outputs.primitives << sm_label.merge(x: 84 / 2,
                                         y: 0,
                                         text: "Space: Shoot",
                                         anchor_y: -1.0,
                                         anchor_x: 0.5)

    # initialize game state
    state.bullets ||= [] # array representing bullets
    state.targets ||= [] # array representing targets
    state.ship ||= { x: 0, y: 0, w: 10, h: 10 } # hash representing the ship

    # if space is pressed, add a bullet to the bullets array
    if inputs.keyboard.key_down.space
      state.bullets << {
        x: state.ship.x + state.ship.w / 2 - 1,
        y: state.ship.y + state.ship.h - 1,
        w: 2,
        h: 2
      }
    end

    # if a or left arrow is pressed/held, decrement the ships x position
    if inputs.keyboard.left
      state.ship.x -= 1
    end

    # if d or right arrow is pressed/held, increment the ships x position
    if inputs.keyboard.right
      state.ship.x += 1
    end

    # if s or down arrow is pressed/held, decrement the ships y position
    if inputs.keyboard.down
      state.ship.y -= 1
    end

    # if w or up arrow is pressed/held, increment the ships y position
    if inputs.keyboard.up
      state.ship.y += 1
    end

    # if there are no targets, add 10 targets to the targets array
    if state.targets.length == 0
      10.times do
        state.targets << {
          x: rand(70) + 10,
          y: rand(25) + 20,
          w: 3,
          h: 3
        }
      end
    end

    # move each bullet upwards
    state.bullets.each do |bullet|
      bullet.y += 1
    end

    # remove bullets that are off screen
    state.bullets.reject! do |bullet|
      bullet.y > 48
    end

    # for each bullet, check if it intersects with a target
    # if it does, remove the bullet and the target
    state.bullets.each do |bullet|
      state.targets.each do |target|
        if Geometry.intersect_rect?(bullet, target)
          state.bullets.delete bullet
          state.targets.delete target
        end
      end
    end

    # render the bullets
    outputs.primitives << state.bullets.map do |bullet|
      {
        x: bullet.x,
        y: bullet.y,
        w: bullet.w,
        h: bullet.h,
        path: :solid,
        r: 0,
        g: 0,
        b: 0
      }
    end

    # render the targets
    outputs.primitives << state.targets.map do |target|
      {
        x: target.x,
        y: target.y,
        w: target.w,
        h: target.w,
        path: :solid,
        r: 0,
        g: 0,
        b: 0
      }
    end

    # render the sprite to the screen using the position stored in state.ship
    outputs.primitives << {
      x: state.ship.x,
      y: state.ship.y,
      w: state.ship.w,
      h: state.ship.h,
      path: 'sprites/monochrome-ship.png',
      # parameters beyond this point are optional
      angle: 0, # Note: rotation angle is denoted in degrees NOT radians
      r: 0,
      g: 0,
      b: 0,
      a: 255
    }
  end

  def sm_label
    { x: 0, y: 0, size_px: 10, font: "tiny.ttf", anchor_x: 0, anchor_y: 0 }
  end

  def md_label
    { x: 0, y: 0, size_px: 20, font: "tiny.ttf", anchor_x: 0, anchor_y: 0 }
  end

  def lg_label
    { x: 0, y: 0, size_px: 30, font: "tiny.ttf", anchor_x: 0, anchor_y: 0 }
  end

  def xl_label
    { x: 0, y: 0, size_px: 40, font: "tiny.ttf", anchor_x: 0, anchor_y: 0 }
  end
end

DR.reset
