// See LICENSE file for copyright and license details.

#![deny(non_camel_case_types)]
#![deny(non_uppercase_statics)]
#![deny(unnecessary_qualification)]
#![deny(unnecessary_typecast)]

extern crate native;
extern crate serialize;
extern crate collections;
extern crate time;
extern crate rand;
extern crate cgmath;
extern crate glfw;
extern crate gl;
extern crate stb_image;

use visualizer::visualizer::Visualizer;

mod core;
mod visualizer;

fn main() {
    let mut visualizer = Visualizer::new();
    while visualizer.is_running() {
        visualizer.tick();
    }
}

#[start]
fn start(argc: int, argv: **u8) -> int {
    native::start(argc, argv, main)
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
