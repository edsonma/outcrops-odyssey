# coding: utf-8
# Copyright 2019 DragonRuby LLC
# MIT License
# runtime.rb has been released under MIT (*only this file*).

# Contributors outside of DragonRuby who also hold Copyright:
# - Kevin Fischer: https://github.com/kfischer-okarin

module GTK
  class Runtime
    module Require
      def require file
        file_metadata = get_require_metadata file

        if !file_metadata[:file_exist]
          raise "#{file_metadata[:output_file]} does not exist."
        end

        if file_metadata[:type] == :rb || file_metadata[:type] == :rbc
          __require_rb_file__ file_metadata
        else
          __require_hlsl_file__ file_metadata
        end
      rescue Exception => e
        raise LoadError.new(file, e, "* ERROR: Exception while requiring #{file}.\n#{e}\n#{e.__backtrace_to_org__}")
      end

      def __require_rb_file__ file_metadata
        file = file_metadata[:output_file]

        if @debug_require
          log "REQUIRE: Started for =#{file}=.".indent(@require_stack.length, indent_char: "*", pad_line_with_space: true)
        end

        if file_metadata[:type] == :rb
          syntax = (@ffi_file.read file) || ''
          syntax_check_result = @ffi_mrb.parse syntax
          okay = (syntax_check_result == "Syntax OK")
          if !okay
            raise <<~S
          ** Failed to load #{file}.
          #{syntax_check_result}

          S
          else
            add_to_require_queue file
          end
        else # type == :rbc
          add_to_require_queue file
        end

        @required_files << file
        @required_files.uniq!
        __require_rb_sync__ file

        if @debug_require
          log "REQUIRE: Completed for =#{file}=.".indent(@require_stack.length, indent_char: "*", pad_line_with_space: true)
        end
      end

      def __require_hlsl_file__ file_metadata
        path = file_metadata[:output_file]
        assert_shaders_enabled!
        @required_files << path
        @required_files.uniq!
        return if @shader_metadata[path]
        compile_shader path
      end

      def assert_shaders_enabled!
        raise "* ERROR: Shaders are only available in the DragonRuby Pro License" if !version_pro?

        return if @args.cvars["game_metadata.shaders"].value

        raise <<-S
* ERROR: Shader compilation is not enabled.

Add the following to your metadata/game_metadata.txt:

  shaders=true
S
      end

      def get_require_metadata file
        if file.end_with?(".hlsl")
          __get_require_metadata_hlsl__ file
        else
          __get_require_metadata_rb__ file
        end
      end

      def __get_require_metadata_rb__ file
        file_path_with_rb_extension = if file.end_with?(".rb")
                                        file
                                      elsif file.end_with?(".rbc")
                                        file.gsub(".rbc", ".rb")
                                      else
                                        "#{file}.rb"
                                      end

        file_path_with_rbc_extension = "#{file_path_with_rb_extension}c"
        file_with_rb_extension_exist = @ffi_file.path_exist? file_path_with_rb_extension
        file_with_rbc_extension_exist = @ffi_file.path_exist? file_path_with_rbc_extension
        file_exist = file_with_rb_extension_exist || file_with_rbc_extension_exist
        type = if file_with_rbc_extension_exist
                 :rbc
               else
                 :rb
               end

        output_file = file_with_rbc_extension_exist ? file_path_with_rbc_extension : file_path_with_rb_extension

        {
          type: type,
          input_file: file,
          file_exist: file_exist,
          output_file: output_file,
        }
      end

      def __get_require_metadata_hlsl__ file
        {
          type: :hlsl,
          input_file: file,
          file_exist: @ffi_file.path_exist?(file),
          output_file: file,
        }
      end

      def require_relative file
        if @debug_require
          log "REQUIRE RELATIVE: Started for =#{file}=.".indent(@require_stack.length, indent_char: "*", pad_line_with_space: true)
        end

        current_require_file = @require_stack.last || "app/main.rb"
        current_require_path = File.dirname current_require_file
        current_require_path = '' if current_require_path == '.'
        full_path = File.join current_require_path, file
        require full_path

        if @debug_require
          log "REQUIRE RELATIVE: Completed for =#{file}=.".indent(@require_stack.length, indent_char: "*", pad_line_with_space: true)
        end
      end

      def __require_rb_sync__ path
        @require_history ||= {}
        return if @require_history[path] == Kernel.global_tick_count
        @require_history[path] = Kernel.global_tick_count
        @require_stack.push path
        @require_stack.uniq!
        reload_requested_ruby_files_synchronously
        @require_stack.pop
      end

      def require_init
        @reload_list = []

        # schema for reload_list_history
        # { PATH: { current: { path: PATH,
        #                      global_at: Fixnum,
        #                      event: (:reload_queued|:processing|reload_completed) },
        #           history: [{ path: PATH,
        #                             global_at: Fixnum,
        #                             event: (:reload_queued|:processing|reload_completed) }]}}
        @reload_list_history = {}

        @reload_debounce = 0
      end

      def most_recent_reload_history path
        return nil unless @reload_list_history[path]
        return nil unless @reload_list_history[path][:history]
        return @reload_list_history[path][:history].last
      end

      def add_to_require_queue path
        @reload_list_history[path] ||= { current: {}, history: [] }
        info = @reload_list_history[path]
        recent = (most_recent_reload_history path)

        return if recent && (((recent[:global_at] || 0) + 60) > Kernel.global_tick_count)
        return if info && info[:current] && info[:current][:event] && (((info[:global_at] || 0) + 60) > Kernel.global_tick_count)

        @reload_list_history[path][:current]   = { path: path, global_at: Kernel.global_tick_count, event: :reload_queued  }
        @reload_list_history[path][:history] ||= []
        @reload_list_history[path][:history]  << { path: path, global_at: Kernel.global_tick_count, event: :reload_queued  }

        log "** INFO: =#{path}= queued to load via ~require~. (#{Kernel.global_tick_count}, #{Kernel.tick_count})", subsystem="Engine"

        if @load_status == :ready
          @reload_list << path
          @reload_list.uniq!
          __require_rb_sync__ path
        else
          @reload_list << path
          @reload_list.uniq!
        end
      rescue Exception => e
        raise e, "* EXCEPTION: ~Runtime#add_to_require_queue~ failed for =#{path}=.\n#{e}"
      end

      def get_ruby_reload_list
        return [] if @reload_list.length == 0
        @reload_list.each do |r|
          @reload_list_history[r]           ||= {}
          @reload_list_history[r][:current]   = { path: r, global_at: Kernel.global_tick_count, event: :processing }
          @reload_list_history[r][:history] ||= []
          @reload_list_history[r][:history]  << { path: r, global_at: Kernel.global_tick_count, event: :processing }
        end
        @exception_occurred = false
        @is_reloading = true
        @reload_list
      end

      def reload_complete
        return unless @is_reloading
        @is_reloading = false

        if !@exception_occurred
          if @load_status == :main_rb_load_error_shown
            DR.reboot
            return
          else
            unpause!
            @console.hide if @console.show_reason == :exception || @console.show_reason == :exception_on_load
          end
        end

        @reload_list_history.keys.each do |k|
          if (@reload_list_history[k][:current][:event] == :processing) || (@reload_list_history[k][:current][:event] == :reload_queued)
            log "* INFO: =#{k}= reloaded. (#{Kernel.global_tick_count}, #{Kernel.tick_count})", subsystem="Engine"
            @reload_list_history[k][:current]  = { path: k, global_at: Kernel.global_tick_count, event: :reload_completed }
            @reload_list_history[k][:history] << { path: k, global_at: Kernel.global_tick_count, event: :reload_completed }
          end
        end

        @last_reload_complete_global_at = Kernel.global_tick_count
        $layout.reset if $layout
        $gtk.reset_framerate_calculation

        process_main_rb_load_status!
      end

      def on_file_reloaded file
      end

      def main_rb_reload_completed?
        return (@reload_list_history['app/main.rb'] &&
                @reload_list_history['app/main.rb'][:history] &&
                @reload_list_history['app/main.rb'][:history].find { |h| h[:event] == :reload_completed }) ||
               (@reload_list_history['app/main.rbc'] &&
                @reload_list_history['app/main.rbc'][:history] &&
                @reload_list_history['app/main.rbc'][:history].find { |h| h[:event] == :reload_completed })
      end

      def important_instance_methods
        Object.instance_methods + dollar_sign_game_methods
      end

      def dollar_sign_game_methods
        return [] if !$game
        return $game.class.instance_methods
      end

      def process_main_rb_load_status!
        if $main.respond_to?(:tick)
          @tick_method = $main.method(:tick)
        end

        return if @load_status == :ready || @load_status == :boot
        return if pending_reload?

        if main_rb_reload_completed?
          reset_all_mtimes
          @load_status = :boot
        end
      end

      def load_status
        @load_status
      end

      def pending_reload?
        return false if @load_status == :dragonruby_started
        @reload_list_history.any? do |key, value|
          value[:current][:event] == :reload_queued ||
          value[:current][:event] == :processing
        end
      end

      def reload_ruby_file file
        ext = File.extname(file)
        return false unless ext == ".rb" || ext == ".rbc"
        return true if @suppress_hotload
        Backup.backup_create file, ffi_file: @ffi_file, production: @production
        syntax = (@ffi_file.read file) || ''
        return true if syntax.strip.length == 0

        # this indicates that main.rb contained a syntax error when
        # first loaded and the dev updated main.rb (in an attempt to fix the syntax error).
        # set the Kernel.tick_count and global_tick_count to -1 which emulates a first time
        # start up of DR -> causes load_main_rb to be invoked.
        if @load_status == :main_rb_load_error_shown
          Kernel.tick_count = -1
          Kernel.global_tick_count = -1
          @load_status = :dragonruby_started
        end

        okay = true
        if ext == ".rb"
          syntax_check_result = @ffi_mrb.parse syntax
          okay = (syntax_check_result == "Syntax OK")
        end

        if okay
          add_to_require_queue file
          log_debug "* INFO - Reloaded #{file}. (#{Kernel.global_tick_count})", subsystem="Engine"
          $gtk.reset_framerate_calculation
          notify_subdued! if @global_notification_at != Kernel.global_tick_count
          return true
        else
          # handle a special case where a syntax error exists in main.rb on startup
          raise <<~S
                ** Failed to load/reload #{file}.
                #{syntax_check_result}

                S
        end
      rescue Exception => e
        pretty_print_exception_and_export! e
        pause!
        self.show_console :exception
        return false
      end

      def load_main_rb
        # @load_status flow is:
        # +-> :dragonruby_started
        # |     success -> :ready (if main.rb has no syntax errors and isn't missing)
        # |     failed  -> :main_rb_load_failed (main.rb has syntax errors or *is* missing)
        # |                :main_rb_load_error_shown (after exception is show which occurs internally in the Runtime)
        # +--------------- :dragonruby_started (load_status is reset to if a file is saved)
        return if @load_status != :dragonruby_started

        # the first load/boot is a little tricky
        # exceptions thrown in this phase need to be stored
        # and presented after Kernel.tick_count >= 0

        # if app/main.rb exists, check it's syntax
        # (if invalid then store the exception to be presented when
        #  Kernel.tick_count > 0)
        if @ffi_file.path_exist? 'app/main.rb'
          syntax = (@ffi_file.read 'app/main.rb') || ''
          syntax_check_result = @ffi_mrb.parse syntax
          syntax_passed = (syntax_check_result == "Syntax OK")

          if !syntax_passed
            @load_status = :main_rb_load_failed
            @load_status_exception = syntax_check_result
          end
        end

        # if either app/main.rb exists (with no syntax errors) or app/main.rbc exists,
        # then load it
        if @ffi_file.path_exist?('app/main.rb') || @ffi_file.path_exist?('app/main.rbc')
          begin
            require 'app/main.rb'
          rescue Exception => e
            @load_status = :main_rb_load_failed
            @load_status_exception = "#{e}"
          end
        else
          # if app/main.rb isn't found, then record that exception
          # so that it's presented when Kernel.tick_count > 0
          @load_status = :main_rb_load_failed
          @load_status_exception = "app/main.rb not found."
        end
      end
    end
  end
end

class Object
  def require file
    $gtk.require file
  end

  def require_relative file
    $gtk.require_relative file
  end
end
