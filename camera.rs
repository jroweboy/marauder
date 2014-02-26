// See LICENSE file for copyright and license details.

use std::num::{
    sin,
    cos,
};
use cgmath::projection;
use cgmath::angle;
use cgmath::matrix::Mat4;
use cgmath::vector::Vec3;
use gl_helpers::{
    tr,
    rot_x,
    rot_z
};
use misc::deg_to_rad;
use gl_types::{
    Float,
    WorldPos,
};

pub struct Camera {
    x_angle: Float,
    z_angle: Float,
    pos: WorldPos,
    zoom: Float,
    projection_mat: Mat4<Float>,
}

fn get_projection_mat() -> Mat4<Float> {
    let fov = angle::deg(45.0 as Float);
    let ratio = 4.0 / 3.0;
    let display_range_min = 0.1;
    let display_range_max = 100.0;
    projection::perspective(
        fov, ratio, display_range_min, display_range_max)
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            x_angle: 45.0,
            z_angle: 0.0,
            pos: Vec3::zero(),
            zoom: 10.0,
            projection_mat: get_projection_mat(),
        }
    }

    pub fn mat(&self) -> Mat4<Float> {
        let mut m = self.projection_mat;
        m = tr(m, Vec3{x: 0.0, y: 0.0, z: -self.zoom});
        m = rot_x(m, -self.x_angle);
        m = rot_z(m, -self.z_angle);
        m = tr(m, self.pos);
        m
    }

    pub fn move(&mut self, angle: Float) {
        let speed_in_radians = deg_to_rad(self.z_angle - angle);
        let dx = sin(speed_in_radians);
        let dy = cos(speed_in_radians);
        self.pos.x -= dy;
        self.pos.y -= dx;
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
