use std::iter;

const CLAMP_ERROR: &str = "min should be less than or equal to max, plz learn to clamp";

pub struct ReactionDiffusionSystem {
    pub width: usize,
    pub height: usize,
    f: f32,
    k: f32,
    delta_u: f32,
    delta_v: f32,
    u: Vec<f32>,
    v: Vec<f32>,
}

#[derive(Debug)]
pub enum ChemicalSpecies {
    U,
    V,
}

impl ReactionDiffusionSystem {
    pub fn new(width: usize, height: usize, f: f32, k: f32, delta_u: f32, delta_v: f32) -> Self {
        let vec_capacity = width * height;
        Self {
            width,
            height,
            f,
            k,
            delta_u,
            delta_v,
            u: iter::repeat(1.0).take(vec_capacity).collect(),
            v: iter::repeat(0.0).take(vec_capacity).collect(),
        }
    }

    pub fn get(&self, cs: &ChemicalSpecies, x: isize, y: isize) -> f32 {
        let index = get_index(x, y, self.width, self.height);
        let cs_vec = match cs {
            ChemicalSpecies::U => &self.u,
            ChemicalSpecies::V => &self.v,
        };

        *cs_vec
            .get(index)
            .unwrap_or_else(|| panic!(format!("Tried to get {:?}[{}] but failed.", cs, index)))
    }

    pub fn set(&mut self, cs: &ChemicalSpecies, x: isize, y: isize, v: f32) {
        let index = get_index(x, y, self.width, self.height);
        let cs_vec = match cs {
            ChemicalSpecies::U => &mut self.u,
            ChemicalSpecies::V => &mut self.v,
        };

        let index = index % cs_vec.len();

        cs_vec[index] = clamp_f32(v, -1.0, 1.0)
    }

    pub fn get_laplacian(&self, cs: ChemicalSpecies, x: isize, y: isize) -> f32 {
        0.05 * self.get(&cs, x - 1, y - 1)
            + 0.2 * self.get(&cs, x - 1, y)
            + 0.05 * self.get(&cs, x - 1, y + 1)
            + 0.2 * self.get(&cs, x, y - 1)
            + -1.0 * self.get(&cs, x, y)
            + 0.2 * self.get(&cs, x, y + 1)
            + 0.05 * self.get(&cs, x + 1, y - 1)
            + 0.2 * self.get(&cs, x + 1, y)
            + 0.05 * self.get(&cs, x + 1, y + 1)
    }

    pub fn update(&mut self, delta_t: f32) {
        let (new_u, new_v): (Vec<f32>, Vec<f32>) = (0..self.height)
            .flat_map(|y| (0..self.width).map(move |x| (x, y)))
            .fold((Vec::new(), Vec::new()), |mut acc, (x, y)| {
                let x = x as isize;
                let y = y as isize;
                let u = self.get(&ChemicalSpecies::U, x, y);
                let v = self.get(&ChemicalSpecies::V, x, y);

                let delta_u = self.delta_u * self.get_laplacian(ChemicalSpecies::U, x, y)
                    - (u * v * v)
                    + self.f * (1.0 - u);

                let delta_v = self.delta_v * self.get_laplacian(ChemicalSpecies::V, x, y)
                    + (u * v * v)
                    - (self.k + self.f) * v;

                acc.0.push(clamp_f32(u + delta_u * delta_t, -1.0, 1.0));
                acc.1.push(clamp_f32(v + delta_v * delta_t, -1.0, 1.0));
                acc
            });

        self.u = new_u;
        self.v = new_v;
    }
}

pub fn get_index(x: isize, y: isize, width: usize, height: usize) -> usize {
    (((y + height as isize) % height as isize) * width as isize
        + ((x + width as isize) % width as isize)) as usize
}

fn clamp_f32(n: f32, min: f32, max: f32) -> f32 {
    if min > max {
        panic!(CLAMP_ERROR)
    }

    (max).min((min).max(n))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_get_index() {
        let test_vec = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
        let test_grid_height = 4;
        let test_grid_width = 3;

        use super::get_index;

        assert_eq!(0, get_index(0, 0, test_grid_width, test_grid_height));
        assert_eq!(0, get_index(3, 0, test_grid_width, test_grid_height));
        assert_eq!(0, get_index(3, 4, test_grid_width, test_grid_height));
        assert_eq!(6, get_index(6, 2, test_grid_width, test_grid_height));
        assert_eq!(7, get_index(-2, -2, test_grid_width, test_grid_height));
        assert_eq!(4, get_index(-2456, 562, test_grid_width, test_grid_height));
    }

    #[test]
    fn test_clamp_f32() {
        use super::clamp_f32;

        assert_eq!(0.0, clamp_f32(-100.0, 0.0, 10.0));
        assert_eq!(10.0, clamp_f32(100.0, 0.0, 10.0));
        assert_eq!(8.56, clamp_f32(8.56, 0.0, 10.0));
    }

    #[test]
    #[should_panic(expected = "min should be less than or equal to max, plz learn to clamp")]
    fn test_clamp_f32_panic() {
        use super::clamp_f32;

        clamp_f32(5.0, 7.0, 2.0);
    }
}
