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
s.decoration true
s.title "TWO Good for you"

s2 = LibFM::Screen.new
s2.set "examples/alula_gasp.png"
s2.visible true

alula = true

t = 0
loop do
    t += 1
    s.move(
        Math.sin(t / 30.0) * 640 + 1280 - 640,
        Math.cos(t / 30.0) * 480 + 720 - 240
    )
    s2.move(
        Math.sin(-t / 30.0) * 640 + 1280 - 640,
        Math.cos(-t / 30.0) * 480 + 720 - 240 
    )

    if t % 240 == 0
        alula = !alula
        if alula
            s2.set "examples/alula_gasp.png"
        else
            s2.set "examples/niko_dizzy.png"
        end
        s.decoration alula
    end
    
    sleep(1.0 / 120.0)
end