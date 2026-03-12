#![allow(non_snake_case)]
mod blackhole;
mod common;
mod grid;
mod ray;
mod step;

use nalgebra::Vector2;

use wasm_bindgen::prelude::*;

use crate::{blackhole::Blackhole, grid::Grid, ray::Ray};

const WIDTH: usize = 300;
const HEIGHT: usize = 300;

/// comunication with js front end
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

/// Rendering for the blackhole and rays.
/// and also managment of objects in the scene
#[wasm_bindgen]
pub struct Renderer {
    buffer: Vec<u8>,
    blackhole: Blackhole,
    rays: Vec<Ray>,
    scale: f64, // world units per pixel
    grid: Grid,
}

#[wasm_bindgen]
impl Renderer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Renderer {
        let scale = 1e9; // 1 pixel per

        let mut rays = Vec::new();
        let blackhole = Blackhole::new(Vector2::new(0.0, 0.0), 8.54e36); // Sagittarius A black hole
        #[allow(clippy::excessive_precision)]
        rays.push(Ray::new(
            // cool cycle
            Vector2::new(-1e11, 3.27608302719999999e10),
            Vector2::new(common::C, 0.0),
            blackhole.pos,
            blackhole.r_s,
        ));

        let grid = Grid::new(40.0 * scale, 20);
        Renderer {
            buffer: vec![0; WIDTH * HEIGHT * 4],
            blackhole,
            rays,
            scale,
            grid,
        }
    }
    // For js to access buffer
    pub fn buffer_ptr(&self) -> *const u8 {
        self.buffer.as_ptr()
    }

    pub fn add_ray_from_click(&mut self, mouse_x: f64, mouse_y: f64) -> Result<(), JsError> {
        if mouse_x < 0.0 || mouse_y < 0.0 {
            return Err(JsError::new("Click coordinates must be non-negative"));
        }
        // Convert screen (pixels) to world coordinates
        let center = Vector2::new(WIDTH as f64 / 2.0, HEIGHT as f64 / 2.0);

        console_log!("Spawning ray at {} {}", mouse_x, mouse_y);
        let screen = Vector2::new(mouse_x, mouse_y);

        let world_pos = (screen - center) * self.scale;
        if (world_pos - self.blackhole.pos).norm() <= self.blackhole.r_s {
            return Ok(());
        }
        let dir = Vector2::new(common::C, 0.0); // always spawn ray going left 

        let ray = Ray::new(world_pos, dir, self.blackhole.pos, self.blackhole.r_s);

        self.rays.push(ray);
        Ok(())
    }

    pub fn set_blackhole_mass(&mut self, mass: f64) -> Result<(), JsError> {
        if mass <= 0.0 {
            return Err(JsError::new("Black hole mass must be positive"));
        }
        if mass > 1e38 {
            return Err(JsError::new("Mass exceeds simulation limits"));
        }
        self.blackhole.mass = mass;
        self.blackhole.r_s = (2.0 * common::G * mass) / (common::C * common::C);
        Ok(())
    }

    pub fn update(&mut self) {
        // Clear screen
        for i in 0..WIDTH * HEIGHT {
            let idx = i * 4;
            self.buffer[idx] = 0;
            self.buffer[idx + 1] = 0;
            self.buffer[idx + 2] = 0;
            self.buffer[idx + 3] = 255;
        }

        self.grid.draw(
            &mut self.buffer,
            WIDTH,
            HEIGHT,
            self.scale,
            self.blackhole.r_s,
        );

        // retain_mut runs a function in a closure and removes from vec when false is returned
        self.rays.retain_mut(|ray| {
            let steps_per_frame = 20;
            let dt = 0.1; // step size 
            for _ in 0..steps_per_frame {
                ray.step(dt, &self.blackhole); // step function 
            }

            ray.draw_trail(&mut self.buffer, WIDTH, HEIGHT, self.scale);

            let max_radius = 3.0e11; // delete outside of this radius 

            ray.draw(&mut self.buffer, WIDTH, HEIGHT, self.scale);
            if (ray.pos - self.blackhole.pos).norm() > max_radius {
                console_log!("Deleting ray outside radius at {}", ray.pos);
                false
            } else {
                true
            }
        });

        self.blackhole
            .draw(&mut self.buffer, WIDTH, HEIGHT, self.scale);
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
