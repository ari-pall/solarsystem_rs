#![allow(unused_imports)]
#![allow(dead_code)]
#![feature(exclusive_range_pattern)]
// use macroquad::miniquad::native::linux_x11::libx11::KeyCode;

use {itertools::{iproduct, iterate},
     macroquad::{color, miniquad::KeyCode, prelude::*},
     std::{boxed::Box,
           collections::{BTreeMap, HashMap},
           convert::identity,
           fmt::{Debug, Display, Result},
           io::{self, stdin, stdout, BufRead, Error, Lines, Read, StdinLock, Write},
           iter::{IntoIterator, Map},
           num,
           ops::{Add, Deref, DerefMut, Div, MulAssign, Not, Rem, Sub},
           str::{FromStr, SplitAsciiWhitespace},
           string,
           vec::{IntoIter, Vec}}};

fn new<T: Default>() -> T { T::default() }
fn not(v: bool) -> bool { v.not() }

const MOVE_SPEED: f32 = 0.1;
const LOOK_SPEED: f32 = 0.1;
const WORLD_UP: Vec3 = vec3(0.0, 1.0, 0.0);

fn conf() -> Conf {
  Conf { window_title: String::from("solar system"),
         // window_width: 1260,
         // window_height: 768,
         fullscreen: true,
         ..Default::default() }
}

fn vec3_from_spherical_coords(yaw: f32, pitch: f32) -> Vec3 {
  vec3(yaw.cos() * pitch.cos(), pitch.sin(), yaw.sin() * pitch.cos()).normalize()
}
// #[derive(Copy, Clone)]
struct Planet {
  pos: Vec3,
  vel: Vec3,
  color: Color,
  radius: f32
}
impl Planet {
  fn random() -> Planet {
    let rng = |a, b| rand::gen_range::<f32>(a, b);
    let x = rng(0.1, 0.9);
    let y = rng(0.1, 0.9);
    let z = rng(0.1, 0.9);
    Planet { color: Color { a: 1.0,
                            r: x,
                            g: y,
                            b: z },
             pos: vec3(x * 20.0, y * 20.0, z * 20.0),
             vel: vec3(rng(-0.01, 0.01), rng(-0.01, 0.01), rng(-0.01, 0.01)),
             radius: rng(0.1, 0.5) }
  }
}

const NUM_PLANETS: usize = 30;
// #[derive(Clone)]
struct Planets([Planet; NUM_PLANETS]);
impl Planets {
  fn gravity(self) -> Self {
    Self(iproduct!(0..NUM_PLANETS, 0..NUM_PLANETS).filter(|(i, j)| i != j)
                                                  .fold(self.0, |mut p, (i, j)| {
                                                    p[i].vel += 0.1
                                                                * p[i].radius.powi(3)
                                                                * p[j].radius.powi(3)
                                                                * (p[j].pos - p[i].pos)
                                                                / p[i].pos.distance(p[j].pos).powi(3);
                                                    p
                                                  }))
  }
  fn movement(self) -> Self {
    Self(self.0.map(|mut planet| {
                 planet.pos += planet.vel;
                 planet
               }))
  }
}
struct State {
  planets: Planets,
  x: f32,
  yaw: f32,
  pitch: f32,
  front: Vec3,
  right: Vec3,
  up: Vec3,
  position: Vec3,
  orientation: Quat,
  last_mouse_position: Vec2,
  grabbed: bool,
  xdiff: f32
}
impl Default for State {
  fn default() -> Self {
    let yaw = 1.18;
    let pitch = 1.18;
    let front = vec3_from_spherical_coords(yaw, pitch);
    Self { planets: Planets([(); NUM_PLANETS].map(|_| Planet::random())),
           x: 0.0,
           yaw,
           pitch,
           front,
           right: front.cross(WORLD_UP).normalize(),
           up: vec3(0.0, 1.0, 0.0),
           position: vec3(0.0, 1.0, 0.0),
           orientation: Quat::default(),
           last_mouse_position: mouse_position().into(),
           xdiff: 0.04,
           grabbed: true }
  }
}
impl State {
  const HI: f32 = 8.0;
  const LO: f32 = -Self::HI;
  fn update(self, change: Vec3, mouse_position: Vec2, delta: f32) -> Self {
    let State { last_mouse_position,
                pitch,
                yaw,
                front,
                right,
                planets,
                position,
                x,
                xdiff,
                .. } = self;
    let mouse_delta = mouse_position - last_mouse_position;

    State { last_mouse_position: mouse_position,
            pitch: (pitch + mouse_delta.y * delta * -LOOK_SPEED).clamp(-1.5, 1.5),
            yaw: yaw + mouse_delta.x * delta * LOOK_SPEED,
            front: vec3_from_spherical_coords(yaw, pitch),
            right: front.cross(WORLD_UP).normalize(),
            up: right.cross(front).normalize(),
            planets: planets.gravity().movement(),
            position: position + change,
            x: x + xdiff,
            xdiff: if x < Self::LO {
              0.04
            } else if x > Self::HI {
              -0.04
            } else {
              xdiff
            },
            ..self }
  }
}
#[macroquad::main(conf)]
async fn main() {
  let mut state = State::default();
  set_cursor_grab(state.grabbed);
  show_mouse(false);
  loop {
    // Quat::from_rotation_y(angle)
    //   Quat::to_axis_angle(self)
    let delta = get_frame_time();

    if is_key_pressed(KeyCode::Escape) {
      break;
    }
    if is_key_pressed(KeyCode::Tab) {
      state.grabbed = !state.grabbed;
      set_cursor_grab(state.grabbed);
      show_mouse(!state.grabbed);
    }
    let change = [(KeyCode::W, state.front, 1.0),
                  (KeyCode::A, state.right, -1.0),
                  (KeyCode::S, state.front, -1.0),
                  (KeyCode::D, state.right, 1.0),
                  (KeyCode::LeftShift, state.up, 1.0),
                  (KeyCode::LeftControl, state.up, -1.0)].into_iter()
                                                         .filter(|(key, ..)| is_key_down(*key))
                                                         .map(|(.., dir, sign)| dir * sign * MOVE_SPEED)
                                                         .fold(Vec3::ZERO, Add::add);
    let mouse_position = Vec2::from(mouse_position());
    state = state.update(change, mouse_position, delta);

    clear_background(BLACK);

    // Going 3d!

    set_camera(&Camera3D { position: state.position,
                           up: state.up,
                           target: state.position + state.front,
                           ..Default::default() });

    draw_grid(20, 1., DARKGRAY, GRAY);

    draw_line_3d(vec3(state.x, 0.0, state.x),
                 vec3(5.0, 5.0, 5.0),
                 Color::new(1.0, 1.0, 0.0, 1.0));

    draw_cube_wires(vec3(0., 1., -6.), vec3(2., 2., 2.), GREEN);
    draw_cube_wires(vec3(0., 1., 6.), vec3(2., 2., 2.), BLUE);
    draw_cube_wires(vec3(2., 1., 2.), vec3(2., 2., 2.), RED);

    for Planet { pos, color, radius, .. } in &state.planets.0 {
      draw_sphere(*pos, *radius, None, *color);
    }

    // Back to screen space, render some text

    set_default_camera();
    // draw_text(a().to_string().as_str(), 10.0, 20.0, 30.0, BLACK);
    draw_text(format!("Press <TAB> to toggle mouse grab: {}", state.grabbed).as_str(),
              10.0,
              48.0 + 42.0,
              30.0,
              WHITE);

    next_frame().await
  }
}
