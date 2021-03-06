// See LICENSE file for copyright and license details.

use cgmath::vector::Vec2;
use core::types::{MInt, MapPos};

pub enum Dir {
  NorthEast,
  East,
  SouthEast,
  SouthWest,
  West,
  NorthWest,
}

static DIR_TO_POS_DIFF: [[MapPos, ..6], ..2] = [
    [
        Vec2{x: 1, y: -1},
        Vec2{x: 1, y: 0},
        Vec2{x: 1, y: 1},
        Vec2{x: 0, y: 1},
        Vec2{x: -1, y: 0},
        Vec2{x: 0, y: -1},
    ],
    [
        Vec2{x: 0, y: -1},
        Vec2{x: 1, y: 0},
        Vec2{x: 0, y: 1},
        Vec2{x: -1, y: 1},
        Vec2{x: -1, y: 0},
        Vec2{x: -1, y: -1},
    ]
];


impl Dir {
    pub fn from_int(n: MInt) -> Dir {
        assert!(n >= 0 && n < 6);
        let dirs = [
            NorthEast,
            East,
            SouthEast,
            SouthWest,
            West,
            NorthWest,
        ];
        dirs[n as uint]
    }

    pub fn to_int(&self) -> MInt {
        match *self {
            NorthEast => 0,
            East => 1,
            SouthEast => 2,
            SouthWest => 3,
            West => 4,
            NorthWest => 5,
        }
    }

    pub fn get_dir_from_to(from: MapPos, to: MapPos) -> Dir {
        // assert!(from.distance(to) == 1);
        let diff = to - from;
        for i in range(0 as MInt, 6) {
            if diff == DIR_TO_POS_DIFF[(from.y % 2) as uint][i as uint] {
                return Dir::from_int(i);
            }
        }
        fail!("impossible positions");
    }

    pub fn get_neighbour_pos(pos: MapPos, dir: Dir) -> MapPos {
        let is_odd_row = pos.y % 2 == 1;
        let subtable_index = if is_odd_row { 1 } else { 0 };
        let direction_index = dir.to_int();
        assert!(direction_index >= 0 && direction_index < 6);
        let difference = DIR_TO_POS_DIFF[subtable_index as uint][direction_index as uint];
        pos + difference
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
