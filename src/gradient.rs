use crate::utils::clamp_f32;
use crate::utils::map_t_of_range_a_to_range_b;
use crate::utils::Interpolate;
use itertools::Itertools;

type Rgb = [u8; 3];
type TColor = (f32, Rgb);

pub struct ColorGradient {
    spectrum: Vec<TColor>,
}

impl ColorGradient {
    pub fn from_colors(color_a: Rgb, color_b: Rgb) -> Self {
        let mut spectrum = Vec::new();
        spectrum.push((0f32, color_a));
        spectrum.push((1f32, color_b));

        ColorGradient { spectrum }
    }

    pub fn add_color_at_t(&mut self, t: f32, color: Rgb) {
        let t = clamp_f32(t, 0.0, 1.0);
        self.spectrum.push((t, color));
        self.spectrum.sort_by(|a, b| {
            a.0.partial_cmp(&b.0)
                .expect("Can't compare A and B because floats are tough")
        });
    }

    pub fn color_at_t(&self, t: f32) -> Rgb {
        let t = clamp_f32(t, 0.0, 1.0);
        let (before_color, after_color) = self.get_bounding_colors_for_t(t);
        let mapped_t = map_t_of_range_a_to_range_b(t, before_color.0, after_color.0, 0.0, 1.0);

        let [r1, g1, b1] = before_color.1;
        let [r2, g2, b2] = after_color.1;

        [
            r1.interpolate(&r2, mapped_t),
            g1.interpolate(&g2, mapped_t),
            b1.interpolate(&b2, mapped_t),
        ]
    }

    pub fn get_bounding_colors_for_t(&self, t: f32) -> (TColor, TColor) {
        if let Some((position, upper_color)) =
            self.spectrum.iter().find_position(|(color_t, _)| *color_t >= t)
        {
            if position >= 1 {
                let p_minus_one = position - 1;
                return (self.spectrum[p_minus_one], *upper_color);
            }
        };

        ((0.0, [0, 0, 0]), (1.0, [255, 255, 255]))
    }
}
