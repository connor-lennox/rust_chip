use core::fmt;

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

pub struct Display {
    pub data: [bool; 2048],
}

fn xy_to_idx(x: usize, y: usize) -> usize { x + (y * WIDTH) }

impl Display {
    pub fn new() -> Display {
        Display { data: [false; 2048] }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, state: bool) {
        self.data[xy_to_idx(x, y)] = state;
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> bool {
        self.data[xy_to_idx(x, y)]
    }

    fn flip_pixel(&mut self, x: usize, y: usize) -> bool {
        self.set_pixel(x, y, !self.get_pixel(x, y));
        !self.get_pixel(x, y)
    }

    pub fn draw(&mut self, x: usize, y: usize, sprite: &[u8]) -> bool {
        let mut collision: bool = false;
        let mut xm = x % WIDTH;
        let mut ym = y % HEIGHT;
        // Iterate lines of the sprite
        for line in sprite {
            for bit in 0..8 {
                let state = (line >> (7 - bit) & 0x1) == 1;
                if state {
                    // Flip collision flag if this bit was already on
                    collision = if self.flip_pixel(xm, ym) {true} else {collision}
                }
                xm += 1;
                if xm >= WIDTH { xm = 0; }
            }
            // Reset xm, progress ym (break if off screen)
            xm = x % WIDTH;
            ym += 1;
            if ym >= HEIGHT { break; }
        }
        collision
    }

    pub fn clear(&mut self) {
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                self.set_pixel(x, y, false);
            }
        }
    }
}

impl fmt::Display for Display {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                if self.get_pixel(x, y) {
                    write!(f, "█")?;
                } else {
                    write!(f, " ")?;
                }
            }
            write!(f, "\n")?;
        }
        fmt::Result::Ok(())
    }
}