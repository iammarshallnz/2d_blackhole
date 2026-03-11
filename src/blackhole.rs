use nalgebra::Vector2;
use crate::common;

/// Blackhole struct
/// Can only have 1 blackhole due to using non Newtonian method for the step

pub struct Blackhole {
    pub pos: Vector2<f64>,
    pub mass: f64,
    pub r_s: f64, // event horizon
}
impl Blackhole {
    pub fn new(pos: Vector2<f64>, mass: f64) -> Blackhole {
        Blackhole {
            pos,
            mass,
            r_s: (2.0 * common::G * mass) / (common::C * common::C),
        }
    }
    pub fn draw(
        &self,
        buffer: &mut [u8],
        width: usize,
        height: usize,
        scale: f64,
        offset: Vector2<f64>,
    ) {

        let center_x = ((self.pos.x - offset.x) / scale) as i32 + (width / 2) as i32;
        let center_y = ((self.pos.y - offset.y) / scale) as i32 + (height / 2) as i32;
        let radius = (self.r_s / scale) as i32;

        for y in -radius..=radius {
            for x in -radius..=radius {
                // x^2 + y^2 = r^2
                if x * x + y * y <= radius * radius {
                    let px = center_x + x;
                    let py = center_y + y;

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
