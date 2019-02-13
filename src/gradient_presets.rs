use crate::gradient::ColorGradient;

pub fn new_pink_and_blue() -> ColorGradient {
    let mut gradient = ColorGradient::from_colors([0, 0, 0], [255, 255, 255]);
    gradient.add_color_at_t(0.80, [0, 20, 230]);
    gradient.add_color_at_t(0.63, [200, 0, 255]);
    gradient.add_color_at_t(0.60, [255, 0, 0]);
    gradient.add_color_at_t(0.53, [0, 255, 255]);
    gradient.add_color_at_t(0.40, [0, 0, 0]);

    gradient
}
