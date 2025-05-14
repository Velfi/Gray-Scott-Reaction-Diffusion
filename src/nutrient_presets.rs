#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NutrientPattern {
    Uniform = 0,
    Checkerboard = 1,
    DiagonalGradient = 2,
    RadialGradient = 3,
    VerticalStripes = 4,
    HorizontalStripes = 5,
    Noise = 6,
    WaveFunction = 7,
    CosineGrid = 8,
}

impl NutrientPattern {
    pub fn as_u32(self) -> u32 {
        self as u32
    }

    pub fn name(&self) -> &'static str {
        match self {
            NutrientPattern::Uniform => "Uniform",
            NutrientPattern::Checkerboard => "Checkerboard",
            NutrientPattern::DiagonalGradient => "Diagonal Gradient",
            NutrientPattern::RadialGradient => "Radial Gradient",
            NutrientPattern::VerticalStripes => "Vertical Stripes",
            NutrientPattern::HorizontalStripes => "Horizontal Stripes",
            NutrientPattern::Noise => "Noise",
            NutrientPattern::WaveFunction => "Wave Function",
            NutrientPattern::CosineGrid => "Cosine Grid",
        }
    }

    pub fn all() -> Vec<NutrientPattern> {
        use NutrientPattern::*;
        vec![
            Uniform,
            Checkerboard,
            DiagonalGradient,
            RadialGradient,
            VerticalStripes,
            HorizontalStripes,
            Noise,
            WaveFunction,
            CosineGrid,
        ]
    }
}

pub fn uniform() -> NutrientPattern {
    NutrientPattern::Uniform
}

pub fn checkerboard() -> NutrientPattern {
    NutrientPattern::Checkerboard
}

pub fn diagonal_gradient() -> NutrientPattern {
    NutrientPattern::DiagonalGradient
}

pub fn radial_gradient() -> NutrientPattern {
    NutrientPattern::RadialGradient
}

pub fn vertical_stripes() -> NutrientPattern {
    NutrientPattern::VerticalStripes
}

pub fn horizontal_stripes() -> NutrientPattern {
    NutrientPattern::HorizontalStripes
}

pub fn noise() -> NutrientPattern {
    NutrientPattern::Noise
}

pub fn wave_function() -> NutrientPattern {
    NutrientPattern::WaveFunction
}
