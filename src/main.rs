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

// /// set up a simple 3D scene
// fn setup(mut commands: Commands,
//          mut meshes: ResMut<Assets<Mesh>>,
//          mut materials: ResMut<Assets<StandardMaterial>>) {
//     // plane
//     commands.spawn(PbrBundle { mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
//                                material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
//                                ..default() });
//     // cube
//     commands.spawn(PbrBundle { mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
//                                material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
//                                transform: Transform::from_xyz(0.0, 0.5, 0.0),
//                                ..default() });
//     // light
//     commands.spawn(PointLightBundle { point_light: PointLight { intensity: 1500.0,
//                                                                 shadows_enabled: true,
//                                                                 ..default() },
//                                       transform: Transform::from_xyz(4.0, 8.0, 4.0),
//                                       ..default() });
//     // camera
//     commands.spawn(Camera3dBundle { transform:
//                                         Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
//                                     ..default() });
// }

// fn main() { App::new().add_plugins(DefaultPlugins).add_startup_system(setup).run(); }

fn new<T: Default>() -> T { T::default() }

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

// fn setup(mut commands: Commands,
//          asset_server: Res<AssetServer>,
//          mut materials: ResMut<Assets<ColorMaterial>>) {
//   let cube = Cube::new(Vec3::new(1.0, 1.0, 1.0));
//   let mesh = asset_server.load_asset::<Mesh>().add_cube(cube);
//   let material = materials.add(Color::rgb(0.9, 0.1, 0.1).into());
//   commands.spawn(Camera3d::default())
//           .spawn(LightComponents::default())
//           .spawn(NodeComponents { mesh: mesh.clone(),
//                                   material: material.clone(),
//                                   translation: Translation::new(0.0, 0.0, -3.0),
//                                   ..Default::default() })
//           .with(Spinner {});
// }

// fn add_people(mut commands: Commands) {
//   commands.spawn((Person, Name("Elaina Proctor".to_string())));
//   commands.spawn((Person, Name("Renzo Hume".to_string())));
//   commands.spawn((Person, Name("Zayna Nieves".to_string())));
// }

// fn main() {
//   // new::<App>().
//   App::new().add_plugins(DefaultPlugins)
//             .add_system(add_people)
//             .add_startup_system(setup)
//             .run();
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
