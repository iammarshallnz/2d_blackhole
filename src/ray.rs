use nalgebra::{Rotation2, Vector2};
use std::collections::VecDeque;
use crate::blackhole::Blackhole;
use crate::step::rk4_step;

/// Ray struct
/// 

pub struct Ray {
    // -- Cartesian coords -- //
    pub pos: Vector2<f64>,

    // -- Polar coords -- //
    pub r: f64,
    pub phi: f64,
    pub dr: f64,
    pub dphi: f64,

    // Trail of points
    pub trail: VecDeque<Vector2<f64>>,

    // Conserved quantities
    pub E: f64,
}

impl Ray {
    pub fn new(pos: Vector2<f64>, dir: Vector2<f64>, bh_pos: Vector2<f64>, r_s: f64) -> Ray {
        let rel = pos - bh_pos; // relative to black hole
        let x = rel.x;
        let y = rel.y;
        let r = rel.norm();
        let phi = y.atan2(x);

        let radial = rel.normalize();
        let tangential = Vector2::new(-radial.y, radial.x);

        let dr = dir.dot(&radial);
        let dphi = dir.dot(&tangential) / r;

        let f = 1.0 - r_s / r;
        let dt_dλ = ((dr * dr) / (f * f) + (r * r * dphi * dphi) / f).sqrt();
        let E = f * dt_dλ;

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
        }
    }
    pub fn draw(
        &mut self,
        buffer: &mut [u8],
        width: usize,
        height: usize,
        scale: f64,
        offset: Vector2<f64>,
    ) {
        // push to trail
        const MAX_TRAIL: usize = 200;
        self.trail.push_back(self.pos);
        if self.trail.len() > MAX_TRAIL {
            self.trail.pop_front();
        }

        let ray = (self.pos - offset) / scale;
        let pixel = ray + Vector2::new(width as f64 / 2.0, height as f64 / 2.0);
        

        if pixel.x >= 0.0 && pixel.x < width as f64 && pixel.y >= 0.0 && pixel.y < height as f64 {
            let idx = (pixel.y as usize * width + pixel.x as usize) * 4;

            buffer[idx] = 255; // R
            buffer[idx + 1] = 255; // G
            buffer[idx + 2] = 255; // B
            buffer[idx + 3] = 255; // A
        }
    }
    pub fn step(&mut self, d_λ: f64, bh: &Blackhole) {
        if (self.pos - bh.pos).norm() <= bh.r_s {
            return;
        } // inside event horizon no light escapes 

        // RK4 
        rk4_step(self, d_λ, bh.r_s);

        // polar to cartesian
        let rot = Rotation2::new(self.phi);
        let cartesian = rot * Vector2::new(self.r, 0.0);

        self.pos = cartesian + bh.pos; // relitive to blackhole 
    }

    pub fn draw_trail(
        &self,
        buffer: &mut [u8],
        width: usize,
        height: usize,
        scale: f64,
        offset: Vector2<f64>,
    ) {
        let size = self.trail.len();
        for (index, point) in self.trail.iter().enumerate() {
            let trail = (point - offset) / scale;
            let pixel = trail + Vector2::new(width as f64 / 2.0, height as f64 / 2.0);
            let ratio: f64 = index as f64 / size as f64;
            if pixel.x >= 0.0 && pixel.x < width as f64 && pixel.y >= 0.0 && pixel.y < height as f64
            {
                let idx = (pixel.y as usize * width + pixel.x as usize) * 4;

                buffer[idx] = (200.0 * ratio) as u8;
                buffer[idx + 1] = (200.0 * ratio) as u8;
                buffer[idx + 2] = (200.0 * ratio) as u8;
                buffer[idx + 3] = 255;
            }
        }
    }
}
