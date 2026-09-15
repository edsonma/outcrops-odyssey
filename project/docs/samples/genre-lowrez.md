### Lowrez Hello World - main.rb
```ruby
  # ./samples/99_genre_lowrez/lowrez_hello_world/app/main.rb
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

```

### Lowrez Labels - main.rb
```ruby
  # ./samples/99_genre_lowrez/lowrez_labels/app/main.rb
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

```

### Nokia 3310 - main.rb
```ruby
  # ./samples/99_genre_lowrez/nokia_3310/app/main.rb
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

```

### Nokia 33snake - main.rb
```ruby
  # ./samples/99_genre_lowrez/nokia_3310_snake/app/main.rb
  module Main
    def tick
      outputs.background_color = [199, 240, 216]

      # create a new game on frame zero
      new_game if Kernel.tick_count == 0
      # calc game
      calc
      # render game
      render
      # increment the clock
      state.clock += 1
    end

    def calc
      calc_game
      calc_restart
    end

    def calc_game
      # return if the game is over
      return if state.game_over

      # return if the game is just starting
      return if state.clock < 30

      # begin capturing input after the initial countdown
      if inputs.keyboard.left && snake.direction.x == 0
        # if keyboard left is pressed or held, and
        # if the snake is not moving left or right,
        # set the next direction to left
        snake.next_direction = { x: -1, y: 0 }
        snake.next_angle = 180
      elsif inputs.keyboard.right && snake.direction.x == 0
        # if keyboard right is pressed or held, and
        # if the snake is not moving left or right,
        # set the next direction to right
        snake.next_direction = { x: 1, y: 0 }
        snake.next_angle = 0
      end

      if inputs.keyboard.up && snake.direction.y == 0
        # if keyboard up is pressed or held, and
        # if the snake is not moving up or down,
        # set the next direction to up
        snake.next_direction = { x: 0, y: 1 }
        snake.next_angle = 90
      elsif inputs.keyboard.down && snake.direction.y == 0
        # if keyboard down is pressed or held, and
        # if the snake is not moving up or down,
        # set the next direction to down
        snake.next_direction = { x: 0, y: -1 }
        snake.next_angle = 270
      end

      # return if the game is in the initial countdown
      return if state.clock < 60

      # process the movement of the snake every 15 frames
      return if !state.clock.zmod?(15)

      # add a new segment to the end of the snake
      snake.body.push_back({ x: snake.head.x, y: snake.head.y })

      # update the snake's direction based on what input was captured
      snake.direction = { **snake.next_direction }

      # update the snake's angle based on what input was captured (for rendering)
      snake.angle = snake.next_angle

      # update the snake's head position based on its direction
      snake.head = { x: snake.head.x + snake.direction.x,
                     y: snake.head.y + snake.direction.y }

      # check if the snake has collided with the world boundaries
      if snake.head.x < 0 || snake.head.x >= state.world_dimensions.w ||
         snake.head.y < 0 || snake.head.y >= state.world_dimensions.h
        state.game_over = true
        state.game_over_at = state.clock
      end

      # check if the snake has collided with itself
      if snake.body.include?(snake.head)
        state.game_over = true
        state.game_over_at = state.clock
      end

      # if the snake body is longer than the snake size
      # remove the first segment of the snake body
      if snake.body.length > snake.sz
        snake.body.pop_front
      end

      # check if the snake has eaten the apple
      if snake.head.x == state.apple.x && snake.head.y == state.apple.y
        # increase the snake size
        snake.sz += 1
        # increase the score
        state.score += 1
        # check if the score is higher than the high score
        # and update the high score if necessary
        state.high_score = state.score if state.score > state.high_score
        # generate a new apple
        state.apple = new_apple
      end
    end

    def calc_restart
      # check keyboard input to see if game should be restarted
      # wait 60 frames after game over before accepting input
      return if !state.game_over
      return if state.game_over_at.elapsed_time(state.clock) < 60

      # if any key is pressed, start a new game
      if inputs.keyboard.key_down.truthy_keys.any?
        new_game
      end
    end

    def render
      # render the main game
      render_game
      # render the game over screen if needed
      render_game_over
    end

    def render_game
      # render the snake's head
      outputs.primitives << {
        x: snake.head.x * 3,
        y: snake.head.y * 3,
        w: 3,
        h: 3,
        path: "sprites/head.png",
        angle: snake.angle
      }

      # render the snake's body
      outputs.primitives << snake.body.map do |segment|
        {
          x: segment.x * 3,
          y: segment.y * 3,
          w: 3,
          h: 3,
          path: "sprites/body.png"
        }
      end

      # render the apple
      outputs.primitives << {
        x: state.apple.x * 3,
        y: state.apple.y * 3,
        w: 3,
        h: 3,
        path: "sprites/apple.png"
      }
    end

    def render_game_over
      # return if the game is not over
      return if !state.game_over

      # wait 60 frames after game over before rendering the game over screen/overlay
      return if state.game_over_at.elapsed_time(state.clock) < 60

      # render background
      outputs.primitives << {
        x: 84 / 2, y: 48 / 2, w: 64, h: 30, path: :solid, r: 67, g: 82, b: 61,
        anchor_x: 0.5, anchor_y: 0.5
      }

      # render game over text
      outputs.primitives << sm_label.merge(x: 84 / 2,
                                           y: 48 / 2,
                                           r: 199, g: 240, b: 216,
                                           text: "GAME OVER",
                                           anchor_x: 0.5,
                                           anchor_y: -0.5)

      # render score text
      outputs.primitives << sm_label.merge(x: 84 / 2,
                                           y: 48 / 2,
                                           r: 199, g: 240, b: 216,
                                           text: "SCORE: #{state.score}",
                                           anchor_x: 0.5,
                                           anchor_y: 0.5)

      # render high score text
      outputs.primitives << sm_label.merge(x: 84 / 2,
                                           y: 48 / 2,
                                           r: 199, g: 240, b: 216,
                                           text: "HI SCORE: #{state.high_score}",
                                           anchor_x: 0.5,
                                           anchor_y: 1.5)
    end

    def snake
      # helper function to access the snake state so we aren't writing state.snake everywhere
      state.snake
    end

    def new_game
      # initial state for a new game
      state.clock = 0
      state.world_dimensions = { w: 28, h: 16 }
      state.snake = {
        sz: 3,
        head: { x: 14, y: 8 },
        body: [],
        direction: { x: 1, y: 0 },
        next_direction: { x: 1, y: 0 },
        angle: 0,
        next_angle: 0
      }
      state.high_score ||= 0
      state.score = 0
      state.apple = new_apple
      state.game_over = false
      state.game_over_at = nil
    end

    def new_apple
      # pick a random location for the apple
      potential_apple = { x: Numeric.rand(0..state.world_dimensions.w - 1),
                          y: Numeric.rand(0..state.world_dimensions.h - 1) }

      if snake.body.include?(potential_apple) || state.snake.head == potential_apple
        # if the apple is on the snake or in the snake's head, pick a new location
        new_apple
      else
        # otherwise, return the apple
        potential_apple
      end
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

```

### Platformer 128x128 - main.rb
```ruby
  # ./samples/99_genre_lowrez/platformer_128x128/app/main.rb
  class Game
    attr :camera, :player, :clock

    FREEZE_TICKS = 4

    def initialize rooms
      @room_index = 0
      @rooms = rooms
      @player = Player.new player_starting_point.ordinal_x,
                           player_starting_point.ordinal_y
      @camera = {
        trauma: 0,
        x_offset: 0,
        y_offset: 0
      }
      @clock = 0
      @freeze_ticks = 0
    end

    def walls
      @rooms[@room_index].walls
    end

    def tiles
      @rooms[@room_index].tiles
    end

    def spikes
      @rooms[@room_index].spikes
    end

    def player_starting_point
      @rooms[@room_index].player
    end

    def tick inputs
      if @freeze_ticks > 0
        @freeze_ticks -= 1
        tick_camera
        return
      end

      load_next_room!

      spike = Geometry.find_intersect_rect @player, spikes, using: :rect
      if spike || @player.y < -8
        kill_and_reset_player!
      end

      @player.tick inputs, walls, @clock

      if @player.dash_at == @clock
        @camera.trauma = 0.1
        @freeze_ticks = FREEZE_TICKS
      end

      if @player.spawned_at && @player.on_ground_at == @clock
        @player.spawned_at = nil
        @camera.trauma = 0.2
      end

      @clock += 1
      tick_camera
    end

    def load_next_room!
      return if @player.y <= 128
      @room_index += 1
      if @room_index >= @rooms.length
        @room_index = 0
      end
      @player.reset player_starting_point.ordinal_x,
                    player_starting_point.ordinal_y,
                    @clock
    end

    def kill_and_reset_player!
      @camera.trauma = 0.2
      @player.reset player_starting_point.ordinal_x,
                    player_starting_point.ordinal_y,
                    @clock
    end

    def tick_camera
      next_offset       = 100.0 * @camera.trauma**2
      @camera.x_offset = next_offset.randomize(:sign, :ratio)
      @camera.y_offset = next_offset.randomize(:sign, :ratio)
      @camera.trauma *= 0.95
    end

    def primitives
      [
        room_primitives,
        @player.primitives
      ]
    end

    def room_primitives
      tiles.map do |h|
        color = if h.type == :wall
                  { r: 30, g: 128, b: 255, a: 255 }
                elsif h.type == :spike
                  { r: 255, g: 128, b: 30, a: 255 }
                else
                  { r: 0, g: 0, b: 0, a: 0 }
                end

        {
          **h.rect,
          path: :solid,
          **color
        }
      end
    end
  end

  class Player
    attr :x, :y, :w, :h, :dead, :on_ground, :on_ground_grace,
         :has_dashed, :facing_x, :dx, :dy, :jump_power, :dash_power, :gravity,
         :dash_at, :spawned_at, :on_ground_at

    WALL_KICK_LOCK = 12
    DASH_FRAMES  = 12
    DASH_SUSTAIN = 2
    DASH_ACCEL   = 0.25
    APEX_SPEED   = 0.15

    def initialize ordinal_x, ordinal_y
      reset ordinal_x, ordinal_y, -1
    end

    def reset ordinal_x, ordinal_y, clock
      @reset_at = clock + 1
      @ordinal_x = ordinal_x
      @ordinal_y = ordinal_y
      @max_dx = 1
      @facing_x = 1
      @dx = 0
      @jump_power = 2
      @dash_power = 2.5
      @gravity = 0.10
      @maxfall = 2
      @dash_target_dx = 0
      @dash_target_dy = 0
      @dash_accel_x = DASH_ACCEL
      @dash_accel_y = DASH_ACCEL
      @x = @ordinal_x * 8
      @y = -8
      @on_ground = false
      @on_ground_at = nil
      @on_ground_grace = 6
      @w = 8
      @h = 8
      @dy = 0
      @dash_ticks = 0
      @wall_kick_ticks = 0
    end

    def rect
      {
        x: @x,
        y: @y,
        w: 8,
        h: 8,
      }
    end

    def solid_at? walls, ox, oy
      probe = { rect: { x: @x + ox, y: @y + oy, w: @w, h: @h } }
      Geometry.find_intersect_rect probe, walls, using: :rect
    end

    def tick inputs, walls, clock
      if @reset_at
        tick_respawn walls, clock
        return
      end

      collide_x! walls, clock
      collide_y! walls, clock

      if @dash_ticks > 0
        @dash_ticks -= 1
        @dx = @dx.towards(@dash_target_dx, @dash_accel_x)
        @dy = @dy.towards(@dash_target_dy, @dash_accel_y)
        @dash_ticks = @dash_ticks.clamp(0, DASH_FRAMES)
      else
        dx! inputs
        dy! inputs, walls
      end

      if dash_pressed?(inputs) && @dash_ticks <= 0 && !@dash_at
        dash! inputs, clock
      elsif jump_pressed?(inputs)
        jump! inputs, walls, clock
      end
    end

    def tick_respawn walls, clock
      if @reset_at == clock - 1
        @y = -8
        @dy = @jump_power
      end

      if @y > (@ordinal_y - 1.5) * 8
        g = @gravity
        @dy = @dy.towards(-@maxfall, g)
      end

      if @y > @ordinal_y * 8
        @reset_at = nil
        @spawned_at = clock
      end

      if @reset_at
        @y += @dy
      end
    end

    def collide_x! walls, clock
      @x += @dx
      collision = Geometry.find_intersect_rect self, walls, using: :rect
      if collision
        if @dx > 0
          @x = collision.rect.x - @w
        elsif @dx < 0
          @x = collision.rect.x + collision.rect.w
        end
        @dx = 0
      end

      if @x < 0
        @x = 0
      elsif @x + @w > 128
        @x = 128 - w
      end
    end

    def collide_y! walls, clock
      @y += @dy
      collision = Geometry.find_intersect_rect self, walls, using: :rect
      if collision
        if @dy > 0
          @y = collision.rect.y - @h
        elsif @dy < 0
          @y = collision.rect.y + collision.rect.h
          @on_ground = true
          @on_ground_at = clock
          @dash_ticks = 0
          @dash_at = nil
          @on_ground_grace = 6
        end
        @dy = 0
      elsif @on_ground
        lower_player = {
          rect: { x: @x, y: @y - 4, w: 8, h: 8 }
        }
        collision = Geometry.find_intersect_rect lower_player, walls, using: :rect
        if !collision
          @on_ground_grace -= 1
          if @on_ground_grace < 0
            @on_ground = false
            @on_ground_at = nil
          end
        end
      end
    end

    def dx! inputs
      if !@spawned_at
        if @wall_kick_ticks > 0
          @wall_kick_ticks -= 1
        else
          accel = 0.489
          deccel = 0.3
          if inputs.left_right == 0
            @dx = @dx.lerp(0, deccel)
          else
            @dx = @dx.lerp(@max_dx * inputs.left_right, accel)
          end
        end
      end

      @facing_x = @dx < 0 ? -1 : 1 if @dx != 0
    end

    def dy! inputs, walls
      maxfall = @maxfall
      slide_dir = inputs.left_right
      if slide_dir != 0 && !@on_ground && solid_at?(walls, slide_dir, 0)
        maxfall = 0.4
      end
      g = @gravity
      g *= 0.5 if @dy.abs <= APEX_SPEED
      @dy = @dy.towards(-maxfall, g)
    end

    def jump_pressed? inputs
      inputs.keyboard.key_down.j || inputs.controller_one.key_down.s
    end

    def dash_pressed? inputs
      inputs.keyboard.key_down.k || inputs.controller_one.key_down.e
    end

    def dash! inputs, clock
      @dash_ticks = DASH_FRAMES
      @dash_at = clock
      dv = inputs.directional_vector || { x: @facing_x, y: 0 }
      @dx = dv.x * @dash_power
      @dy = dv.y * @dash_power
      @dash_target_dx = DASH_SUSTAIN * @dx.sign
      @dash_target_dy = DASH_SUSTAIN * @dy.sign
      @dash_accel_x = DASH_ACCEL
      @dash_accel_y = DASH_ACCEL
      @dash_target_dy *= 0.5 if @dy > 0
      @dash_accel_x *= 0.7071 if @dy != 0
      @dash_accel_y *= 0.7071 if @dx != 0
    end

    def jump! inputs, walls, clock
      return if @dash_ticks > 0
      if @on_ground
        @on_ground = false
        @dy = @jump_power
      else
        wall_dir = if    solid_at?(walls, -3, 0) then -1
                   elsif solid_at?(walls,  3, 0) then  1
                   else 0
                   end
        if wall_dir != 0
          @dy = @jump_power
          @dx = -wall_dir * (@max_dx + 1)
          @facing_x = -wall_dir
          @wall_kick_ticks = WALL_KICK_LOCK
        end
      end
    end

    def primitives
      {
        **rect,
        path: :solid,
        r: 30, g: 255, b: 128
      }
    end
  end

  module Main
    def start
      @game = Game.new parse_rooms
    end

    def tick
      @game.tick inputs
      outputs.background_color = [30, 30, 30]
      outputs[:scene].background_color = [30, 30, 30]
      outputs[:scene].primitives << @game.primitives
      outputs.primitives << {
        x: 0 - @game.camera.x_offset,
        y: 0 - @game.camera.y_offset,
        w: 128,
        h: 128,
        path: :scene,
      }
    end

    def parse_rooms
      ascii_rooms.map do |ascii_room|
        parse_room ascii_room
      end
    end

    def parse_room ascii_room
      symbol_room = ascii_room.strip
                              .split("\n")
                              .reverse
                              .map
                              .with_index do |line, row|
                                line.chars.map do |char, col|
                                  case char
                                  when "0"
                                    :wall
                                  when "X"
                                    :spike
                                  when "P"
                                    :player
                                  else
                                    :empty
                                  end
                                end
                              end

      tiles = []
      walls = []
      spikes = []
      player = nil
      symbol_room.each_with_index do |xs, y|
        xs.each_with_index do |t, x|
          if t == :player
            player = { ordinal_x: x, ordinal_y: y }
          elsif t != :empty
            entry = {
              ordinal_x: x,
              ordinal_y: y,
              type: t,
              rect: {
                x: x * 8,
                y: y * 8,
                w: 8,
                h: 8
              },
            }
            tiles << entry
            if t == :wall
              walls << entry
            elsif t == :spike
              spikes << entry
            end
          end
        end
      end

      {
        player: player,
        tiles: tiles,
        walls: walls,
        spikes: spikes
      }
    end

    def ascii_rooms
      [
        <<~S,
        0000000000000  0
        0000000        0
        0000           0
        000          000
        000         0000
        000           00
        00000          0
        00             0
        0              0
        0            000
                      00
               00     00
         P 00  00  XXX00
        00000  00XX00000
        00000XX000000000
        0000000000000000
        S
        <<~S
        000000000000   0
        000000    00   0
        0000      00   0
        000       00   0
        000       0    0
        000    X  0    0
        0      0      00
        0000   0      00
        0000   0      00
        0000   0      00
        0000   0     000
        0000   0       0
          00   0       0
           0   0XXX
         P     0000
        00000000000
        S
      ]
    end
  end

  # DR.reset_and_replay speed: 2
  DR.reset

```

### Resolution 64x64 - main.rb
```ruby
  # ./samples/99_genre_lowrez/resolution_64x64/app/main.rb
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

```

### Resolution 64x64 With Touch Controls - main.rb
```ruby
  # ./samples/99_genre_lowrez/resolution_64x64_with_touch_controls/app/main.rb
  # sample app shows how to display on-screen touch controls with a 64x64 canvas
  # see ./metadata/game_metadata.txt for aspect_size and aspect_mode configuration.

  module Main
    def start
      @on_screen_gamepad = OnScreenGamepad.new inputs, neutral_pos: { x: 14, y: 25 }

      @player = { x: 32 - 8, y: 32 - 8, w: 16, h: 16, r: 80, g: 255, b: 128, path: :solid, angle: 0 }

      @touch_a_button = Geometry.rect(x: 64 - 8 - 8 - 8 - 10,
                                      y: 25 - 14,
                                      w: 16,
                                      h: 16)

      @touch_b_button = Geometry.rect(x: 64 - 8 - 10,
                                      y: 25 - 2,
                                      w: 16,
                                      h: 16)
    end

    def tick
      @on_screen_gamepad.tick

      # outputs.watch "#{@on_screen_gamepad.dpad_vector&.x}", font: "tiny.ttf", size_px: 10
      # outputs.watch "#{@on_screen_gamepad.dpad_vector&.y}", font: "tiny.ttf", size_px: 10

      if @on_screen_gamepad.dpad_vector
        @player.x += @on_screen_gamepad.dpad_vector.x
        @player.y += @on_screen_gamepad.dpad_vector.y
      else
        @player.x += inputs.left_right
        @player.y += inputs.up_down
      end


      @player.x = @player.x.clamp(0, 64 - @player.w)
      @player.y = @player.y.clamp(0, 64 - @player.h)

      if spin_left?
        @player.angle += 2
      elsif spin_right?
        @player.angle -= 2
      end

      # game will be rendered to a Render Target with size 64x64
      outputs[:lowrez].set w: 64, h: 64, background_color: [30, 30, 30]
      outputs[:lowrez].primitives << @player

      # the render target will be positioned at the top of the portrait
      # screen
      outputs.primitives << {
        x: 0,
        y: Grid.top - 64,
        w: 64,
        h: 64,
        path: :lowrez
      }

      # touch controls will be positioned at the bottom
      outputs.primitives << button_primitives(@touch_a_button, "A", is_down: spin_left?)
      outputs.primitives << button_primitives(@touch_b_button, "B", is_down: spin_right?)
      outputs.primitives << @on_screen_gamepad.primitives
      if inputs.finger_left
        outputs.primitives << { x: inputs.finger_left.x, y: inputs.finger_left.y, w: 16, h: 16, path: :solid, r: 0, g: 0, b: 255, anchor_x: 0.5, anchor_y: 0.5, a: 128 }
      end

      if inputs.finger_right
        outputs.primitives << { x: inputs.finger_right.x, y: inputs.finger_right.y, w: 16, h: 16, path: :solid, r: 255, g: 0, b: 0, anchor_x: 0.5, anchor_y: 0.5, a: 128  }
      end
    end

    def button_primitives rect, label, is_down: false
      button_color = if is_down
                       { r: 0, g: 0, b: 0, a: 255 }
                     else
                       { r: 0, g: 0, b: 0, a: 32 }
                     end
      [
        rect.merge(path: "sprites/circle/solid.png",
                   **button_color),
        rect.center
            .merge(text: label,
                   font: "tiny.ttf",
                   size_px: 10,
                   r: 255, g: 255, b: 255,
                   anchor_x: 0.5, anchor_y: 0.5)
      ]
    end

    def spin_left?
      if inputs.last_active == :mouse
        return inputs.finger_right && Geometry.intersect_rect?(inputs.finger_right, @touch_a_button)
      else
        return inputs.keyboard.h || inputs.controller_one.south
      end
    end

    def spin_right?
      if inputs.last_active == :mouse
        return inputs.finger_right && Geometry.intersect_rect?(inputs.finger_right, @touch_b_button)
      else
        return inputs.keyboard.l || inputs.controller_one.east
      end
    end
  end

  class OnScreenGamepad
    attr :inputs, :directional_vector, :directional_angle, :dpad_vector

    def initialize inputs, side: :left, neutral_pos: nil
      @neutral_pos = neutral_pos
      @side = side
      @inputs = inputs
      if @neutral_pos
        @joystick = {
          center: @neutral_pos,
          a: 32
        }
      end
    end

    def tick
      if finger
        @joystick ||= {
          center: @neutral_pos || { x: finger.x, y: finger.y },
          a: 32
        }
        @joystick.distance = Geometry.distance(finger, @joystick.center)
        @joystick.angle = Geometry.angle(finger, @joystick.center)
        @joystick.vector = @joystick.angle.to_vector
        if @joystick.distance > 160
          @joystick.center.x = @joystick.center.x.lerp(finger.x + @joystick.vector.x * 160, 0.1)
          @joystick.center.y = @joystick.center.y.lerp(finger.y + @joystick.vector.y * 160, 0.1)
        end

        perc = @joystick.distance.clamp(0, 48).fdiv(48)
        @directional_angle = (@joystick.angle + 180) % 360

        @directional_vector = {
          x: @directional_angle.to_vector.x * perc ** 4 * 10,
          y: @directional_angle.to_vector.y * perc ** 4 * 10,
        }

        if perc > 0.1
          @dpad_vector = {
            x: Geometry.angle_cardinal_vec2(@directional_angle).x,
            y: Geometry.angle_cardinal_vec2(@directional_angle).y,
          }
        else
          @dpad_vector = nil
        end

        @joystick.a = @joystick.a.lerp(128, 0.01)
      elsif @joystick
        @joystick.a = @joystick.a.lerp(32, 0.25)
        @directional_vector = nil
        @dpad_vector = nil
        @joystick = nil if @joystick.a < 1
      end
    end

    def joystick_primitives
      center = if @joystick
                 @joystick.center
               elsif @neutral_pos
                 @neutral_pos
               else
                 nil
               end

      return nil if !center

      [
        {
          **center,
          w: 24,
          h: 24,
          path: "sprites/circle/solid.png",
          anchor_x: 0.5,
          anchor_y: 0.5,
          r: 30,
          g: 30,
          b: 30,
          a: @joystick.a - 16
        },
        {
          **center,
          text: "ʘ",
          font: "tiny.ttf",
          size_px: 60,
          anchor_x: 0.5,
          anchor_y: 0.5,
          r: 0,
          g: 0,
          b: 0,
          a: @joystick.a
        },
      ]
    end

    def finger
      if @side == :left
        inputs.finger_left
      else
        inputs.finger_right
      end
    end

    def direction_indicator_primitives
      return nil if !finger
      return nil if !@joystick

      joystick_center = if @joystick.distance > 48
                          { x: @joystick.center.x + 48 * -@joystick.vector.x,
                            y: @joystick.center.y + 48 * -@joystick.vector.y }
                        else
                          finger
                        end

      {
        x: joystick_center.x,
        y: joystick_center.y,
        w: 4,
        h: 4,
        path: "sprites/circle/solid.png",
        r: 255,
        g: 0,
        b: 0,
        a: @joystick.a,
        anchor_x: 0.5,
        anchor_y: 0.5
      }
    end

    def primitives
      [
        joystick_primitives,
        direction_indicator_primitives
      ]
    end
  end

  DR.reset

```
