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

// fn main() { Conf {} }

const MOVE_SPEED: f32 = 0.1;
const LOOK_SPEED: f32 = 0.1;

fn conf() -> Conf {
  Conf { window_title: String::from("Macroquad"),
         window_width: 1260,
         window_height: 768,
         fullscreen: false,
         ..Default::default() }
}

#[macroquad::main(conf)]
async fn main() {
  let mut x = 0.0;
  let mut switch = false;
  let bounds = 8.0;

  let world_up = vec3(0.0, 1.0, 0.0);
  let mut yaw: f32 = 1.18;
  let mut pitch: f32 = 0.0;

  let mut front = vec3(yaw.cos() * pitch.cos(), pitch.sin(), yaw.sin() * pitch.cos()).normalize();
  let mut right = front.cross(world_up).normalize();
  let mut up;

  let mut position = vec3(0.0, 1.0, 0.0);
  let mut last_mouse_position: Vec2 = mouse_position().into();

  let mut grabbed = true;
  set_cursor_grab(grabbed);
  show_mouse(false);

  loop {
    let delta = get_frame_time();

    if is_key_pressed(KeyCode::Escape) {
      break;
    }
    if is_key_pressed(KeyCode::Tab) {
      grabbed = !grabbed;
      set_cursor_grab(grabbed);
      show_mouse(!grabbed);
    }

    if is_key_down(KeyCode::Up) {
      position += front * MOVE_SPEED;
    }
    if is_key_down(KeyCode::Down) {
      position -= front * MOVE_SPEED;
    }
    if is_key_down(KeyCode::Left) {
      position -= right * MOVE_SPEED;
    }
    if is_key_down(KeyCode::Right) {
      position += right * MOVE_SPEED;
    }

    let mouse_position: Vec2 = mouse_position().into();
    let mouse_delta = mouse_position - last_mouse_position;
    last_mouse_position = mouse_position;

    yaw += mouse_delta.x * delta * LOOK_SPEED;
    pitch += mouse_delta.y * delta * -LOOK_SPEED;

    pitch = if pitch > 1.5 { 1.5 } else { pitch };
    pitch = if pitch < -1.5 { -1.5 } else { pitch };

    front = vec3(yaw.cos() * pitch.cos(), pitch.sin(), yaw.sin() * pitch.cos()).normalize();

    right = front.cross(world_up).normalize();
    up = right.cross(front).normalize();

    x += if switch { 0.04 } else { -0.04 };
    if x >= bounds || x <= -bounds {
      switch = !switch;
    }

    clear_background(LIGHTGRAY);

    // Going 3d!

    set_camera(&Camera3D { position: position,
                           up: up,
                           target: position + front,
                           ..Default::default() });

    draw_grid(20, 1., BLACK, GRAY);

    draw_line_3d(vec3(x, 0.0, x),
                 vec3(5.0, 5.0, 5.0),
                 Color::new(1.0, 1.0, 0.0, 1.0));

    draw_cube_wires(vec3(0., 1., -6.), vec3(2., 2., 2.), GREEN);
    draw_cube_wires(vec3(0., 1., 6.), vec3(2., 2., 2.), BLUE);
    draw_cube_wires(vec3(2., 1., 2.), vec3(2., 2., 2.), RED);

    // Back to screen space, render some text

    set_default_camera();
    draw_text("First Person Camera", 10.0, 20.0, 30.0, BLACK);

    draw_text(format!("X: {} Y: {}", mouse_position.x, mouse_position.y).as_str(),
              10.0,
              48.0 + 18.0,
              30.0,
              BLACK);
    draw_text(format!("Press <TAB> to toggle mouse grab: {}", grabbed).as_str(),
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
fn aaa() {
  let n = new::<i32>();

  let x = Option::Some(3);
  if let Some(v) = x {
    println!("yep");
  }
}

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
