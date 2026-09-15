module Main
  def tick
    # How to set the background color
    outputs.background_color = [255, 255, 255]

    # ==== HELLO WORLD ======================================================
    # Steps to get started:
    # 1. ~def tick~ is the entry point for your game.
    # 2. There are quite a few code samples below, remove the "##"
    #    before each line and save the file to see the changes.
    # 3. 0,  0 is in bottom left and 63, 63 is in top right corner.
    # 4. Be sure to come to the discord channel if you need
    #    more help: [[http://discord.dragonruby.org]].
    #
    # Commenting and uncommenting code:
    # - Add a "#" infront of lines to comment out code
    # - Remove the "#" infront of lines to comment out code
    #
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


    # ==== HOW TO MOVE A SPRITE BASED OFF OF USER INPUT =====================
    # Remove the "#" at the beginning of the line below
    # how_to_move_a_sprite
    # =======================================================================


    # ==== HOW TO ANIMATE A SPRITE (SEPERATE PNGS) ==========================
    # Remove the "#" at the beginning of the line below
    # how_to_animate_a_sprite
    # =======================================================================


    # ==== HOW TO ANIMATE A SPRITE (SPRITE SHEET) ===========================
    # Remove the "#" at the beginning of the line below
    # how_to_animate_a_sprite_sheet
    # =======================================================================


    # ==== HOW TO DETERMINE COLLISION =============================================
    # Remove the "#" at the beginning of the line below
    # how_to_determine_collision
    # =======================================================================


    # ==== HOW TO CREATE BUTTONS ==================================================
    # Remove the "#" at the beginning of the line below
    # how_to_create_buttons
    # =======================================================================
  end

  def hello_world
    outputs.primitives << { x: 0, y: 64, w: 10, h: 10, path: :solid, r: 255 }

    outputs.primitives << {
      x: 32,
      y: 63,
      text: "lowrezjam 2020",
      size_px: 10, # size_px of 10 is a small font size, 20 is medium, 30 is large, 40 is extra large
      font: "tiny.ttf",
      anchor_x: 0.5,
      anchor_y: 0,
      r: 0,
      g: 0,
      b: 0,
      a: 255
    }

    outputs.primitives << {
      x: 32 - 10,
      y: 32 - 10,
      w: 20,
      h: 20,
      path: 'sprites/lowrez-ship-blue.png',
      a: Kernel.tick_count % 255,
      angle: Kernel.tick_count % 360
    }
  end


  # =======================================================================
  # ==== HOW TO RENDER A LABEL ============================================
  # =======================================================================
  def how_to_render_a_label
    # NOTE: Text is aligned from the TOP LEFT corner

    # Render an EXTRA LARGE/XL label (remove the "#" in front of each line below)
    outputs.primitives << { x: 0, y: 57, text: "Hello World",
                            size_px: 40, font: "tiny.ttf",
                            anchor_x: 0, anchor_y: 0,
                            r: 0, g: 0, b: 0, a: 255 }

    # Render a LARGE/LG label (remove the "#" in front of each line below)
    outputs.primitives << { x: 0, y: 36, text: "Hello World",
                            size_px: 30, font: "tiny.ttf",
                            anchor_x: 0, anchor_y: 0,
                            r: 0, g: 0, b: 0, a: 255 }

    # Render a MEDIUM/MD label (remove the "#" in front of each line below)
    outputs.primitives << { x: 0, y: 20, text: "Hello World",
                            size_px: 20, font: "tiny.ttf",
                            anchor_x: 0, anchor_y: 0,
                            r: 0, g: 0, b: 0, a: 255 }

    # Render a SMALL/SM label (remove the "#" in front of each line below)
    outputs.primitives << { x: 0, y: 9, text: "Hello World",
                            size_px: 10, font: "tiny.ttf",
                            anchor_x: 0, anchor_y: 0,
                            r: 0, g: 0, b: 0, a: 255 }

    # You can use the ~sm_label~ helper which returns a Hash that you
    # can ~merge~ properties with
    # Example 1
    outputs.primitives << sm_label.merge(text: "Default")

    # Example 2
    outputs.primitives << sm_label.merge(x: 31,
                                         text: "Default",
                                         r: 128,
                                         g: 128,
                                         b: 128)
  end

  ## # =============================================================================
  ## # ==== HOW TO RENDER FILLED SQUARES (SOLIDS) ==================================
  ## # =============================================================================
  def how_to_render_solids
    # Render a red square at 0, 0 with a width and height of 1
    outputs.primitives << { x: 0, y: 0, w: 1, h: 1, path: :solid, r: 255, g: 0, b: 0, a: 255 }

    # Render a red square at 1, 1 with a width and height of 2
    outputs.primitives << { x: 1, y: 1, w: 2, h: 2, path: :solid, r: 255, g: 0, b: 0, a: 255 }

    # Render a red square at 3, 3 with a width and height of 3
    outputs.primitives << { x: 3, y: 3, w: 3, h: 3, path: :solid, r: 255, g: 0, b: 0 }

    # Render a red square at 6, 6 with a width and height of 4
    outputs.primitives << { x: 6, y: 6, w: 4, h: 4, path: :solid, r: 255, g: 0, b: 0 }
  end

  ## # =============================================================================
  ## # == HOW TO RENDER A SPRITE ===================================================
  ## # =============================================================================
  def how_to_render_sprites
    # Loop 10 times and create 10 sprites in 10 positions
    # Render a sprite at the bottom left with a width and height of 5 and a path of 'sprites/lowrez-ship-blue.png'
    10.times do |i|
      outputs.primitives << {
        x: i * 5,
        y: i * 5,
        w: 5,
        h: 5,
        path: 'sprites/lowrez-ship-blue.png'
      }
    end

    # Given an array of positions create sprites
    positions = [
      { x: 10, y: 42 },
      { x: 15, y: 45 },
      { x: 22, y: 33 },
    ]

    positions.each do |position|
      # use Ruby's ~Hash#merge~ function to create a sprite
      outputs.primitives << position.merge(path: 'sprites/lowrez-ship-red.png',
                                           w: 5,
                                           h: 5)
    end
  end

  ## # =============================================================================
  ## # ==== HOW TO ANIMATE A SPRITE (SEPERATE PNGS) ==========================
  ## # =============================================================================
  def how_to_animate_a_sprite
    # STEP 1: Define when you want the animation to start. The animation in this case will start in 3 seconds
    start_animation_on_tick = 180

    # STEP 2: Get the frame_index given the start tick.
    sprite_index = Numeric.frame_index start_at: start_animation_on_tick,
                                       frame_count: 7,  # how many sprites?
                                       hold_for: 4,     # how long to hold each sprite?
                                       repeat: true     # should it repeat?

    # STEP 3: frame_index will return nil if the frame hasn't arrived yet
    if sprite_index
      # if the sprite_index is populated, use it to determine the sprite path and render it
      sprite_path  = "sprites/explosion-#{sprite_index}.png"
      outputs.primitives << { x: 0, y: 0, w: 64, h: 64, path: sprite_path }
    else
      # if the sprite_index is nil, render a countdown instead
      countdown_in_seconds = ((start_animation_on_tick - Kernel.tick_count) / 60).round(1)

      outputs.primitives << sm_label.merge(x: 32,
                                           y: 32,
                                           text: "Count Down: #{countdown_in_seconds}",
                                           anchor_x: 0.5)
    end

    # render the current tick and the resolved sprite index
    outputs.primitives << sm_label.merge(x: 4, y: 11, text: "Tick: #{Kernel.tick_count}")
    outputs.primitives << sm_label.merge(x: 4, y: 5,  text: "sprite_index: #{sprite_index}")
  end

  ## # =============================================================================
  ## # ==== HOW TO ANIMATE A SPRITE (SPRITE SHEET) =================================
  ## # =============================================================================
  def how_to_animate_a_sprite_sheet
    # STEP 1: Define when you want the animation to start. The animation in this case will start in 3 seconds
    start_animation_on_tick = 180

    # STEP 2: Get the frame_index given the start tick.
    sprite_index = Numeric.frame_index start_at: start_animation_on_tick,
                                       frame_count: 7,  # how many sprites?
                                       hold_for: 4,     # how long to hold each sprite?
                                       repeat: true     # should it repeat?

    # STEP 3: frame_index will return nil if the frame hasn't arrived yet
    if sprite_index
      # if the sprite_index is populated, use it to determine the source rectangle and render it
      outputs.primitives << {
        x: 0,
        y: 0,
        w: 64,
        h: 64,
        path:  "sprites/explosion-sheet.png",
        source_x: 32 * sprite_index,
        source_y: 0,
        source_w: 32,
        source_h: 32
      }
    else
      # if the sprite_index is nil, render a countdown instead
      countdown_in_seconds = ((start_animation_on_tick - Kernel.tick_count) / 60).round(1)

      outputs.primitives << sm_label.merge(x: 32,
                                           y: 32,
                                           text: "Count Down: #{countdown_in_seconds}",
                                           anchor_x: 0.5)
    end

    # render the current tick and the resolved sprite index
    outputs.primitives << sm_label.merge(x: 4, y: 11, text: "tick: #{Kernel.tick_count}")
    outputs.primitives << sm_label.merge(x: 4, y: 5,  text: "sprite_index: #{sprite_index}")
  end

  ## # =============================================================================
  ## # ==== HOW TO STORE STATE, ACCEPT INPUT, AND RENDER SPRITE BASED OFF OF STATE =
  ## # =============================================================================
  def how_to_move_a_sprite
    outputs.primitives << sm_label.merge(x: 32, y: 62, text: "Use Arrow Keys", anchor_x: 0.5)
    outputs.primitives << sm_label.merge(x: 32, y: 56, text: "Use WASD",       anchor_x: 0.5)
    outputs.primitives << sm_label.merge(x: 32, y: 50, text: "Or Click",       anchor_x: 0.5)

    # set the initial values for x and y using ||= ("or equal operator")
    state.ship ||= {
      x: 0, y: 0
    }

    # if a mouse click occurs, update the ship's x and y to be the location of the click
    if inputs.mouse.click
      state.ship.x = inputs.mouse.click.x
      state.ship.y = inputs.mouse.click.y
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

    # render the sprite to the screen using the position stored in state.ship
    outputs.primitives << {
      x: state.ship.x,
      y: state.ship.y,
      w: 5,
      h: 5,
      path: 'sprites/lowrez-ship-blue.png',
      # parameters beyond this point are optional
      angle: 0, # Note: rotation angle is denoted in degrees NOT radians
      r: 255,
      g: 255,
      b: 255,
      a: 255
    }
  end

  # =======================================================================
  # ==== HOW TO DETERMINE COLLISION =======================================
  # =======================================================================
  def how_to_determine_collision
    # Render the instructions
    outputs.primitives << sm_label.merge(x: 32, y: 62, text: "Click Anywhere", anchor_x: 0.5)

    # if a mouse click occurs:
    # - set ship_one if it isn't set
    # - set ship_two if it isn't set
    # - otherwise reset ship one and ship two
    if inputs.mouse.click
      # is ship_one set?
      if !state.ship_one
        state.ship_one = { x: inputs.mouse.click.x - 10,
                           y: inputs.mouse.click.y - 10,
                           w: 20,
                           h: 20 }
      # is ship_two set?
      elsif !state.ship_two
        state.ship_two = { x: inputs.mouse.click.x - 10,
                           y: inputs.mouse.click.y - 10,
                           w: 20,
                           h: 20 }
      # should we reset?
      else
        state.ship_one = nil
        state.ship_two = nil
      end
    end

    # render ship one if it's set
    if state.ship_one
      # use Ruby's .merge method which is available on ~Hash~ to set the sprite and alpha
      outputs.primitives << state.ship_one.merge(path: 'sprites/lowrez-ship-blue.png', a: 100)
    end

    if state.ship_two
      # use Ruby's .merge method which is available on ~Hash~ to set the sprite and alpha
      outputs.primitives << state.ship_two.merge(path: 'sprites/lowrez-ship-red.png', a: 100)
    end

    # if both ship one and ship two are set, then determine collision
    if state.ship_one && state.ship_two
      # collision is determined using the intersect_rect? method
      if Geometry.intersect_rect?(state.ship_one, state.ship_two)
        # if collision occurred, render the words collision!
        outputs.primitives << sm_label.merge(x: 31, y: 5, text: "Collision!",   anchor_x: 0.5)
      else
        # if no collision, render the words no collision.
        outputs.primitives << sm_label.merge(x: 31, y: 5, text: "No Collision.", anchor_x: 0.5)
      end
    else
      # if both ship one and ship two aren't set, then render --
      outputs.primitives << sm_label.merge(x: 31, y: 6, text: "--", anchor_x: 0.5)
    end
  end

  ## # =============================================================================
  ## # ==== HOW TO CREATE BUTTONS ==================================================
  ## # =============================================================================
  def how_to_create_buttons
    # Render instructions
    state.button_message ||= "Press a Button!"
    outputs.primitives << sm_label.merge(x: 32, y: 62,
                                         text: state.button_message,
                                         anchor_x: 0.5,
                                         r: 80, g: 80, b: 80)

    # Creates button one using a border and a label
    state.button_one_border = { x: 1, y: 32, w: 62, h: 10 }
    outputs.primitives << state.button_one_border.merge(r: 80, g: 80, b: 80).border!
    outputs.primitives << sm_label.merge(x: state.button_one_border.x + 2,
                                         y: state.button_one_border.y,
                                         text: "Button One",
                                         r: 80, g: 80, b: 80)

    # Creates button two using a border and a label
    state.button_two_border = { x: 1, y: 20, w: 62, h: 10 }
    outputs.primitives << state.button_two_border.merge(r: 80, g: 80, b: 80).border!
    outputs.primitives << sm_label.merge(x: state.button_two_border.x + 2,
                                         y: state.button_two_border.y,
                                         text: "Button Two",
                                         r: 80, g: 80, b: 80)

    # Initialize the state variable that tracks which button was clicked to "" (empty string)
    state.last_button_clicked ||= "--"

    # If a click occurs, check to see if either button one, or button two was clicked
    # using the inside_rect? method of the mouse
    # set state.last_button_clicked accordingly
    if inputs.mouse.click
      if Geometry.inside_rect?(inputs.mouse.click, state.button_one_border)
        state.last_button_clicked = "One Clicked!"
      elsif Geometry.inside_rect?(inputs.mouse.click, state.button_two_border)
        state.last_button_clicked = "Two Clicked!"
      else
        state.last_button_clicked = "--"
      end
    end

    # Render the current value of state.last_button_clicked
    outputs.primitives << sm_label.merge(x: 32, y: 5,
                                         text: state.last_button_clicked,
                                         anchor_x: 0.5,
                                         r: 80, g: 80, b: 80)
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
