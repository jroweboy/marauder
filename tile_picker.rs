// See LICENSE file for copyright and license details.

use std;
use cgmath::vector::{
    Vec3,
    Vec2,
};
use gl_helpers::{
    Shader,
    get_attr,
    get_uniform,
    uniform_mat4f,
    set_clear_color,
    clear,
};
use map::MapPosIter;
use camera::Camera;
use geom::Geom;
use mesh::Mesh;
use core_types::{
    Int,
    Size2,
    MapPos,
};
use gl_types::{
    VertexCoord,
    Color3,
    Float,
    MatId,
};

fn build_hex_map_mesh(
    geom: &Geom,
    map_size: Size2<Int>
) -> (~[VertexCoord], ~[Color3]) {
    let mut c_data = ~[];
    let mut v_data = ~[];
    for tile_pos in MapPosIter::new(map_size) {
        let pos3d = geom.map_pos_to_world_pos(tile_pos);
        for num in range(0 as Int, 6) {
            let vertex = geom.index_to_hex_vertex(num);
            let next_vertex = geom.index_to_hex_vertex(num + 1);
            let col_x = tile_pos.x as Float / 255.0;
            let col_y = tile_pos.y as Float / 255.0;
            let color = Color3{r: col_x, g: col_y, b: 1.0};
            v_data.push(pos3d + vertex);
            c_data.push(color);
            v_data.push(pos3d + next_vertex);
            c_data.push(color);
            v_data.push(pos3d + Vec3::zero());
            c_data.push(color);
        }
    }
    (v_data, c_data)
}

pub struct TilePicker {
    shader: Shader,
    map_mesh: Mesh,
    mat_id: MatId,
    win_size: Size2<Int>,
}

impl TilePicker {
    pub fn new(
        win_size: Size2<Int>,
        geom: &Geom,
        map_size: Size2<Int>
    ) -> ~TilePicker {
        let mut picker = ~TilePicker {
            shader: Shader(0),
            map_mesh: Mesh::new(),
            mat_id: MatId(0),
            win_size: win_size,
        };
        picker.init(geom, map_size);
        picker
    }

    pub fn set_win_size(&mut self, win_size: Size2<Int>) {
        self.win_size = win_size;
    }

    fn init(&mut self, geom: &Geom, map_size: Size2<Int>) {
        self.shader = Shader::new("pick.vs.glsl", "pick.fs.glsl");
        self.shader.activate();
        let position_attr = get_attr(
            &self.shader, "in_vertex_coordinates");
        let color_attr = get_attr(&self.shader, "color");
        position_attr.enable();
        color_attr.enable();
        position_attr.vertex_pointer(3);
        color_attr.vertex_pointer(3);
        let (vertex_data, color_data) = build_hex_map_mesh(geom, map_size);
        self.map_mesh.set_vertex_coords(vertex_data);
        self.map_mesh.set_color(color_data);
        self.mat_id = MatId(get_uniform(&self.shader, "mvp_mat"));
    }

    fn read_coords_from_image_buffer(
        &self,
        mouse_pos: Vec2<Int>
    ) -> Option<MapPos> {
        use gl; // TODO: remove
        let height = self.win_size.h;
        let reverted_y = height - mouse_pos.y;
        let data: [u8, ..4] = [0, 0, 0, 0]; // mut
        unsafe {
            let data_ptr = std::cast::transmute(&data[0]);
            gl::ReadPixels(
                mouse_pos.x, reverted_y, 1, 1,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data_ptr
            );
        }
        if data[2] != 0 {
            Some(Vec2{x: data[0] as Int, y: data[1] as Int})
        } else {
            None
        }
    }

    pub fn pick_tile(
        &mut self,
        camera: &Camera,
        mouse_pos: Vec2<Int>
    ) -> Option<MapPos> {
        self.shader.activate();
        uniform_mat4f(self.mat_id, &camera.mat());
        set_clear_color(0.0, 0.0, 0.0);
        clear();
        self.map_mesh.draw(&self.shader);
        self.read_coords_from_image_buffer(mouse_pos)
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
