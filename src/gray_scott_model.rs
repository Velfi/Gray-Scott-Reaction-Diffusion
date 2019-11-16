use crate::{utils::clamp_f32, utils::get_wrapping_index};
use rayon::prelude::*;
use std::iter;

pub struct ReactionDiffusionSystem {
    pub width: usize,
    pub height: usize,
    coords_list: Vec<(usize, usize)>,
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
        let coords_list = (0..height)
            .flat_map(|y| (0..width).map(move |x| (x, y)))
            .collect();
        Self {
            width,
            height,
            coords_list,
            f,
            k,
            delta_u,
            delta_v,
            u: iter::repeat(1.0).take(vec_capacity).collect(),
            v: iter::repeat(0.0).take(vec_capacity).collect(),
        }
    }

    pub fn get(&self, cs: &ChemicalSpecies, x: isize, y: isize) -> f32 {
        let index = get_wrapping_index(x, y, self.width, self.height);
        let cs_vec = match cs {
            ChemicalSpecies::U => &self.u,
            ChemicalSpecies::V => &self.v,
        };

        *cs_vec
            .get(index)
            .unwrap_or_else(|| panic!(format!("Tried to get {:?}[{}] but failed.", cs, index)))
    }

    pub fn set(&mut self, cs: &ChemicalSpecies, x: isize, y: isize, v: f32) {
        let index = get_wrapping_index(x, y, self.width, self.height);
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
        let (new_u, new_v): (Vec<f32>, Vec<f32>) = self
            .coords_list
            .par_iter()
            .fold(
                || (Vec::new(), Vec::new()),
                |mut acc, (x, y)| {
                    let x = *x as isize;
                    let y = *y as isize;
                    let u = self.get(&ChemicalSpecies::U, x, y);
                    let v = self.get(&ChemicalSpecies::V, x, y);

                    let delta_u = self.delta_u * self.get_laplacian(ChemicalSpecies::U, x, y)
                        - (u * v * v)
                        + self.f * (1.0 - u);

                    let delta_v = self.delta_v * self.get_laplacian(ChemicalSpecies::V, x, y)
                        + (u * v * v)
                        - (self.k + self.f) * v;

                    let dt = delta_t * 0.05;

                    acc.1.push(clamp_f32(v + delta_v * dt, -1.0, 1.0));
                    acc.0.push(clamp_f32(u + delta_u * dt, -1.0, 1.0));
                    acc
                },
            )
            .reduce(
                || (Vec::new(), Vec::new()),
                |mut acc: (Vec<f32>, Vec<f32>), mut vecs: (Vec<f32>, Vec<f32>)| {
                    acc.0.append(&mut vecs.0);
                    acc.1.append(&mut vecs.1);
                    acc
                },
            );

        self.u = new_u;
        self.v = new_v;
    }
}
