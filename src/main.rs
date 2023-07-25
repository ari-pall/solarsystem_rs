#![allow(unused_imports)]
#![allow(dead_code)]
#![feature(let_chains)]

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
const WORLD_UP: Vec3 = vec3(0.0, 1.0, 0.0);

fn conf() -> Conf {
  Conf { fullscreen: true,
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
    let rng = |a, b| rand::gen_range::<f64>(a, b);
    let r = rng(0.1, 0.99) as f32;
    let g = rng(0.1, 0.99) as f32;
    let b = rng(0.1, 0.99) as f32;
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
// fn collect<C:FromIterator< >>()
const NUM_PLANETS: usize = 60;
struct Planets([Option<Planet>; NUM_PLANETS]);
impl Default for Planets {
  fn default() -> Self { Self([Some(Planet::random()); NUM_PLANETS].map(|_| Some(Planet::random()))) }
}
pub fn fold<T, G>(f: impl Fn(T, G) -> T, init: T, coll: impl IntoIterator<Item = G>) -> T {
  coll.into_iter().fold(init, f)
}
impl Planets {
  fn movement(self) -> Self {
    Self(self.0.map(|op| {
                 op.map(|p| Planet { pos: p.pos + p.vel,
                                     ..p })
               }))
  }
  fn gravity(self) -> Self {
    fold(|Self(mut p), (i, j)| {
           if let (Some(pi), Some(pj)) = (p[i], p[j]) {
             let posdiff = pj.pos - pi.pos;
             let dist = posdiff.length();
             let pimass = pi.radius.powi(3);
             let pjmass = pj.radius.powi(3);
             let g = 0.037;
             p[i] = Some(Planet { vel: pi.vel + g * pjmass * posdiff / dist.powi(3),
                                  ..pi });
             p[j] = Some(Planet { vel: pj.vel - g * pimass * posdiff / dist.powi(3),
                                  ..pj });
           }
           Self(p)
         },
         self,
         iproduct!(0..NUM_PLANETS, 0..NUM_PLANETS).filter(|(i, j)| i < j))
  }
  fn collisions(self) -> Self {
    fold(|Self(mut p), (i, j)| {
           if let (Some(pi), Some(pj)) = (p[i], p[j]) && pi.pos.distance(pj.pos) < pi.radius + pj.radius {
          let pimass = pi.radius.powi(3);
          let pjmass = pj.radius.powi(3);
          let total_mass = pimass + pjmass;
          p[j] = None;
          p[i] = Some(Planet { pos: (pi.pos * pimass + pj.pos * pjmass) / total_mass,
                               vel: (pi.vel * pimass + pj.vel * pjmass) / total_mass,
                               color:Color { r: pi.color.r.max(pj.color.r),
                                             g: pi.color.g.max(pj.color.g),
                                             b: pi.color.b.max(pj.color.b),
                                             a: 1.0 },
                               radius: total_mass.cbrt() });
        }
           Self(p)
         },
         self,
         iproduct!(0..NUM_PLANETS, 0..NUM_PLANETS).filter(|(i, j)| i < j))
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

    draw_grid(100, 2.0, DARKGRAY, DARKGRAY);

    planets.0.iter().for_each(|&p| {
                      if let Some(Planet { pos: DVec3 { x, y, z },
                                           color,
                                           radius,
                                           .. }) = p
                      {
                        draw_sphere(Vec3 { x: x as f32,
                                           y: y as f32,
                                           z: z as f32 },
                                    radius as f32,
                                    None,
                                    color)
                      }
                    });

    // Back to screen space, render some text

    set_default_camera();
    draw_text(format!("Press <TAB> to toggle mouse grab: {grabbed}").as_str(),
              10.0,
              48.0 + 42.0,
              30.0,
              WHITE);

    next_frame().await
  }
}
