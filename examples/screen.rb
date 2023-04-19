# frozen_string_literal: true

require 'libfm'

screen = LibFM::Screen.new
viewport = LibFM::Viewport.new(screen, visible: true)

loop do
  sleep(1)
end
