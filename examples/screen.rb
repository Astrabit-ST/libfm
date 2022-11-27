# Copyright (C) 2022 Lily Lyons
# 
# This file is part of libfm.
# 
# libfm is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
# 
# libfm is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
# 
# You should have received a copy of the GNU General Public License
# along with libfm.  If not, see <http://www.gnu.org/licenses/>.

require "libfm"

s = LibFM::Screen.new
s.set "examples/two_83c.png"
s.visible true

t = 0
loop do
    t += 0.05
    s.move(
        Math.sin(t) * 640 + 1280 - 640,
        Math.cos(t) * 480 + 720 - 240
    )
    sleep(1.0 / 60.0)
end