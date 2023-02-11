#![allow(unused_imports)]
#![allow(dead_code)]

use macroquad::color;
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
     macroquad::prelude::*,
     std::{// arch::x86_64::_mm_mask_fnmadd_pd,
           boxed::Box,
           collections::{BTreeMap, HashMap},
           convert::identity,
           fmt::{Debug, Display, Result},
           io::{self, stdin, stdout, BufRead, Error, Lines, Read, StdinLock, Write},
           iter::Map,
           num,
           ops::{Add, Div, Not, Rem, Sub},
           str::{FromStr, SplitAsciiWhitespace},
           string,
           vec::{IntoIter, Vec}}};

fn new<T: Default>() -> T { T::default() }
fn not(v: bool) -> bool { v.not() }
fn swap<R, F: Fn(&R) -> R>(r: &mut R, f: F) { *r = f(&r); }

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
struct Planet(Vec3, Vec3, color::Color, f32);
struct State {
  pub planets: Vec<Planet>,
  pub x: f32,
  pub switch: bool,
  pub yaw: f32,
  pub pitch: f32,
  pub front: Vec3,
  pub right: Vec3,
  pub up: Vec3,
  pub position: Vec3,
  pub last_mouse_position: Vec2,
  pub grabbed: bool,
}
impl Default for State {
  fn default() -> Self {
    let yaw_default: f32 = 1.18;
    let pitch_default: f32 = 1.18;
    let front_default: Vec3 = vec3_from_spherical_coords(yaw_default, pitch_default);
    let zerovec = vec3(0.0, 0.0, 0.0);
    Self { planets: vec![Planet(vec3(1.5, 1.5, 1.5), zerovec, GREEN, 1.0),
                         Planet(vec3(1.1, -2.2, 1.5), zerovec, BLUE, 1.0),
                         Planet(vec3(-1.1, -2.2, -1.5), zerovec, GREEN, 1.0),
                         Planet(vec3(3.9, -2.1, -1.5), zerovec, RED, 1.0)],
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

#[macroquad::main(conf)]
async fn main() {
  let mut state = State::default();
  let bounds = 8.0;
  set_cursor_grab(state.grabbed);
  show_mouse(false);

  loop {
    let delta = get_frame_time();

    if is_key_pressed(KeyCode::Escape) {
      break;
    }
    if is_key_pressed(KeyCode::Tab) {
      // swap(&mut state.grabbed, not);
      state.grabbed = !state.grabbed;
      set_cursor_grab(state.grabbed);
      show_mouse(!state.grabbed);
    }

    if is_key_down(KeyCode::Up) {
      state.position += state.front * MOVE_SPEED;
    }
    if is_key_down(KeyCode::Down) {
      state.position -= state.front * MOVE_SPEED;
    }
    if is_key_down(KeyCode::Left) {
      state.position -= state.right * MOVE_SPEED;
    }
    if is_key_down(KeyCode::Right) {
      state.position += state.right * MOVE_SPEED;
    }

    let mouse_position: Vec2 = mouse_position().into();
    let mouse_delta = mouse_position - state.last_mouse_position;
    state.last_mouse_position = mouse_position;

    state.yaw += mouse_delta.x * delta * LOOK_SPEED;
    state.pitch += mouse_delta.y * delta * -LOOK_SPEED;

    // swap(state.pitch, |p| p.clamp(-1.5, 1.5));
    state.pitch = if state.pitch > 1.5 { 1.5 } else { state.pitch };
    state.pitch = if state.pitch < -1.5 { -1.5 } else { state.pitch };

    state.front = vec3_from_spherical_coords(state.yaw, state.pitch);

    state.right = state.front.cross(WORLD_UP).normalize();
    state.up = state.right.cross(state.front).normalize();

    state.x += if state.switch { 0.04 } else { -0.04 };
    if state.x >= bounds || state.x <= -bounds {
      state.switch = !state.switch;
    }

    clear_background(DARKGRAY);

    // Going 3d!

    set_camera(&Camera3D { position: state.position,
                           up: state.up,
                           target: state.position + state.front,
                           ..Default::default() });

    draw_grid(20, 1., BLACK, GRAY);

    draw_line_3d(vec3(state.x, 0.0, state.x),
                 vec3(5.0, 5.0, 5.0),
                 Color::new(1.0, 1.0, 0.0, 1.0));

    draw_cube_wires(vec3(0., 1., -6.), vec3(2., 2., 2.), GREEN);
    draw_cube_wires(vec3(0., 1., 6.), vec3(2., 2., 2.), BLUE);
    draw_cube_wires(vec3(2., 1., 2.), vec3(2., 2., 2.), RED);

    for Planet(coord, vel, color, radius) in &state.planets {
      draw_sphere(coord.clone(),
                  radius.clone(),
                  Option::<Texture2D>::None,
                  color.clone());
    }
    // for p1 in &state.planets {}

    // Back to screen space, render some text

    set_default_camera();
    draw_text("First Person Camera", 10.0, 20.0, 30.0, BLACK);

    draw_text(format!("X: {} Y: {}", mouse_position.x, mouse_position.y).as_str(),
              10.0,
              48.0 + 18.0,
              30.0,
              BLACK);
    draw_text(format!("Press <TAB> to toggle mouse grab: {}", state.grabbed).as_str(),
              10.0,
              48.0 + 42.0,
              30.0,
              BLACK);

    next_frame().await
  }
}

// struct Spinner;

// // In
// impl System for Spinner {
//   fn run(&mut self, mut query: Query<(&mut Transform, &Node)>) {
//     for (transform, _node) in &mut query.iter() {
//       transform.rotate(Quat::from_rotation_x(0.01));
//       transform.rotate(Quat::from_rotation_y(0.01));
//     }
//   }
// }
// fn aaa() {
//   let n = new::<i32>();

//   let x = Option::Some(3);
//   if let Some(v) = x {
//     println!("yep");
//   }
// }

// struct Burrito {
//     meat: bool,
//     tomato: bool,
//     mayonnaise: bool,
// }
// struct Monad(Burrito);
// struct Lambda<A, B, T: Fn(A) -> B>(T);

// fn main() {
//     let m = Monad(Burrito { meat: true, tomato: true, mayonnaise: false });
//     let myapp = App::new().world;
//     ()
//     // "aaa"

//     // test();
// }
// type Coord = [Int; 2];
// #[derive(PartialEq, Clone, Copy, Hash)]
// struct Coord(i16, i16);
// enum ItemId {
//     Wood,
//     Sword,
//     Loot,
// }
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
