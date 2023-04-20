# frozen_string_literal: true

require 'libfm'

screen = LibFM::Screen.new
viewport = LibFM::Viewport.new(screen, visible: true, decorations: true)
viewport.resize(800, 800)

sprite = LibFM::Sprite.new(viewport)
sprite.set('./examples/two_83c.png')

sprite2 = LibFM::Sprite.new(viewport)
sprite2.set('./examples/alula_gasp.png')

t = 0
loop do
  t += 1

  sprite.x = Math.sin(t / 30.0) * 240 + 240 #+ 1280 - 640
  sprite.y = Math.cos(t / 30.0) * 240 + 240 #+ 720 - 240

  sprite2.x = Math.sin(-t / 30.0) * 240 + 320 #+ 1280 - 640
  sprite2.y = Math.cos(-t / 30.0) * 240 + 320 #+ 720 - 24

  sleep(1.0 / 60.0)
end
