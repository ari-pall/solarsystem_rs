#![allow(unused_imports)]
#![allow(dead_code)]

use {derive_more::From,
     macroquad::{color,
                 miniquad::KeyCode,
                 prelude::*,
                 rand::{ChooseRandom, RandomRange}},
     rust_utils::{filter, iproduct, map, prelude::Tap},
     std::{boxed::Box,
           collections::HashMap,
           hash::Hash,
           iter::{IntoIterator, Map},
           num,
           ops::{Add, Deref, DerefMut, Not},
           str::{FromStr, SplitAsciiWhitespace},
           string,
           vec::{IntoIter, Vec}}};

fn new<T: Default>() -> T { T::default() }

const MOVE_SPEED: f32 = 0.16;
const LOOK_SPEED: f32 = 0.0017;
const WORLD_UP: Vec3 = vec3(0.0, 1.0, 0.0);

fn conf() -> Conf {
  Conf { fullscreen: true,
         ..Default::default() }
}

fn vec3_from_spherical_coords(yaw: f32, pitch: f32) -> Vec3 {
  vec3(yaw.cos() * pitch.cos(),
       pitch.sin(),
       yaw.sin() * pitch.cos()).normalize()
}
#[derive(Copy, Clone)]
struct Planet {
  pos: DVec3,
  vel: DVec3,
  color: Color,
  mass: f64
}
impl Planet {
  fn star() -> Planet {
    Planet { pos: DVec3 { x: 3.1,
                          y: 1.3,
                          z: 2.3 },
             vel: DVec3::default(),
             color: Color::from_rgba(255, 255, 0, 255),
             mass: 1.1 }
  }
  fn random() -> Planet {
    fn rng<T: RandomRange>(a: T, b: T) -> T { rand::gen_range(a, b) }
    let r = rng(0.01, 0.99);
    let g = rng(0.01, 0.99);
    let b = rng(0.01, 0.99);
    let mass = (rng(0.0002, 0.6) as f64).powi(2);
    let speed = 0.03;
    Planet { color: Color { a: 1.0, r, g, b },
             pos: dvec3(r as f64 * 50.0 - 25.0,
                        g as f64 * 50.0 - 25.0,
                        b as f64 * 50.0 - 25.0),
             vel: dvec3(rng(-speed, speed), rng(-speed, speed), rng(-speed, speed)),
             mass }
  }
  fn radius(&self) -> f64 { self.mass.cbrt() }
}
fn hashmap<K: Eq + Hash, V>(coll: impl IntoIterator<Item = (K, V)>) -> HashMap<K, V> {
  coll.into_iter().collect()
}
const NUM_PLANETS: usize = 60;
struct Planets([Option<Planet>; NUM_PLANETS]);
impl Default for Planets {
  fn default() -> Self {
    Self([Some(Planet::random()); NUM_PLANETS].map(|_| Some(Planet::random())))
  }
}
pub fn fold<T, G>(f: impl Fn(T, G) -> T, init: T, coll: impl IntoIterator<Item = G>) -> T {
  coll.into_iter().fold(init, f)
}
impl Planets {
  fn get(&self, i: usize) -> Option<Planet> { self.0[i] }
  fn set(self, i: usize, op: Option<Planet>) -> Self { self.tap_mut(|s| s.0[i] = op) }
  fn movement(self) -> Self {
    Self(self.0.map(|op| {
                 op.map(|p| Planet { pos: p.pos + p.vel,
                                     ..p })
               }))
  }
  fn gravity(self) -> Self {
    fold(|p, (i, j)| {
           if let (Some(pi), Some(pj)) = (p.get(i), p.get(j)) {
             let posdiff = pj.pos - pi.pos;
             let dist = posdiff.length();
             let g = 0.007;
             p.set(i,
                   Some(Planet { vel: pi.vel + g * pj.mass * posdiff / dist.powi(3),
                                 ..pi }))
              .set(j,
                   Some(Planet { vel: pj.vel - g * pi.mass * posdiff / dist.powi(3),
                                 ..pj }))
           } else {
             p
           }
         },
         self,
         iproduct!(0..NUM_PLANETS, 0..NUM_PLANETS).filter(|(i, j)| i < j))
  }
  fn collisions(self) -> Self {
    fold(|p, (i, j)| match (p.get(i), p.get(j)) {
           (Some(pi), Some(pj)) if pi.pos.distance(pj.pos) < pi.radius() + pj.radius() => {
             let total_mass = pi.mass + pj.mass;
             p.set(i,
                   Some(Planet { pos: (pi.pos * pi.mass + pj.pos * pj.mass) / total_mass,
                                 vel: (pi.vel * pi.mass + pj.vel * pj.mass) / total_mass,
                                 color: Color { r: pi.color.r.max(pj.color.r),
                                                g: pi.color.g.max(pj.color.g),
                                                b: pi.color.b.max(pj.color.b),
                                                a: 1.0 },
                                 mass: total_mass }))
              .set(j, None)
           }
           _ => p
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
  fn update(self, change: Vec3, mouse_position: Vec2) -> Self {
    let Self { last_mouse_position,
               pitch,
               yaw,
               planets,
               position,
               .. } = self;
    let mouse_delta = mouse_position - last_mouse_position;

    Self { last_mouse_position: mouse_position,
           pitch: (pitch - mouse_delta.y * LOOK_SPEED).clamp(-1.5, 1.5),
           yaw: yaw + mouse_delta.x * LOOK_SPEED,
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
                 * sum(map(|(_, dir)| dir,
                           filter(|(key, _)| is_key_down(*key),
                                  [(KeyCode::W, front),
                                   (KeyCode::A, -right),
                                   (KeyCode::S, -front),
                                   (KeyCode::D, right),
                                   (KeyCode::LeftShift, up),
                                   (KeyCode::LeftControl, -up)])));
    let mouse_position = Vec2::from(mouse_position());
    state = state.update(change, mouse_position);
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
                                           mass,
                                           .. }) = p
                      {
                        draw_sphere(Vec3 { x: x as f32,
                                           y: y as f32,
                                           z: z as f32 },
                                    mass.cbrt() as f32,
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
