// See LICENSE file for copyright and license details.

// Marauder is turn-based strategy game with hex grid.

extern mod glfw;
extern mod gl;
extern mod cgmath;

use std::f32::consts::{
  PI,
  FRAC_PI_2
};
use std::num::{
  sqrt,
  pow,
  abs,
  sin,
  cos
};
use std::option;
use gltypes = gl::types;
use cgmath::matrix::{
  Matrix,
  Mat4,
  Mat3,
  ToMat4
};
use cgmath::vector::{
  Vec3,
  Vec2,
  Vector
};
use cgmath::projection;
use cgmath::angle;

static WIN_SIZE: Vec2<u32> = Vec2{x: 640, y: 480};

// hack to pass mutable Win to glfs-rs callbacks
static mut WIN: Option<*mut Win> = None;

fn get_win() -> &mut Win {
  unsafe {
    match WIN {
      Some(win) => &mut *win,
      None => fail!("Bad Win pointer")
    }
  }
}

fn set_win(win: &mut Win) {
  unsafe { WIN = Some(win as (*mut Win)); }
}

static VERTEX_SHADER_SRC: &'static str = "
  #version 130
  in vec3 position;
  uniform mat4 model_view_proj_matrix;
  void main() {
    vec4 v = vec4(position, 1);
    gl_Position = model_view_proj_matrix * v;
  }
";
 
static FRAGMENT_SHADER_SRC: &'static str = "
  #version 130
  out vec4 out_color;
  void main() {
    out_color = vec4(1.0, 1.0, 1.0, 1.0);
  }
";

struct Camera {
  x_angle: f32,
  z_angle: f32,
  pos: Vec3<f32>,
  zoom: f32,
  projection_matrix: Mat4<f32>,
}

impl Camera {
  pub fn new() -> Camera {
    Camera {
      x_angle: 0.0,
      z_angle: 0.0,
      pos: Vec3{x: 0.0, y: 0.0, z: 0.0},
      zoom: 10.0,
      projection_matrix: get_projection_matrix(),
    }
  }

  pub fn matrix(&self) -> Mat4<f32> {
    let mut mvp_matrix = self.projection_matrix;
    mvp_matrix = tr(mvp_matrix, Vec3{x: 0.0f32, y: 0.0, z: -10.0f32});
    mvp_matrix = rot_x(mvp_matrix, self.z_angle);
    mvp_matrix = rot_y(mvp_matrix, self.x_angle);
    mvp_matrix = tr(mvp_matrix, self.pos);
    mvp_matrix
  }
}

pub struct Visualizer {
  hex_ex_radius: gltypes::GLfloat,
  hex_in_radius: gltypes::GLfloat
}

impl Visualizer {
  pub fn new() -> Visualizer {
    let hex_ex_radius: gltypes::GLfloat = 1.0 / 2.0;
    let hex_in_radius = sqrt(
        pow(hex_ex_radius, 2.0) - pow(hex_ex_radius / 2.0, 2.0));
    let visualizer = Visualizer {
      hex_ex_radius: hex_ex_radius,
      hex_in_radius: hex_in_radius
    };
    visualizer
  }

  pub fn dist(a: Vec2<f32>, b: Vec2<f32>) -> f32 {
    let dx = abs(b.x - a.x);
    let dy = abs(b.y - a.y);
    sqrt(pow(dx, 2.0) + pow(dy, 2.0))
  }

  pub fn v2i_to_v2f(&self, i: Vec2<i32>) -> Vec2<f32> {
    let v = Vec2 {
      x: (i.x as f32) * self.hex_in_radius * 2.0,
      y: (i.y as f32) * self.hex_ex_radius * 1.5
    };
    if i.y % 2 == 0 {
      Vec2{x: v.x + self.hex_in_radius, y: v.y}
    } else {
      v
    }
  }

  pub fn index_to_circle_vertex(&self, count: int, i: int) -> Vec2<f32> {
    let n = FRAC_PI_2 + 2.0 * PI * (i as f32) / (count as f32);
    Vec2{x: cos(n), y: sin(n)}.mul_s(self.hex_ex_radius)
  }

  pub fn index_to_hex_vertex(&self, i: int) -> Vec2<f32> {
    self.index_to_circle_vertex(6, i)
  }
}

fn compile_shader(src: &str, shader_type: gltypes::GLenum) -> gltypes::GLuint {
  let shader = gl::CreateShader(shader_type);
  unsafe {
    gl::ShaderSource(shader, 1, &src.to_c_str().unwrap(), std::ptr::null());
    gl::CompileShader(shader);

    let mut status = gl::FALSE as gltypes::GLint;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

    if status != (gl::TRUE as gltypes::GLint) {
      let mut len = 0;
      gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
      // subtract 1 to skip the trailing null character
      let mut buf = std::vec::from_elem(len as uint - 1, 0u8);
      gl::GetShaderInfoLog(shader, len, std::ptr::mut_null(),
        buf.as_mut_ptr() as *mut gltypes::GLchar
      );
      fail!("compile_shader(): " + std::str::raw::from_utf8(buf));
    }
  }
  shader
}

fn link_program(
    vertex_shader: gltypes::GLuint,
    fragment_shader: gltypes::GLuint
) -> gltypes::GLuint {
  let program = gl::CreateProgram();
  gl::AttachShader(program, vertex_shader);
  gl::AttachShader(program, fragment_shader);
  gl::LinkProgram(program);
  unsafe {
    let mut status = gl::FALSE as gltypes::GLint;
    gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

    if status != (gl::TRUE as gltypes::GLint) {
      let mut len: gltypes::GLint = 0;
      gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
      // subtract 1 to skip the trailing null character
      let mut buf = std::vec::from_elem(len as uint - 1, 0u8);
      gl::GetProgramInfoLog(program, len, std::ptr::mut_null(),
        buf.as_mut_ptr() as *mut gltypes::GLchar
      );
      fail!("link_program(): " + std::str::raw::from_utf8(buf));
    }
  }
  program
}

fn tr(m: Mat4<f32>, v: Vec3<f32>) -> Mat4<f32> {
  let mut t = Mat4::<f32>::identity();
  *t.mut_cr(3, 0) = v.x;
  *t.mut_cr(3, 1) = v.y;
  *t.mut_cr(3, 2) = v.z;
  m.mul_m(&t)
}

fn rot_x(m: Mat4<f32>, angle: f32) -> Mat4<f32> {
  let r = Mat3::from_angle_x(angle::rad(angle)).to_mat4();
  m.mul_m(&r)
}

fn rot_y(m: Mat4<f32>, angle: f32) -> Mat4<f32> {
  let r = Mat3::from_angle_y(angle::rad(angle)).to_mat4();
  m.mul_m(&r)
}

pub struct Win {
  vertex_shader: gltypes::GLuint,
  fragment_shader: gltypes::GLuint,
  program: gltypes::GLuint,
  vertex_buffer_obj: gltypes::GLuint,
  matrix_id: gltypes::GLint,
  window: Option<glfw::Window>,
  vertex_data: ~[gltypes::GLfloat],
  mouse_pos: Vec2<f32>,
  camera: Camera
}

fn get_projection_matrix() -> Mat4<f32> {
  let fov = angle::deg(45.0f32);
  let ratio = 4.0 / 3.0;
  let display_range_min = 0.1;
  let display_range_max = 100.0;
  projection::perspective(
    fov, ratio, display_range_min, display_range_max
  )
}

// TODO: use iterator?
fn for_each_tile(f: |Vec2<i32>|) {
  let map_size = Vec2{x: 3, y: 4};
  for y in range(0i32, map_size.y) {
    for x in range(0i32, map_size.x) {
      f(Vec2{x: x, y: y});
    }
  }
}

impl Win {
  pub fn new() -> ~Win {
    let mut win = ~Win {
      vertex_shader: 0,
      fragment_shader: 0,
      program: 0,
      vertex_buffer_obj: 0,
      matrix_id: 0,
      window: option::None,
      vertex_data: ~[],
      mouse_pos: Vec2{x: 0.0f32, y: 0.0},
      camera: Camera::new()
    };
    set_win(&mut *win);
    win.init_glfw();
    win.init_opengl();
    win.init_model();
    win
  }

  fn add_point(&mut self, pos: &Vec3<f32>, x: f32, y: f32, z: f32) {
    self.vertex_data.push(x + pos.x);
    self.vertex_data.push(y + pos.y);
    self.vertex_data.push(z + pos.z);
  }

  fn build_hex_mesh(&mut self) {
    let v = Visualizer::new(); // TODO: move out here
    for_each_tile(|tile_pos| {
      let pos3d = v.v2i_to_v2f(tile_pos).extend(0.0);
      for num in range(0, 6) {
        let vertex = v.index_to_hex_vertex(num);
        let next_vertex = v.index_to_hex_vertex(num + 1);
        self.add_point(&pos3d, vertex.x, vertex.y, 0.0);
        self.add_point(&pos3d, next_vertex.x, next_vertex.y, 0.0);
        self.add_point(&pos3d, 0.0, 0.0, 0.0);
      }
    });
  }

  fn init_model(&mut self) {
    self.build_hex_mesh();
    unsafe {
      // Create a Vertex Buffer Object
      gl::GenBuffers(1, &mut self.vertex_buffer_obj);
      gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer_obj);

      // Copy vertex data to VBO
      let float_size = std::mem::size_of::<gltypes::GLfloat>();
      let vertices_ptr = (self.vertex_data.len() * float_size)
        as gltypes::GLsizeiptr;
      gl::BufferData(
        gl::ARRAY_BUFFER,
        vertices_ptr,
        std::cast::transmute(&self.vertex_data[0]),
        gl::STATIC_DRAW
      );

      gl::UseProgram(self.program);
      gl::BindFragDataLocation(
        self.program, 0, "out_color".to_c_str().unwrap());

      // Specify the layout of the vertex data
      let pos_attr = gl::GetAttribLocation(
        self.program, "position".to_c_str().unwrap()) as gltypes::GLuint;
      gl::EnableVertexAttribArray(pos_attr);

      let size = 3;
      let normalized = gl::FALSE;
      let stride = 0;
      gl::VertexAttribPointer(
        pos_attr,
        size,
        gl::FLOAT,
        normalized,
        stride,
        std::ptr::null()
      );

      self.matrix_id = gl::GetUniformLocation(
        self.program, "model_view_proj_matrix".to_c_str().unwrap()
      );
    }
  }

  fn init_glfw(&mut self) {
    // glfw::window_hint::context_version(3, 2);
    glfw::set_error_callback(~glfw::LogErrorHandler);
    glfw::init();
    self.window = option::Some(
      glfw::Window::create(
        WIN_SIZE.x,
        WIN_SIZE.y,
        "OpenGL",
        glfw::Windowed
      ).unwrap()
    );
  }

  fn init_opengl(&mut self) {
    let window = self.window.get_ref();
    window.make_context_current();
    window.set_cursor_pos_callback(~CursorPosContext);
    window.set_key_callback(~KeyContext);

    // Load the OpenGL function pointers
    gl::load_with(glfw::get_proc_address);

    self.vertex_shader = compile_shader(
      VERTEX_SHADER_SRC, gl::VERTEX_SHADER);
    self.fragment_shader = compile_shader(
      FRAGMENT_SHADER_SRC, gl::FRAGMENT_SHADER);
    self.program = link_program(self.vertex_shader, self.fragment_shader);
  }

  pub fn cleanup_opengl(&self) {
    gl::DeleteProgram(self.program);
    gl::DeleteShader(self.fragment_shader);
    gl::DeleteShader(self.vertex_shader);
    unsafe {
      gl::DeleteBuffers(1, &self.vertex_buffer_obj);
    }
  }

  fn update_matrices(&self) {
    let mvp_matrix = self.camera.matrix();
    unsafe {
      // Send our transformation to the currently bound shader,
      // in the "model_view_proj_matrix" uniform for each model
      // you render, since the model_view_proj_matrix will be
      // different (at least the M part).
      gl::UniformMatrix4fv(self.matrix_id, 1, gl::FALSE, mvp_matrix.cr(0, 0));
    }
  }

  pub fn draw(&self) {
    self.update_matrices();
    gl::ClearColor(0.3, 0.3, 0.3, 1.0);
    gl::Clear(gl::COLOR_BUFFER_BIT);
    gl::DrawArrays(gl::TRIANGLES, 0, self.vertex_data.len() as i32);
    self.window.get_ref().swap_buffers();
  }

  pub fn is_running(&self) -> bool {
    return !self.window.get_ref().should_close()
  }

  pub fn process_events(&self) {
    glfw::poll_events();
  }
}

impl Drop for Win {
  fn drop(&mut self) {
    self.cleanup_opengl();

    // destroy glfw::Window before terminating glfw
    self.window = option::None;

    glfw::terminate();
  }
}

struct CursorPosContext;
impl glfw::CursorPosCallback for CursorPosContext {
  fn call(&self, w: &glfw::Window, xpos: f64, ypos: f64) {
    if w.get_mouse_button(glfw::MouseButtonRight) == glfw::Press {
      let dx = get_win().mouse_pos.x - xpos as f32;
      let dy = get_win().mouse_pos.y - ypos as f32;
      get_win().camera.z_angle += dx / 10.0;
      get_win().camera.x_angle += dy / 10.0;
      get_win().mouse_pos.x = xpos as f32;
      get_win().mouse_pos.y = ypos as f32;
    }
  }
}

struct KeyContext;
impl glfw::KeyCallback for KeyContext {
  fn call(
    &self,
    window: &glfw::Window,
    key:    glfw::Key,
    _:      std::libc::c_int,
    action: glfw::Action,
    _:      glfw::Modifiers
  ) {
    let distance = 1.0;
    if action != glfw::Press {
      return;
    }
    match key {
      glfw::KeyEscape | glfw::KeyQ
                     => window.set_should_close(true),
      glfw::KeySpace => println!("space"),
      glfw::KeyUp    => get_win().camera.pos.y -= distance,
      glfw::KeyDown  => get_win().camera.pos.y += distance,
      glfw::KeyRight => get_win().camera.pos.x -= distance,
      glfw::KeyLeft  => get_win().camera.pos.x += distance,
      _ => {}
    }
  }
}

// vim: set tabstop=2 shiftwidth=2 softtabstop=2 expandtab: