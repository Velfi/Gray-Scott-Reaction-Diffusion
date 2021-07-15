use std::ops::{Add, Div, Mul, Range, Sub};

pub fn get_wrapping_index(x: isize, y: isize, width: usize, height: usize) -> usize {
    let (width, height) = (width as isize, height as isize);
    (((y + height) % height) * width + ((x + width) % width)) as usize
}

pub fn map_t_of_range_a_to_range_b<T>(t: T, range_a: Range<T>, range_b: Range<T>) -> T
where
    T: Copy + Sub<Output = T> + Div<Output = T> + Add<Output = T> + Mul<Output = T>,
{
    let slope = (range_b.end - range_b.start) / (range_a.end - range_a.start);
    range_b.start + slope * (t - range_a.start)
}

pub trait Interpolate {
    fn interpolate(&self, other: &Self, t: f32) -> Self;
}

// This can be used once specialization is stable https://github.com/rust-lang/rust/issues/31844
// It's been two years since I wrote that, lol
//impl<T: Mul<f32, Output = T> + Add<Output = T> + Copy> Interpolate for T {
//    fn interpolate(&self, other: &Self, t: f32) -> Self {
//        *self * (1.0 - t) + *other * t
//    }
//}

type Rgba = [u8; 4];

impl Interpolate for Rgba {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        let [r1, g1, b1, a1] = self;
        let [r2, g2, b2, a2] = other;

        [
            r1.interpolate(&r2, t),
            g1.interpolate(&g2, t),
            b1.interpolate(&b2, t),
            a1.interpolate(&a2, t),
        ]
    }
}

impl Interpolate for u8 {
    fn interpolate(&self, other: &Self, t: f32) -> u8 {
        if self == other {
            *self
        } else {
            (f32::from(*self) * (1.0 - t) + f32::from(*other) * t) as u8
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_get_index() {
        let test_grid_height = 4;
        let test_grid_width = 3;

        use super::get_wrapping_index;

        assert_eq!(
            0,
            get_wrapping_index(0, 0, test_grid_width, test_grid_height)
        );
        assert_eq!(
            0,
            get_wrapping_index(3, 0, test_grid_width, test_grid_height)
        );
        assert_eq!(
            0,
            get_wrapping_index(3, 4, test_grid_width, test_grid_height)
        );
        assert_eq!(
            6,
            get_wrapping_index(6, 2, test_grid_width, test_grid_height)
        );
        assert_eq!(
            7,
            get_wrapping_index(-2, -2, test_grid_width, test_grid_height)
        );
        assert_eq!(
            4,
            get_wrapping_index(-2456, 562, test_grid_width, test_grid_height)
        );
    }
}
