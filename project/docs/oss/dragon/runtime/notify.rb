# coding: utf-8
# Copyright 2019 DragonRuby LLC
# MIT License
# notify.rb has been released under MIT (*only this file*).

module GTK
  module Notify
    def notify message, duration = 300
      notify! message, duration
    end

    def notify! message, duration = 300
      return if self.production
      message ||= ""
      message = "#{message}"
      return if @notification_message == message
      return if @production
      @global_notification_at = Kernel.global_tick_count
      @notification_duration = duration
      @notification_message = message
      @console.add_text "* NOTIFY: #{message}" if message.strip.length > 0
    end

    def notify_subdued!
      notify! "", 60
    end

    def notify_extended! opts = {}
      message   = opts.message  || ""
      duration  = opts.duration || 300
      env       = opts.env      || :dev
      a         = opts.a        || 255
      overwrite = opts.overwrite
      return if @production && env != :prod
      return if !overwrite && @notification_message == message
      @global_notification_at = Kernel.global_tick_count
      @notification_duration = duration
      @notification_message = message
      @notification_max_alpha = a
      @console.add_text "* NOTIFY: #{message}" if message.strip.length > 0
    end

    alias notify_extended notify_extended!

    def notification_prefab message, alpha
      # aspect_size=32
      if Grid.aspect_size <= 32
        return [
          {
            x: Grid.left + 2,
            y: Grid.bottom + 2,
            w: 8,
            h: 8,
            path: 'console-logo.png',
            a: alpha
          }
        ]
      end

      font = "font.ttf"
      line_height = 1.0
      logo_y = @args.grid.bottom + 4
      icon_offset_x = 4
      icon_size = 32
      label_size_px = 20
      max_character_length = 72

      case [Grid.w, Grid.h]
      when [64, 64]
        font = "tiny.ttf"
        line_height = 0.7
        logo_y = @args.grid.bottom + 2
        icon_offset_x = 2
        icon_size = 12
        label_size_px = 10
        max_character_length = 12
      when [128, 128]
        font = "tiny.ttf"
        line_height = 0.7
        logo_y = @args.grid.bottom + 2
        icon_offset_x = 2
        icon_size = 20
        label_size_px = 10
        max_character_length = 26
      when [256, 256]
        font = "tiny.ttf"
        line_height = 0.70
        logo_y = @args.grid.bottom + 2
        icon_offset_x = 4
        icon_size = 24
        label_size_px = 10
        max_character_length = 54
      when [512, 512]
        font = "font.ttf"
        line_height = 1.0
        logo_y = @args.grid.bottom + 4
        icon_offset_x = 4
        icon_size = 40
        label_size_px = 20
        max_character_length = 52
      when [720, 720]
        font = "font.ttf"
        line_height = 1.0
        logo_y = @args.grid.bottom + 4
        icon_offset_x = 4
        icon_size = 40
        label_size_px = 20
        max_character_length = 72
      when [84, 48]
        font = "tiny.ttf"
        line_height = 0.7
        logo_y = @args.grid.bottom + 2
        icon_offset_x = 2
        icon_size = 12
        label_size_px = 10
        max_character_length = 18
      when [48, 84]
        font = "tiny.ttf"
        line_height = 0.7
        logo_y = @args.grid.bottom + 2
        icon_offset_x = 2
        icon_size = 12
        label_size_px = 10
        max_character_length = 8
      when [114, 64]
        font = "tiny.ttf"
        line_height = 0.7
        logo_y = @args.grid.bottom + 2
        icon_offset_x = 2
        icon_size = 12
        label_size_px = 10
        max_character_length = 22
      when [64, 114]
        font = "tiny.ttf"
        line_height = 0.7
        logo_y = @args.grid.bottom + 2
        icon_offset_x = 2
        icon_size = 12
        label_size_px = 10
        max_character_length = 11
      when [128, 72]
        font = "tiny.ttf"
        line_height = 0.7
        logo_y = @args.grid.bottom + 2
        icon_offset_x = 2
        icon_size = 12
        label_size_px = 10
        max_character_length = 26
      when [72, 128]
        font = "tiny.ttf"
        line_height = 0.7
        logo_y = @args.grid.bottom + 2
        icon_offset_x = 2
        icon_size = 12
        label_size_px = 10
        max_character_length = 13
      when [160, 90]
        font = "tiny.ttf"
        line_height = 0.7
        logo_y = @args.grid.bottom + 2
        icon_offset_x = 2
        icon_size = 12
        label_size_px = 10
        max_character_length = 32
      when [90, 160]
        font = "tiny.ttf"
        line_height = 0.7
        logo_y = @args.grid.bottom + 2
        icon_offset_x = 2
        icon_size = 12
        label_size_px = 10
        max_character_length = 16
      when [228, 128]
        font = "tiny.ttf"
        line_height = 0.7
        logo_y = @args.grid.bottom + 2
        icon_offset_x = 2
        icon_size = 12
        label_size_px = 10
        max_character_length = 50
      when [128, 228]
        font = "tiny.ttf"
        line_height = 0.7
        logo_y = @args.grid.bottom + 2
        icon_offset_x = 2
        icon_size = 12
        label_size_px = 10
        max_character_length = 26
      when [256, 144]
        font = "tiny.ttf"
        line_height = 0.7
        logo_y = @args.grid.bottom + 2
        icon_offset_x = 4
        icon_size = 24
        label_size_px = 10
        max_character_length = 54
      when [144, 256]
        font = "tiny.ttf"
        line_height = 0.7
        logo_y = @args.grid.bottom + 2
        icon_offset_x = 4
        icon_size = 24
        label_size_px = 10
        max_character_length = 26
      when [320, 180]
        font = "tiny.ttf"
        line_height = 0.7
        logo_y = @args.grid.bottom + 2
        icon_offset_x = 4
        icon_size = 24
        label_size_px = 10
        max_character_length = 70
      when [180, 320]
        font = "tiny.ttf"
        line_height = 0.7
        logo_y = @args.grid.bottom + 2
        icon_offset_x = 4
        icon_size = 24
        label_size_px = 10
        max_character_length = 32
      when [454, 256]
        font = "font.ttf"
        line_height = 1.0
        logo_y = @args.grid.bottom + 4
        icon_offset_x = 4
        icon_size = 40
        label_size_px = 20
        max_character_length = 40
      when [256, 454]
        font = "font.ttf"
        line_height = 1.0
        logo_y = @args.grid.bottom + 4
        icon_offset_x = 4
        icon_size = 40
        label_size_px = 20
        max_character_length = 20
      when [640, 360]
        font = "font.ttf"
        line_height = 1.0
        logo_y = @args.grid.bottom + 4
        icon_offset_x = 4
        icon_size = 40
        label_size_px = 20
        max_character_length = 64
      when [360, 640]
        font = "font.ttf"
        line_height = 1.0
        logo_y = @args.grid.bottom + 4
        icon_offset_x = 4
        icon_size = 40
        label_size_px = 20
        max_character_length = 32
      when [910, 512]
        font = "font.ttf"
        line_height = 1.0
        logo_y = @args.grid.bottom + 4
        icon_offset_x = 4
        icon_size = 40
        label_size_px = 20
        max_character_length = 94
      when [512, 910]
        font = "font.ttf"
        line_height = 1.0
        logo_y = @args.grid.bottom + 4
        icon_offset_x = 4
        icon_size = 40
        label_size_px = 20
        max_character_length = 48
      when [1024, 576]
        font = "font.ttf"
        line_height = 1.0
        logo_y = @args.grid.bottom + 4
        icon_offset_x = 4
        icon_size = 40
        label_size_px = 20
        max_character_length = 100
      when [576, 1024]
        font = "font.ttf"
        line_height = 1.0
        logo_y = @args.grid.bottom + 4
        icon_offset_x = 4
        icon_size = 40
        label_size_px = 20
        max_character_length = 56
      when [1280, 720]
        font = "font.ttf"
        line_height = 1.0
        logo_y = @args.grid.bottom + 4
        icon_offset_x = 4
        icon_size = 40
        label_size_px = 20
        max_character_length = 136
      when [720, 1280]
        font = "font.ttf"
        line_height = 1.0
        logo_y = @args.grid.bottom + 4
        icon_offset_x = 4
        icon_size = 40
        label_size_px = 20
        max_character_length = 72
      end

      if message.length == 0
        return {
          x: @args.grid.left + icon_offset_x,
          y: logo_y,
          w: icon_size,
          h: icon_size,
          path: 'console-logo.png',
          a: alpha
        }
      else
        # !!! FIXME: still kinda of jank
        message_lines = String.wrapped_lines message, max_character_length
        text_height = ((message_lines.length * line_height) * label_size_px).max(icon_size + 8)
        if message_lines.length == 1
          return [
            {
              x: @args.grid.left,
              y: args.grid.bottom,
              w: $grid.w,
              h: text_height,
              path: :solid,
              r: 0,
              g: 0,
              b: 0,
              a: alpha
            },
            message_lines.map_with_index do |s, i|
              [
                {
                  x: @args.grid.left + icon_size + icon_offset_x + 4,
                  y: logo_y + (text_height.idiv(2) * line_height).floor,
                  size_px: label_size_px,
                  text: s.lstrip,
                  font: font,
                  r: 255,
                  g: 255,
                  b: 255,
                  a: alpha,
                  anchor_y: 0.5,
                  anchor_x: 0
                }
              ]
            end,
            {
              x: @args.grid.left + icon_offset_x,
              y: logo_y + (text_height.idiv(2) * line_height).floor,
              w: icon_size,
              h: icon_size,
              path: 'console-logo.png',
              a: alpha,
              anchor_y: 0.5
            }
          ]
        else
          return [
            {
              x: @args.grid.left,
              y: args.grid.bottom,
              w: $grid.w,
              h: text_height + line_height * label_size_px,
              path: :solid,
              r: 0,
              g: 0,
              b: 0,
              a: alpha
            },
            message_lines.map_with_index do |s, i|
              [
                {
                  x: @args.grid.left + icon_size + icon_offset_x + 4,
                  y: text_height - line_height.ceil * 2,
                  size_px: label_size_px,
                  text: s.lstrip,
                  font: font,
                  r: 255,
                  g: 255,
                  b: 255,
                  a: alpha,
                  anchor_y: 0.5 + i * line_height,
                  anchor_x: 0
                }
              ]
            end,
            {
              x: @args.grid.left + icon_offset_x,
              y: logo_y + (text_height.idiv(2) * line_height).floor,
              w: icon_size,
              h: icon_size,
              path: 'console-logo.png',
              a: alpha,
              anchor_y: 0.5
            }
          ]
        end
      end
    end

    def tick_notification
      return if Kernel.tick_count <= -1
      @notification_max_alpha ||= 255
      @notification_message = nil if @console.visible?
      return if !@notification_message
      return if !@notification_duration
      return if !@global_notification_at
      if Kernel.global_tick_count > @global_notification_at + @notification_duration
        @notification_message = nil
        @global_notification_at = nil
        @notification_duration = nil
        return
      end

      fade_in_at = @global_notification_at
      hold_at = @global_notification_at + 15
      fade_out_at = @global_notification_at + @notification_duration - 15
      alpha = if Kernel.global_tick_count > fade_out_at
                Easing.smooth_start(start_at: fade_out_at,
                                    tick_count: Kernel.global_tick_count,
                                    duration: 15,
                                    flip: true) * 255
              elsif Kernel.global_tick_count > hold_at
                255
              else
                Easing.smooth_start(start_at: fade_in_at,
                                    tick_count: Kernel.global_tick_count,
                                    duration: 15) * 255
              end

      @args.outputs.reserved << notification_prefab(@notification_message, alpha)
    end
  end
end
