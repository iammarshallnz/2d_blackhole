mod common;

use nalgebra::Vector2;
use std::collections::VecDeque;
use wasm_bindgen::prelude::*;

const WIDTH: usize = 300;
const HEIGHT: usize = 300;

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello, {}!", name));
}

struct Blackhole {
    pos: Vector2<f64>,
    mass: f64,
    r_s: f64,
}
impl Blackhole {
    fn new(pos: Vector2<f64>, mass: f64) -> Blackhole {
        Blackhole {
            pos,
            mass,
            r_s: (2.0 * common::G * mass) / (common::C * common::C),
        }
    }
    fn draw(
        &self,
        buffer: &mut [u8],
        width: usize,
        height: usize,
        scale: f64,
        offset: Vector2<f64>,
    ) {
        let cx = ((self.pos.x - offset.x) / scale) as i32 + (width / 2) as i32;
        let cy = ((self.pos.y - offset.y) / scale) as i32 + (height / 2) as i32;
        let radius = (self.r_s / scale) as i32;

        for y in -radius..=radius {
            for x in -radius..=radius {
                if x * x + y * y <= radius * radius {
                    let px = cx + x;
                    let py = cy + y;

                    if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                        let idx = (py as usize * width + px as usize) * 4;

                        buffer[idx] = 255;
                        buffer[idx + 1] = 0;
                        buffer[idx + 2] = 0;
                        buffer[idx + 3] = 255;
                    }
                }
            }
        }
    }
}

struct Ray {
    // Cartesian for rendering
    pub pos: Vector2<f64>,

    // Polar for physics
    pub r: f64,
    pub phi: f64,
    pub dr: f64,
    pub dphi: f64,

    // Trail of positions
    pub trail: VecDeque<Vector2<f64>>, // could limit size

    // Conserved quantities
    pub E: f64,
    pub L: f64,
}

impl Ray {
    pub fn new(pos: Vector2<f64>, dir: Vector2<f64>, bh_pos: Vector2<f64>, r_s: f64) -> Ray {
        let rel = pos - bh_pos; // relative to black hole
        let x = rel.x;
        let y = rel.y;
        let r = (x * x + y * y).sqrt();
        let phi = y.atan2(x);

        let dr = dir.x * phi.cos() + dir.y * phi.sin();
        let dphi = (-dir.x * phi.sin() + dir.y * phi.cos()) / r;

        let L = r * r * dphi;
        let f = 1.0 - r_s / r;
        let dt_dlambda = ((dr * dr) / (f * f) + (r * r * dphi * dphi) / f).sqrt();
        let E = f * dt_dlambda;

        let mut trail = VecDeque::new();
        trail.push_back(pos);

        Ray {
            pos,
            r,
            phi,
            dr,
            dphi,
            trail,
            E,
            L,
        }
    }
    fn draw(
        &mut self,
        buffer: &mut [u8],
        width: usize,
        height: usize,
        scale: f64,
        offset: Vector2<f64>,
    ) {
        let x = ((self.pos.x - offset.x) / scale) as i32 + (width / 2) as i32;
        let y = ((self.pos.y - offset.y) / scale) as i32 + (height / 2) as i32;
        // push to trail (limit size for performance)
        const MAX_TRAIL: usize = 200;
        self.trail.push_back(self.pos);
        if self.trail.len() > MAX_TRAIL {
            self.trail.pop_front();
        }
        if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
            let idx = (y as usize * width + x as usize) * 4;

            buffer[idx] = 255; // R
            buffer[idx + 1] = 255; // G
            buffer[idx + 2] = 255; // B
            buffer[idx + 3] = 255; // A
        }
    }
    fn step(&mut self, d_lambda: f64, bh: &Blackhole) {
        if (self.pos - bh.pos).norm() <= bh.r_s {
            return;
        } // inside event horizon

        // RK4 integration
        rk4_step(self, d_lambda, bh.pos, bh.r_s);

        // update Cartesian for drawing
        self.pos.x = self.r * self.phi.cos() + bh.pos.x;
        self.pos.y = self.r * self.phi.sin() + bh.pos.y;
    }

    fn draw_trail(
        &self,
        buffer: &mut [u8],
        width: usize,
        height: usize,
        scale: f64,
        offset: Vector2<f64>,
    ) {
        let size = self.trail.len();
        for (index, point) in self.trail.iter().enumerate() {
            let x = ((point.x - offset.x) / scale) as i32 + (width / 2) as i32;
            let y = ((point.y - offset.y) / scale) as i32 + (height / 2) as i32;
            let ratio: f64 = index as f64 / size as f64;
            if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                let idx = (y as usize * width + x as usize) * 4;

                buffer[idx] = (200.0 * ratio) as u8;
                buffer[idx + 1] = (200.0 * ratio) as u8;
                buffer[idx + 2] = (200.0 * ratio) as u8;
                buffer[idx + 3] = 255;
            }
        }
    }
}

fn geodesic_rhs(ray: &Ray, bh_pos: Vector2<f64>, r_s: f64) -> [f64; 4] {
    let rel = ray.pos - bh_pos;
    let r = (rel.x * rel.x + rel.y * rel.y).sqrt();
    let dr = ray.dr;
    let dphi = ray.dphi;
    let E = ray.E;

    let f = 1.0 - r_s / r;

    let dt_dlambda = E / f;

    let d2r = -(r_s / (2.0 * r * r)) * f * (dt_dlambda * dt_dlambda)
        + (r_s / (2.0 * r * r * f)) * (dr * dr)
        + (r - r_s) * (dphi * dphi);

    let d2phi = -2.0 * dr * dphi / r;

    [dr, dphi, d2r, d2phi]
}

fn add_state(a: &[f64; 4], b: &[f64; 4], factor: f64) -> [f64; 4] {
    [
        a[0] + b[0] * factor,
        a[1] + b[1] * factor,
        a[2] + b[2] * factor,
        a[3] + b[3] * factor,
    ]
}

fn rk4_step(ray: &mut Ray, d_lambda: f64, bh_pos: Vector2<f64>, r_s: f64) {
    let y0 = [ray.r, ray.phi, ray.dr, ray.dphi];

    let k1 = geodesic_rhs(ray, bh_pos, r_s);

    let temp = add_state(&y0, &k1, d_lambda / 2.0);
    let r2 = Ray {
        pos: ray.pos,
        r: temp[0],
        phi: temp[1],
        dr: temp[2],
        dphi: temp[3],
        trail: VecDeque::new(),
        E: ray.E,
        L: ray.L,
    };
    let k2 = geodesic_rhs(&r2, bh_pos, r_s);

    let temp = add_state(&y0, &k2, d_lambda / 2.0);
    let r3 = Ray {
        pos: ray.pos,
        r: temp[0],
        phi: temp[1],
        dr: temp[2],
        dphi: temp[3],
        trail: VecDeque::new(),
        E: ray.E,
        L: ray.L,
    };
    let k3 = geodesic_rhs(&r3, bh_pos, r_s);

    let temp = add_state(&y0, &k3, d_lambda);
    let r4 = Ray {
        pos: ray.pos,
        r: temp[0],
        phi: temp[1],
        dr: temp[2],
        dphi: temp[3],
        trail: VecDeque::new(),
        E: ray.E,
        L: ray.L,
    };
    let k4 = geodesic_rhs(&r4, bh_pos, r_s);

    ray.r += d_lambda / 6.0 * (k1[0] + 2.0 * k2[0] + 2.0 * k3[0] + k4[0]);
    ray.phi += d_lambda / 6.0 * (k1[1] + 2.0 * k2[1] + 2.0 * k3[1] + k4[1]);
    ray.dr += d_lambda / 6.0 * (k1[2] + 2.0 * k2[2] + 2.0 * k3[2] + k4[2]);
    ray.dphi += d_lambda / 6.0 * (k1[3] + 2.0 * k2[3] + 2.0 * k3[3] + k4[3]);
}

#[wasm_bindgen]
pub struct Renderer {
    buffer: Vec<u8>,
    blackhole: Blackhole,
    rays: Vec<Ray>,
    scale: f64,           // world units per pixel
    offset: Vector2<f64>, // world position at the center of the screen
}

#[wasm_bindgen]
impl Renderer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Renderer {
        let scale = 1e9; // 1 pixel per
        let offset = Vector2::new(150.0, 150.0); // center in world coordinates

        let mut rays = Vec::new();
        let blackhole = Blackhole::new(Vector2::new(0.0, 0.0), 8.54e36);
        rays.push(Ray::new( // cool cycle
                Vector2::new(-1e11,  3.28409215719999999e10), 
                Vector2::new(common::C, 0.0),
                blackhole.pos,
                blackhole.r_s,
            ));
        
        Renderer {
            buffer: vec![0; WIDTH * HEIGHT * 4],
            blackhole: blackhole,
            rays,
            scale,
            offset,
        }
    }

    pub fn buffer_ptr(&self) -> *const u8 {
        self.buffer.as_ptr()
    }

    pub fn update(&mut self) {
        // Clear
        for i in 0..WIDTH * HEIGHT {
            let idx = i * 4;
            self.buffer[idx] = 0;
            self.buffer[idx + 1] = 0;
            self.buffer[idx + 2] = 0;
            self.buffer[idx + 3] = 255;
        }

        for ray in &mut self.rays {
            let steps_per_frame = 2;
            let dt = 1.0;
            for _ in 0..steps_per_frame {
                ray.step(dt, &self.blackhole);
            }
            ray.draw_trail(&mut self.buffer, WIDTH, HEIGHT, self.scale, self.offset);
            ray.draw(&mut self.buffer, WIDTH, HEIGHT, self.scale, self.offset);
        }

        self.blackhole
            .draw(&mut self.buffer, WIDTH, HEIGHT, self.scale, self.offset);
    }
}
