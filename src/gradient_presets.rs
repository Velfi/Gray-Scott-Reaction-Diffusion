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

pub fn new_magma() -> ColorGradient {
    let mut gradient = ColorGradient::from_colors([0, 0, 0, 255], [255, 255, 255, 255]);

    // Magma color scheme from dark to light
    gradient.add_color_at_t(0.20, [0, 0, 4, 255]); // Dark blue-black
    gradient.add_color_at_t(0.40, [48, 0, 89, 255]); // Deep purple
    gradient.add_color_at_t(0.60, [189, 0, 38, 255]); // Deep red
    gradient.add_color_at_t(0.80, [251, 106, 74, 255]); // Orange-red
    gradient.add_color_at_t(0.90, [252, 197, 192, 255]); // Light pink
    gradient.add_color_at_t(0.95, [255, 255, 255, 255]); // White

    gradient
}

pub fn new_monochrome() -> ColorGradient {
    let mut gradient = ColorGradient::from_colors([0, 0, 0, 255], [255, 255, 255, 255]);

    // Simple black to white gradient
    gradient.add_color_at_t(0.20, [51, 51, 51, 255]); // Dark gray
    gradient.add_color_at_t(0.40, [102, 102, 102, 255]); // Medium gray
    gradient.add_color_at_t(0.60, [153, 153, 153, 255]); // Light gray
    gradient.add_color_at_t(0.80, [204, 204, 204, 255]); // Very light gray
    gradient.add_color_at_t(0.90, [255, 255, 255, 255]); // White

    gradient
}
