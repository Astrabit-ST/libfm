# frozen_string_literal: true

require 'libfm'

screen = LibFM::Screen.new
viewport = LibFM::Viewport.new(screen, visible: true)
viewport.resize(960, 540)

sprite = LibFM::Sprite.new(viewport)
sprite.set('./examples/two_83c.png')

sprite.x = 32
sprite.y = 32

loop do
  sleep(1)
end
