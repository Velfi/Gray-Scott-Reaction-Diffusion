use crate::utils::clamp_f32;
use crate::utils::map_t_of_range_a_to_range_b;
use crate::utils::Interpolate;
use itertools::Itertools;

type Rgb = [u8; 3];
type TColor = (f32, Rgb);

struct Spectrum(Vec<TColor>);

impl Spectrum {
    pub fn from_colors(color_a: Rgb, color_b: Rgb) -> Self {
        let mut spectrum = Vec::new();
        spectrum.push((0f32, color_a));
        spectrum.push((1f32, color_b));

        Self(spectrum)
    }

    pub fn add_color_at_t(&mut self, t: f32, color: Rgb) {
        let t = clamp_f32(t, 0.0, 1.0);
        self.0.push((t, color));
        self.0.sort_by(|a, b| {
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

    fn get_bounding_colors_for_t(&self, t: f32) -> (TColor, TColor) {
        if let Some((position, upper_color)) =
            self.0.iter().find_position(|(color_t, _)| *color_t >= t)
        {
            if position >= 1 {
                return (self.0[position - 1], *upper_color);
            }
        };

        ((0.0, [0, 0, 0]), (1.0, [255, 255, 255]))
    }
}

struct GradientLut([Rgb; 256]);
impl GradientLut {
    pub fn color_at_t(&self, t: u8) -> Rgb {
        self.0[t as usize].clone()
    }
}

impl From<&Spectrum> for GradientLut {
    fn from(spectrum: &Spectrum) -> Self {
        let mut arr = [[0u8, 0u8, 0u8]; 256];

        for i in 0..=255 {
            arr[i] = spectrum.color_at_t(i as f32 / 255.0);
        }

        GradientLut(arr)
    }
}
pub struct ColorGradient {
    spectrum: Spectrum,
    lut: GradientLut,
}

impl ColorGradient {
    pub fn from_colors(color_a: Rgb, color_b: Rgb) -> Self {
        let spectrum = Spectrum::from_colors(color_a, color_b);

        Self {
            lut: (&spectrum).into(),
            spectrum,
        }
    }

    pub fn add_color_at_t(&mut self, t: f32, color: Rgb) {
        self.spectrum.add_color_at_t(t, color);
        self.rebuild_lut();
    }

    pub fn color_at_t(&self, t: f32) -> Rgb {
        if t > 1.0 || t < 0.0 {
            panic!("color_at_t only takes values in range of 0.0 - 1.0")
        }

        self.lut.color_at_t((t * 255.0) as u8)
    }

    pub fn color_at_t_u8(&self, t: u8) -> Rgb {
        self.lut.color_at_t(t)
    }

    fn rebuild_lut(&mut self) {
        self.lut = (&self.spectrum).into();
    }
}
