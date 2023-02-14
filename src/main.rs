#![allow(unused_imports)]
#![allow(dead_code)]

use {// bevy::{animation::prelude::Keyframes,
     //         app::App,
     //         ecs::{event::{Event, Events},
     //               prelude::FromWorld,
     //               schedule::{IntoSystemDescriptor, Schedule, ShouldRun, Stage, StageLabel, State,
     //                          StateData, SystemSet, SystemStage},
     //               system::{Commands, In, Resource},
     //               world::World},
     //         prelude::*,
     //         window::Windows,
     //         DefaultPlugins}
     // ,
     macroquad::{color, miniquad::start, prelude::*},
  macroquad::material::gl_use_material
     std::{boxed::Box,
           collections::{BTreeMap, HashMap},
           convert::identity,
           fmt::{Debug, Display, Result},
           io::{self, stdin, stdout, BufRead, Error, Lines, Read, StdinLock, Write},
           iter::{IntoIterator, Map},
           num,
           ops::{Add, Div, MulAssign, Not, Rem, Sub},
           str::{FromStr, SplitAsciiWhitespace},
           string,
           vec::{IntoIter, Vec}}};

fn new<T: Default>() -> T { T::default() }
fn not(v: bool) -> bool { v.not() }

macro_rules! swap {
  ($x:expr,$f:expr, $($args:expr),*) => {{
    $x = $f($x, $($args),*);
    $x
  }};
  ($x:expr,$f:expr) => {{
    $x = $f($x);
    $x
  }};
}
// a().to_string().as_str()
/* -> &'static str */
// fn to_str<T: ToString>(v: T) -> &'static str { v.to_string().as_str() }
fn inc<T: Add<Output = T> + From<u8>>(a: T) -> T { a + T::from(1u8) }
fn a() -> i32 {
  let mut f = 5;
  swap!(f, inc);
  swap!(f, i32::add, 2)
}

// fn main() { Conf {} }

const MOVE_SPEED: f32 = 0.1;
const LOOK_SPEED: f32 = 0.1;
const WORLD_UP: Vec3 = vec3(0.0, 1.0, 0.0);

fn conf() -> Conf {
  Conf { window_title: String::from("Macroquad"),
         window_width: 1260,
         window_height: 768,
         fullscreen: false,
         ..Default::default() }
}

fn vec3_from_spherical_coords(yaw: f32, pitch: f32) -> Vec3 {
  vec3(yaw.cos() * pitch.cos(), pitch.sin(), yaw.sin() * pitch.cos()).normalize()
}
#[derive(Copy, Clone)]
struct Planet {
  pos: Vec3,
  vel: Vec3,
  color: Color,
  radius: f32,
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
const NUM_PLANETS: usize = 12;
#[derive(Copy, Clone)]
struct Planets([Planet; NUM_PLANETS]);
impl Planets {
  fn gravity(&mut self) {
    let p = &mut self.0;
    for i in 0..NUM_PLANETS {
      for j in 0..NUM_PLANETS {
        if j != i {
          p[i].vel +=
            0.004 * p[i].radius * p[j].radius * (p[j].pos - p[i].pos) / p[i].pos.distance(p[j].pos).powi(3);
        }
      }
    }
  }
  fn movement(&mut self) {
    for i in &mut self.0 {
      i.pos += i.vel;
      // i.pos += vec3(0.01, 0.01, 0.01);
    }
  }
}
struct State {
  planets: Planets,
  x: f32,
  switch: bool,
  yaw: f32,
  pitch: f32,
  front: Vec3,
  right: Vec3,
  up: Vec3,
  position: Vec3,
  last_mouse_position: Vec2,
  grabbed: bool,
}
impl Default for State {
  fn default() -> Self {
    let yaw_default: f32 = 1.18;
    let pitch_default: f32 = 1.18;
    let front_default: Vec3 = vec3_from_spherical_coords(yaw_default, pitch_default);
    // let zerovec = vec3(0.0, 0.0, 0.0);
    Self { planets: Planets([Planet::random(); NUM_PLANETS].map(|_| Planet::random())),
           x: 0.0,
           switch: false,
           yaw: yaw_default,
           pitch: pitch_default,
           front: front_default,
           right: front_default.cross(WORLD_UP).normalize(),
           up: vec3(0.0, 1.0, 0.0),
           position: vec3(0.0, 1.0, 0.0),
           last_mouse_position: mouse_position().into(),
           grabbed: true }
  }
}
enum A {
  B = 1,
  C = 2,
  D = 3,
}

#[macroquad::main(conf)]
async fn main() {
  let mut state = State::default();
  let bounds = 8.0;
  set_cursor_grab(state.grabbed);
  show_mouse(false);

  loop {
    // state.planets.0.sort_by(compare)
    let delta = get_frame_time();

    if is_key_pressed(KeyCode::Escape) {
      break;
    }
    if is_key_pressed(KeyCode::Tab) {
      swap!(state.grabbed, not);
      // state.grabbed = !state.grabbed;
      set_cursor_grab(state.grabbed);
      show_mouse(!state.grabbed);
    }

    if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
      // (swap! state.position + (* state.front MOVE_SPEED))
      swap!(state.position, Add::add, state.front * MOVE_SPEED);
      // state.position += state.front * MOVE_SPEED;
    }
    if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
      state.position -= state.front * MOVE_SPEED;
    }
    if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
      state.position -= state.right * MOVE_SPEED;
    }
    if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
      state.position += state.right * MOVE_SPEED;
    }

    let mouse_position: Vec2 = mouse_position().into();
    let mouse_delta = mouse_position - state.last_mouse_position;
    state.last_mouse_position = mouse_position;

    state.yaw += mouse_delta.x * delta * LOOK_SPEED;
    state.pitch += mouse_delta.y * delta * -LOOK_SPEED;

    swap!(state.pitch, clamp, -1.5, 1.5);
    // state.pitch = if state.pitch > 1.5 { 1.5 } else { state.pitch };
    // state.pitch = if state.pitch < -1.5 { -1.5 } else { state.pitch };

    state.front = vec3_from_spherical_coords(state.yaw, state.pitch);

    state.right = state.front.cross(WORLD_UP).normalize();
    state.up = state.right.cross(state.front).normalize();

    state.x += if state.switch { 0.04 } else { -0.04 };
    if state.x >= bounds || state.x <= -bounds {
      state.switch = !state.switch;
    }

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

    for Planet { pos, color, radius, .. } in state.planets.0 {
      draw_sphere(pos, radius, Option::<Texture2D>::None, color);
    }
    state.planets.gravity();
    state.planets.movement();
    // gravity(&mut state.planets);
    // movement(&mut state.planets);
    // swap!(state.planets, gravity);

    // Back to screen space, render some text

    set_default_camera();
    // draw_text("First Person Camera", 10.0, 20.0, 30.0, BLACK);
    draw_text(a().to_string().as_str(), 10.0, 20.0, 30.0, BLACK);
    draw_text(format!("Press <TAB> to toggle mouse grab: {}", state.grabbed).as_str(),
              10.0,
              48.0 + 42.0,
              30.0,
              WHITE);

    next_frame().await
  }
}

struct Coord(i32, i32);
const ORIGIN: Coord = Coord(0, 0);
struct Keyword(String);
enum EntityId {
  DynamicEntity(u32),
  ItemEntity(Keyword),
  Tile(Coord),
}
// enum CurrentView {
//     WorldView,
//     InventoryView,
// }
// enum Key {
//     Left,
//     Right,
//     Up,
//     Down,
// }
// struct Cev {} // all components in a stuct...
//               // struct Db{
//               //    mouse_over_relative_coord:Option<Coord>,
//               //    scroll_pos:u8,
//               //    entity_count:Int,
//               //    cev:Cev,
//               //    selected_entity:Option<EntityId>,
//               //    // history                   '()
//               //    reverse_time:bool,
//               //    time:u32,
//               //    message_log:Vec<String>,
//               //    pressed_keys              #{}
//               //    new_pressed_keys          #{}
//               //    newest_pressed_y:Key,
//               //    newest_pressed_x:Key,
//               //    current_view:CurrentView
//               // }
// impl Default for Db {}
