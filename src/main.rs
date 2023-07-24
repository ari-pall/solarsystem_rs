#![allow(unused_imports)]
#![allow(dead_code)]

use {derive_more::From,
     itertools::{iproduct, Combinations, Itertools},
     // iter_comprehensions::sum,
     list_comprehension::comp,
     macroquad::{color, miniquad::KeyCode, prelude::*},
     std::{boxed::Box,
           collections::HashSet,
           collections::{BTreeMap, HashMap},
           convert::identity,
           fmt::{Debug, Display, Result},
           hash::Hash,
           io::{self, stdin, stdout, BufRead, Error, Lines, Read, StdinLock, Write},
           iter::{IntoIterator, Map},
           num,
           ops::{Add, Deref, DerefMut, Div, MulAssign, Not, Rem, Sub},
           str::{FromStr, SplitAsciiWhitespace},
           string,
           vec::{IntoIter, Vec}}};

fn new<T: Default>() -> T { T::default() }
fn not(v: bool) -> bool { v.not() }

const MOVE_SPEED: f32 = 0.12;
const LOOK_SPEED: f32 = 0.08;
const WORLD_UP: Vec3 = Vec3::new(0.0, 1.0, 0.0);

fn conf() -> Conf {
  Conf { window_title: String::from("solar system"),
         fullscreen: true,
         ..Default::default() }
}

fn vec3_from_spherical_coords(yaw: f32, pitch: f32) -> Vec3 {
  vec3(yaw.cos() * pitch.cos(), pitch.sin(), yaw.sin() * pitch.cos()).normalize()
}
#[derive(Copy, Clone)]
struct Planet {
  pos: DVec3,
  vel: DVec3,
  color: Color,
  radius: f64
}
impl Planet {
  fn star() -> Planet {
    Planet { pos: DVec3 { x: 3.1,
                          y: 1.3,
                          z: 2.3 },
             vel: DVec3::default(),
             color: Color::from_rgba(255, 255, 0, 255),
             radius: 1.3 }
  }
  fn random() -> Planet {
    let rng = |a, b| rand::gen_range::<f32>(a, b);
    let r = rng(0.1, 0.9);
    let g = rng(0.1, 0.9);
    let b = rng(0.1, 0.9);
    Planet { color: Color { a: 1.0, r, g, b },
             pos: dvec3(r as f64 * 50.0 - 25.0,
                        g as f64 * 50.0 - 25.0,
                        b as f64 * 50.0 - 25.0),
             vel: dvec3(rng(-0.03, 0.03) as f64,
                        rng(-0.03, 0.03) as f64,
                        rng(-0.03, 0.03) as f64),
             radius: rng(0.1, 0.4) as f64 }
  }
}

fn hashmap<K: Eq + Hash, V>(coll: impl IntoIterator<Item = (K, V)>) -> HashMap<K, V> {
  coll.into_iter().collect()
}
const NUM_PLANETS: usize = 80;
struct Planets(HashMap<usize, Planet>);
impl Default for Planets {
  fn default() -> Self {
    Self(hashmap(comp!(
      (i, Planet::random()), i in 0..NUM_PLANETS)))
  }
}
pub fn fold<T, G>(f: impl Fn(T, G) -> T, init: T, coll: impl IntoIterator<Item = G>) -> T {
  coll.into_iter().fold(init, f)
}
impl Planets {
  fn movement(mut self) -> Self {
    self.0.values_mut().for_each(|planet| {
                         planet.pos += planet.vel;
                       });
    self
  }
  fn gravity(self) -> Self {
    let keypairs = comp!((i.clone(), j.clone()), &i in self.0.keys(), &j in self.0.keys(), i!=j);
    fold(|Self(mut p), (i, j)| {
           let pi = p.get(&i).unwrap();
           let pj = p.get(&j).unwrap();
           let posdiff = pj.pos - pi.pos;
           let dist = posdiff.length();
           let pjmass = pj.radius.powi(3);
           let g = 0.027;
           p.get_mut(&i).unwrap().vel += g * pjmass * posdiff / dist.powi(3);
           Self(p)
         },
         self,
         keypairs)
  }
  fn collisions(self) -> Self {
    let keypairs = comp!((i.clone(), j.clone()), &i in self.0.keys(), &j in self.0.keys(), i<j);
    fold(|Self(mut p), (i, j)| {
           let pi = p.get(&i);
           let pj = p.get(&j);
           match (pi, pj) {
             (Some(pi), Some(pj)) => {
               let posdiff = pj.pos - pi.pos;
               let dist = posdiff.length();
               if dist < pi.radius + pj.radius {
                 let pimass = pi.radius.powi(3);
                 let pjmass = pj.radius.powi(3);
                 let total_mass = pimass + pjmass;
                 let pos = (pi.pos * pimass + pj.pos * pjmass) / total_mass;
                 let vel = (pi.vel * pimass + pj.vel * pjmass) / total_mass;
                 let r = pi.color.r.max(pj.color.r);
                 let g = pi.color.g.max(pj.color.g);
                 let b = pi.color.b.max(pj.color.b);
                 let color = Color { r, g, b, a: 1.0 };
                 let radius = total_mass.cbrt();
                 p.insert(i, Planet { pos,
                                      vel,
                                      color,
                                      radius });

                 p.remove(&j);
               }
             }
             _ => ()
           }
           Self(p)
         },
         self,
         keypairs)
  }
}

struct State {
  planets: Planets,
  yaw: f32,
  pitch: f32,
  position: Vec3,
  orientation: Quat,
  last_mouse_position: Vec2,
  grabbed: bool
}
impl Default for State {
  fn default() -> Self {
    Self { planets: Planets::default(),
           yaw: 1.18,
           pitch: 1.18,
           position: vec3(0.0, 1.0, 0.0),
           orientation: Quat::default(),
           last_mouse_position: mouse_position().into(),
           grabbed: true }
  }
}
impl State {
  fn update(self, change: Vec3, mouse_position: Vec2, delta: f32) -> Self {
    let Self { last_mouse_position,
               pitch,
               yaw,
               planets,
               position,
               .. } = self;
    let mouse_delta = mouse_position - last_mouse_position;

    Self { last_mouse_position: mouse_position,
           pitch: (pitch + mouse_delta.y * delta * -LOOK_SPEED).clamp(-1.5, 1.5),
           yaw: yaw + mouse_delta.x * delta * LOOK_SPEED,
           planets: planets.gravity().movement().collisions(),
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
    let front = vec3_from_spherical_coords(state.yaw, state.pitch);
    let right = front.cross(WORLD_UP).normalize();
    let up = right.cross(front).normalize();
    let change = MOVE_SPEED
                 * sum(comp!(dir, (key, dir) in [(KeyCode::W, front),
                                                 (KeyCode::A, -right),
                                                 (KeyCode::S, -front),
                                                 (KeyCode::D, right),
                                                 (KeyCode::LeftShift, up),
                                                 (KeyCode::LeftControl, -up)]
                             , is_key_down(key)));
    let mouse_position = Vec2::from(mouse_position());
    state = state.update(change, mouse_position, delta);
    let State { planets,
                position,
                grabbed,
                .. } = &state;

    clear_background(BLACK);

    // Going 3d!

    set_camera(&Camera3D { position: *position,
                           up,
                           target: *position + front,
                           ..Default::default() });

    draw_grid(20, 1., DARKGRAY, GRAY);

    for (..,
         Planet { pos: DVec3 { x, y, z },
                  color,
                  radius,
                  .. }) in &planets.0
    {
      draw_sphere(Vec3 { x: *x as f32,
                         y: *y as f32,
                         z: *z as f32 },
                  *radius as f32,
                  None,
                  *color);
    }

    // Back to screen space, render some text

    set_default_camera();
    draw_text(format!("Press <TAB> to toggle mouse grab: {}", grabbed).as_str(),
              10.0,
              48.0 + 42.0,
              30.0,
              WHITE);

    next_frame().await
  }
}
