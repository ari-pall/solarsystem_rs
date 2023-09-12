#![allow(unused_imports)]
#![allow(dead_code)]

use {macroquad::{camera::Camera3D,
                 miniquad::KeyCode,
                 models::{draw_mesh, Mesh, Vertex},
                 prelude::{Color, Vec2, Vec3, *},
                 rand::RandomRange},
     rust_utils::{filter, fold, iproduct, map, tap_mut},
     std::{collections::HashMap, hash::Hash, iter::IntoIterator}};

fn coolmesh() -> Mesh {
  // Vertex
  let rand_color = || Color::from_rgba(rng(0, 255), rng(0, 255), rng(0, 255), 155);
  let uv = Vec2::from([0.6, 0.7]);
  // let rng = || rand::gen_range(-5.0, 5.0);
  let rand_pos = || Vec3::from([rng(-5.0, 5.0), rng(-5.0, 5.0), rng(-5.0, 5.0)]);
  let rand_vertex = || Vertex { position: rand_pos(),
                                uv,
                                color: rand_color() };
  let len = 20;
  Mesh { vertices: (0..len).map(|_| rand_vertex()).collect(),
         indices: (0..len).collect(),
         texture: None }
}
// #[test]
fn draw_it() { draw_mesh(&coolmesh()); }

const MOVE_SPEED: f32 = 0.16;
const LOOK_SPEED: f32 = 0.001;

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
  pos: Vec3,
  vel: Vec3,
  color: Color,
  mass: f32
}

fn rng<T: RandomRange>(a: T, b: T) -> T { rand::gen_range(a, b) }
impl Planet {
  fn star() -> Planet {
    Planet { pos: Vec3 { x: 3.1,
                         y: 1.3,
                         z: 2.3 },
             vel: Vec3::default(),
             color: Color::from_rgba(255, 255, 0, 255),
             mass: 1.1 }
  }
  fn random() -> Planet {
    let r = rng(0.01, 0.99);
    let g = rng(0.01, 0.99);
    let b = rng(0.01, 0.99);
    let mass = (rng(0.0002, 0.6) as f32).powi(2);
    let speed = 0.03;
    Planet { color: Color { a: 1.0, r, g, b },
             pos: Vec3 { x: r, y: g, z: b } * 50.0 - Vec3::ONE * 25.0,
             vel: vec3(rng(-speed, speed), rng(-speed, speed), rng(-speed, speed)),
             mass }
  }
  fn radius(&self) -> f32 { self.mass.cbrt() }
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
impl Planets {
  fn get(&self, i: usize) -> Option<Planet> { self.0[i] }
  fn set(self, i: usize, op: Option<Planet>) -> Self { tap_mut(self, |s| s.0[i] = op) }
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
  position: Vec3,
  orientation: Quat,
  last_mouse_position: Vec2,
  grabbed: bool
}
impl Default for State {
  fn default() -> Self {
    Self { planets: Planets::default(),
           position: vec3(0.0, 1.0, 0.0),
           orientation: Quat::default(),
           last_mouse_position: mouse_position().into(),
           grabbed: true }
  }
}
impl State {
  fn update(self, poschange: Vec3, mouse_position: Vec2) -> Self {
    let Self { last_mouse_position,
               planets,
               orientation,
               position,
               .. } = self;
    let Vec2 { x, y } = (mouse_position - last_mouse_position) * LOOK_SPEED;

    Self { last_mouse_position: mouse_position,
           orientation: orientation.normalize()
                        * Quat::from_rotation_x(y)
                        * Quat::from_rotation_y(-x),
           planets: planets.gravity().movement().collisions(),
           position: position + poschange,
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
  let mesh = coolmesh();
  loop {
    if is_key_pressed(KeyCode::Escape) {
      break;
    }
    if is_key_pressed(KeyCode::Tab) {
      state.grabbed = !state.grabbed;
      set_cursor_grab(state.grabbed);
      show_mouse(!state.grabbed);
    }

    let front = state.orientation * Vec3::Z;
    let up = state.orientation * Vec3::Y;
    let left = state.orientation * Vec3::X;
    let poschange = MOVE_SPEED
                    * sum(map(|(_, dir)| dir,
                              filter(|(key, _)| is_key_down(*key),
                                     [(KeyCode::W, front),
                                      (KeyCode::A, left),
                                      (KeyCode::S, -front),
                                      (KeyCode::D, -left),
                                      (KeyCode::LeftShift, up),
                                      (KeyCode::LeftControl, -up)])));
    let mouse_position = Vec2::from(mouse_position());
    state = state.update(poschange, mouse_position);
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
    draw_mesh(&mesh);

    draw_grid(100, 2.0, DARKGRAY, DARKGRAY);

    for Planet { pos, color, mass, .. } in planets.0.iter().filter_map(ToOwned::to_owned) {
      draw_sphere(pos, mass.cbrt(), None, color)
    }

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
