#![allow(unused_imports)]
#![allow(dead_code)]

use macroquad::miniquad::native::linux_x11::libx11::KeyCode;

use {itertools::{iproduct, iterate},
     // bevy::{animation::prelude::Keyframes,
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
  // KeyCode::LeftShift
  // iterate(initial_value, f)
  let mut f = 5;
  swap!(f, inc);
  swap!(f, i32::add, 2)
}

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
// #[derive(Copy, Clone)]
struct Planet {
  pos: Vec3,
  vel: Vec3,
  color: Color,
  radius: f32,
}
// macro_rules! with {
//   ($($a:expr),*) => {
//     1
//   }; // ($a:expr) => {
//      //   $a + 1
//      // };
// }
// let u = with! {rng = |a, b| rand::gen_range::<f32>(a, b),
//                x = rng(0.1, 0.9),
//                y = rng(0.1, 0.9),
//                z = rng(0.1, 0.9),
//                Planet { color: Color { a: 1.0,
//                                        r: x,
//                                        g: y,
//                                        b: z },
//                         pos: vec3(x * 20.0, y * 20.0, z * 20.0),
//                         vel: vec3(rng(-0.01, 0.01), rng(-0.01, 0.01), rng(-0.01, 0.01)),
//                         radius: rng(0.1, 0.5) }
// };
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
    Self(iproduct!(0..NUM_PLANETS, 0..NUM_PLANETS).fold(self.0, |mut p, (i, j)| {
                                                    if j != i {
                                                      p[i].vel += 0.1
                                                                  * p[i].radius.powi(3)
                                                                  * p[j].radius.powi(3)
                                                                  * (p[j].pos - p[i].pos)
                                                                  / p[i].pos.distance(p[j].pos).powi(3);
                                                      p
                                                    } else {
                                                      p
                                                    }
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
  switch: bool,
  yaw: f32,
  pitch: f32,
  front: Vec3,
  right: Vec3,
  up: Vec3,
  position: Vec3,
  orientation: Quat,
  last_mouse_position: Vec2,
  grabbed: bool,
}
impl Default for State {
  fn default() -> Self {
    let yaw = 1.18;
    let pitch = 1.18;
    let front = vec3_from_spherical_coords(yaw, pitch);
    Self { planets: Planets([(); NUM_PLANETS].map(|_| Planet::random())),
           x: 0.0,
           switch: false,
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
impl state {
  fn update(self, change: Vec3, mouse_position: Vec2) -> Self {
    let State { front,
                last_mouse_position,
                pitch,
                planets,
                position,
                right,
                up,
                x,
                yaw,
                grabbed,
                .. } = self;
    let mouse_delta = mouse_position - last_mouse_position;
    State { planets: planets.gravity().movement(),
            x: x += if state.switch { 0.04 } else { -0.04 },
            position: position + change,
            last_mouse_position: mouse_position }
    // lost progress...
  }
}
// swap!(state.position, Add::add, state.front * MOVE_SPEED);
// const LOWER_BOUND: f32 = -8.0;
// const UPPER_BOUND: f32 = -LOWER_BOUND;
#[macroquad::main(conf)]
async fn main() {
  let mut state = State::default();
  let bounds = 8.0;
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
      // swap!(state.grabbed, not);
      state.grabbed = !state.grabbed;
      set_cursor_grab(state.grabbed);
      show_mouse(!state.grabbed);
    }
    let mut change = Vec3::ZERO;
    for (key, dir, sign) in [(KeyCode::W, state.front, 1.0),
                             (KeyCode::A, state.right, -1.0),
                             (KeyCode::S, state.front, -1.0),
                             (KeyCode::D, state.right, 1.0),
                             (KeyCode::LeftShift, state.up, 1.0),
                             (KeyCode::LeftControl, state.up, -1.0)]
    {
      if is_key_down(key) {
        change += dir * sign * MOVE_SPEED;
      }
    }

    let mouse_position: Vec2 = mouse_position().into();
    let mouse_delta = mouse_position - state.last_mouse_position;
    // state.last_mouse_position = mouse_position;
    state = state.update(change, mouse_position);

    // use quaternions instead of Vec3's?
    // Quat::clone_from(&mut self, source)
    state.yaw += mouse_delta.x * delta * LOOK_SPEED;
    state.pitch += mouse_delta.y * delta * -LOOK_SPEED;

    // swap!(state.pitch, clamp, -1.5, 1.5);
    state.pitch = state.pitch.clamp(-1.5, 1.5);

    state.front = vec3_from_spherical_coords(state.yaw, state.pitch);

    state.right = state.front.cross(WORLD_UP).normalize();
    state.up = state.right.cross(state.front).normalize();

    state.x += if state.switch { 0.04 } else { -0.04 };
    // match state.x {
    //   LOWER_BOUND..=UPPER_BOUND => (),
    //   _ => state.switch = !state.switch,
    // };
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

    // let State { planets: Planets(p), .. } = &state;
    for Planet { pos, color, radius, .. } in &state.planets.0 {
      draw_sphere(*pos, *radius, None, *color);
    }
    // state = State { planets: state.planets.gravity().movement(),
    //                 ..state };
    state.planets = state.planets.gravity().movement();

    // Back to screen space, render some text

    set_default_camera();
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
enum CurrentView {
  WorldView,
  InventoryView,
}
enum Key {
  Left,
  Right,
  Up,
  Down,
}
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
