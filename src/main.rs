#![allow(unused_imports)]
#![allow(dead_code)]

use {derive_more::From,
     // iter_comprehensions::sum,
     list_comprehension::comp,
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
             pos: vec3(x * 20.0 - 4.0, y * 20.0 - 4.0, z * 20.0 - 4.0),
             vel: vec3(rng(-0.02, 0.02), rng(-0.02, 0.02), rng(-0.02, 0.02)),
             radius: rng(0.1, 0.3) }
  }
}

const NUM_PLANETS: usize = 30;
// #[derive(From)]
struct Planets(Vec<Planet>);
impl Default for Planets {
  fn default() -> Self {
    Self(comp!(
      if i == 0 { Planet { pos: Vec3 { x: 3.1,
                                       y: 1.3,
                                       z: 2.3 },
                           vel: Vec3::default(),
                           color: Color::from_rgba(255, 255, 0, 255),
                           radius: 1.3 }
      } else {
        Planet::random()
      }, i in 0..NUM_PLANETS))
  }
}
pub fn fold<T, G>(init: T, f: impl Fn(T, G) -> T, coll: impl IntoIterator<Item = G>) -> T {
  coll.into_iter().fold(init, f)
}
impl Planets {
  fn movement(self) -> Self { Self(comp!(Planet{pos: p.pos+p.vel,..p}, p in self.0)) }
  fn gravity(self) -> Self {
    Self(fold(self.0,
              |mut p, (i, j)| {
                let dist = p[i].pos.distance(p[j].pos);
                let posdiff = p[j].pos - p[i].pos;
                let pimass = p[i].radius.powi(3);
                let pjmass = p[j].radius.powi(3);
                let g = 0.003;
                p[i].vel += g * pjmass * posdiff / dist.powi(3);
                p[j].vel -= g * pimass * posdiff / dist.powi(3);
                p
              },
              comp!((i,j), i in 0..NUM_PLANETS, j in 0..i)))
  }
}
struct State {
  planets: Planets,
  yaw: f32,
  pitch: f32,
  front: Vec3,
  right: Vec3,
  up: Vec3,
  position: Vec3,
  orientation: Quat,
  last_mouse_position: Vec2,
  grabbed: bool
}
impl Default for State {
  fn default() -> Self {
    let yaw = 1.18;
    let pitch = 1.18;
    let front = vec3_from_spherical_coords(yaw, pitch);
    Self { planets: Planets::default(),
           yaw,
           pitch,
           front,
           right: front.cross(WORLD_UP).normalize(),
           up: vec3(0.0, 1.0, 0.0),
           position: vec3(0.0, 1.0, 0.0),
           orientation: Quat::default(),
           last_mouse_position: mouse_position().into(),
           grabbed: true }
  }
}
impl State {
  // const HI: f32 = 8.0;
  // const LO: f32 = -Self::HI;
  fn update(self, change: Vec3, mouse_position: Vec2, delta: f32) -> Self {
    let Self { last_mouse_position,
               pitch,
               yaw,
               front,
               right,
               planets,
               position,
               .. } = self;
    let mouse_delta = mouse_position - last_mouse_position;

    Self { last_mouse_position: mouse_position,
           pitch: (pitch + mouse_delta.y * delta * -LOOK_SPEED).clamp(-1.5, 1.5),
           yaw: yaw + mouse_delta.x * delta * LOOK_SPEED,
           front: vec3_from_spherical_coords(yaw, pitch),
           right: front.cross(WORLD_UP).normalize(),
           up: right.cross(front).normalize(),
           planets: planets.gravity().movement(),
           position: position + change,
           ..self }
  }
}
pub fn sum<T: Default + std::ops::Add<Output = T>>(coll: impl IntoIterator<Item = T>) -> T {
  coll.into_iter().fold(T::default(), T::add)
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
    // let change = MOVE_SPEED
    //              * sum!((key,dir) in [(KeyCode::W, state.front),
    //                              (KeyCode::A, -state.right),
    //                              (KeyCode::S, -state.front),
    //                              (KeyCode::D, state.right),
    //                              (KeyCode::LeftShift, state.up),
    //                              (KeyCode::LeftControl, -state.up)]
    //               , is_key_down(key) ,dir);
    let change = MOVE_SPEED
                 * sum(comp!(dir, (key, dir) in [(KeyCode::W, state.front),
                                                 (KeyCode::A, -state.right),
                                                 (KeyCode::S, -state.front),
                                                 (KeyCode::D, state.right),
                                                 (KeyCode::LeftShift, state.up),
                                                 (KeyCode::LeftControl, -state.up)]
                             , is_key_down(key)));
    let mouse_position = Vec2::from(mouse_position());
    state = state.update(change, mouse_position, delta);

    clear_background(BLACK);

    // Going 3d!

    set_camera(&Camera3D { position: state.position,
                           up: state.up,
                           target: state.position + state.front,
                           ..Default::default() });

    draw_grid(20, 1., DARKGRAY, GRAY);

    // draw_line_3d(vec3(state.x, 0.0, state.x),
    //              vec3(5.0, 5.0, 5.0),
    //              Color::new(1.0, 1.0, 0.0, 1.0));

    // draw_cube_wires(vec3(0., 1., -6.), vec3(2., 2., 2.), GREEN);
    // draw_cube_wires(vec3(0., 1., 6.), vec3(2., 2., 2.), BLUE);
    // draw_cube_wires(vec3(2., 1., 2.), vec3(2., 2., 2.), RED);

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
