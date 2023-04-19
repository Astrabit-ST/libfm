// Copyright (C) 2023 Lily Lyons
//
// This file is part of libfm.
//
// libfm is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// libfm is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with libfm.  If not, see <http://www.gnu.org/licenses/>.

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub title: String,
    pub pos: Option<(i32, i32)>,
    pub visible: bool,
    pub size: (u32, u32),
    pub z: Option<i32>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub enum Message {}
