use crate::gradient::ColorGradient;

pub fn new_pink_and_blue() -> ColorGradient {
    let mut gradient = ColorGradient::from_colors([0, 0, 0, 255], [255, 255, 255, 255]);
    gradient.add_color_at_t(0.80, [0, 20, 230, 255]);
    gradient.add_color_at_t(0.63, [200, 0, 255, 255]);
    gradient.add_color_at_t(0.60, [255, 0, 0, 255]);
    gradient.add_color_at_t(0.53, [0, 255, 255, 255]);
    gradient.add_color_at_t(0.40, [0, 0, 0, 255]);

    gradient
}

pub fn new_rainbow() -> ColorGradient {
    let mut gradient = ColorGradient::from_colors([0, 0, 0, 255], [255, 255, 255, 255]);

    gradient.add_color_at_t(0.45, [131, 58, 180, 255]);
    gradient.add_color_at_t(0.50, [243, 31, 42, 255]);
    gradient.add_color_at_t(0.55, [253, 252, 29, 255]);
    gradient.add_color_at_t(0.60, [29, 253, 49, 255]);
    gradient.add_color_at_t(0.85, [29, 220, 253, 255]);
    gradient.add_color_at_t(0.95, [30, 29, 253, 255]);

    gradient
}

pub fn new_protanopia_friendly() -> ColorGradient {
    let mut gradient = ColorGradient::from_colors([0, 0, 0, 255], [255, 255, 255, 255]);

    // Using colors that are distinguishable for people with protanopia
    gradient.add_color_at_t(0.20, [0, 0, 0, 255]); // Black
    gradient.add_color_at_t(0.40, [0, 0, 255, 255]); // Blue
    gradient.add_color_at_t(0.60, [255, 255, 0, 255]); // Yellow
    gradient.add_color_at_t(0.80, [255, 0, 255, 255]); // Magenta
    gradient.add_color_at_t(0.90, [255, 255, 255, 255]); // White

    gradient
}
