use rand::SeedableRng;

pub type NutrientPattern = Box<dyn Fn(usize, usize, usize, usize) -> f32 + Send + Sync>;

pub fn uniform() -> NutrientPattern {
    Box::new(|_x, _y, _width, _height| 1.0)
}

pub fn checkerboard() -> NutrientPattern {
    Box::new(
        |x, y, _width, _height| {
            if (x / 20 + y / 20) % 2 == 0 { 1.0 } else { 0.5 }
        },
    )
}

pub fn diagonal_gradient() -> NutrientPattern {
    Box::new(|x, y, width, height| {
        let normalized_x = x as f32 / width as f32;
        let normalized_y = y as f32 / height as f32;
        (normalized_x + normalized_y) / 2.0
    })
}

pub fn radial_gradient() -> NutrientPattern {
    Box::new(|x, y, width, height| {
        let center_x = width as f32 / 2.0;
        let center_y = height as f32 / 2.0;
        let dx = x as f32 - center_x;
        let dy = y as f32 - center_y;
        let distance = (dx * dx + dy * dy).sqrt();
        let max_distance = (center_x * center_x + center_y * center_y).sqrt();
        1.0 - (distance / max_distance)
    })
}

pub fn vertical_stripes() -> NutrientPattern {
    Box::new(|x, _y, width, _height| {
        let stripe_width = width / 10;
        if (x / stripe_width) % 2 == 0 {
            1.0
        } else {
            0.5
        }
    })
}

pub fn horizontal_stripes() -> NutrientPattern {
    Box::new(|_x, y, _width, height| {
        let stripe_height = height / 10;
        if (y / stripe_height) % 2 == 0 {
            1.0
        } else {
            0.5
        }
    })
}

pub fn noise() -> NutrientPattern {
    Box::new(|x, y, _width, _height| {
        let seed = (x * 73856093) ^ (y * 19349663);
        let mut rng = rand::rngs::SmallRng::seed_from_u64(seed as u64);
        rand::Rng::gen_range(&mut rng, 0.5..1.0)
    })
}
