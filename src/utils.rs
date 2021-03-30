pub fn get_wrapping_index(x: isize, y: isize, width: usize, height: usize) -> usize {
    (((y + height as isize) % height as isize) * width as isize
        + ((x + width as isize) % width as isize)) as usize
}

pub fn map_t_of_range_a_to_range_b(
    t: f32,
    range_a_start: f32,
    range_a_end: f32,
    range_b_start: f32,
    range_b_end: f32,
) -> f32 {
    let slope = (range_b_end - range_b_start) / (range_a_end - range_a_start);
    range_b_start + slope * (t - range_a_start)
}

pub trait Interpolate {
    fn interpolate(&self, other: &Self, t: f32) -> Self;
}

// This can be used once specialization is stable
//impl<T: Mul<f32, Output = T> + Add<Output = T> + Copy> Interpolate for T {
//    fn interpolate(&self, other: &Self, t: f32) -> Self {
//        *self * (1.0 - t) + *other * t
//    }
//}

impl Interpolate for u8 {
    fn interpolate(&self, other: &Self, t: f32) -> u8 {
        (f32::from(*self) * (1.0 - t) + f32::from(*other) * t) as u8
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_get_index() {
        let test_vec = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
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
