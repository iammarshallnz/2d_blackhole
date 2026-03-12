use nalgebra::Vector2;

pub struct Grid {
    lines: u32,
    points: Vec<Vector2<f64>>,
}

impl Grid {
    pub fn new(spacing: f64, lines: u32) -> Grid {
        let mut points = Vec::new();
        let half = lines as i32 / 2; // each side 

        // n x n
        for j in -half..=half {
            for i in -half..=half {
                points.push(Vector2::new(i as f64 * spacing, j as f64 * spacing));
            }
        }

        Grid { lines, points }
    }

    pub fn warp_points(&self, r_s: f64) -> Vec<Vector2<f64>> {
        self.points
            .iter()
            .map(|rel| {
                let r = rel.norm();
                // zero division
                if r < 0.1 {
                    return *rel;
                }
                let warp = r_s / r;
                rel * (1.0 - warp) // move towards center 
            })
            .collect()
    }

    pub fn draw(&self, buffer: &mut [u8], width: usize, height: usize, scale: f64, r_s: f64) {
        let warped = self.warp_points(r_s);
        let cols = self.lines as usize + 1;

        // n x n
        for j in 0..cols {
            for i in 0..cols {
                let idx = j * cols + i;
                // horizontal line
                if i + 1 < cols {
                    self.draw_line(buffer, width, height, scale, warped[idx], warped[idx + 1]);
                }
                // vertical line
                if j + 1 < cols {
                    self.draw_line(
                        buffer,
                        width,
                        height,
                        scale,
                        warped[idx],
                        warped[(j + 1) * cols + i],
                    );
                }
            }
        }
    }
    
    fn draw_line(
        &self,
        buffer: &mut [u8],
        width: usize,
        height: usize,
        scale: f64,
        a: Vector2<f64>,
        b: Vector2<f64>,
    ) {
        // convert from world coord to pixel pos 
        let screen_a = (a) / scale + Vector2::new(width as f64 / 2.0, height as f64 / 2.0);
        let (x0, y0) = (screen_a.x as i32, screen_a.y as i32);
        let screen_b = (b) / scale + Vector2::new(width as f64 / 2.0, height as f64 / 2.0);
        let (x1, y1) = (screen_b.x as i32, screen_b.y as i32);

        // https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm#All_cases
        // Using Bresenham's principles ??? if it works dont touch it
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let mut x = x0;
        let mut y = y0;

        loop {
            if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                let idx = (y as usize * width + x as usize) * 4;
                buffer[idx] = 40;
                buffer[idx + 1] = 40;
                buffer[idx + 2] = 60;
                buffer[idx + 3] = 255;
            }

            let e2 = 2 * err;
            if e2 >= dy {
                if x == x1 {
                    break;
                }
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                if y == y1 {
                    break;
                }
                err += dx;
                y += sy;
            }
        }
    }
}
