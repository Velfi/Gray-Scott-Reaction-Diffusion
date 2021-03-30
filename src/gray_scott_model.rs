use crate::utils::get_wrapping_index;
use rayon::prelude::*;
use std::iter;

pub struct ReactionDiffusionSystem {
    pub width: u32,
    pub height: u32,
    coords_list: Vec<(u32, u32)>,
    feed_rate: f32,
    kill_rate: f32,
    delta_u: f32,
    delta_v: f32,
    u: Vec<f32>,
    v: Vec<f32>,
}

#[derive(Debug, Clone, Copy)]
pub enum ChemicalSpecies {
    U,
    V,
}

impl ReactionDiffusionSystem {
    pub fn new(
        width: u32,
        height: u32,
        feed_rate: f32,
        kill_rate: f32,
        delta_u: f32,
        delta_v: f32,
    ) -> Self {
        let vec_capacity = (width * height) as usize;
        let coords_list = (0..height)
            .flat_map(|y| (0..width).map(move |x| (x, y)))
            .collect();
        Self {
            width,
            height,
            coords_list,
            feed_rate,
            kill_rate,
            delta_u,
            delta_v,
            u: iter::repeat(1.0).take(vec_capacity).collect(),
            v: iter::repeat(0.0).take(vec_capacity).collect(),
        }
    }

    pub fn get(&self, cs: ChemicalSpecies, x: isize, y: isize) -> f32 {
        let index = get_wrapping_index(x, y, self.width, self.height);
        self.get_by_index(cs, index)
    }

    pub fn get_by_index(&self, cs: ChemicalSpecies, index: usize) -> f32 {
        assert!(
            (0..self.len()).contains(&index),
            "index {} is not in range 0..{}!",
            index,
            self.u.len()
        );

        let cs_vec = match cs {
            ChemicalSpecies::U => &self.u,
            ChemicalSpecies::V => &self.v,
        };

        *cs_vec
            .get(index)
            .unwrap_or_else(|| panic!("Tried to get {:?}[{}] but failed.", cs, index))
    }

    pub fn len(&self) -> usize {
        (self.height * self.width) as usize
    }

    pub fn set(&mut self, cs: ChemicalSpecies, x: isize, y: isize, v: f32) {
        let index = get_wrapping_index(x, y, self.width, self.height);
        let cs_vec = match cs {
            ChemicalSpecies::U => &mut self.u,
            ChemicalSpecies::V => &mut self.v,
        };

        let index = index % cs_vec.len();

        cs_vec[index] = v.clamp(-1.0, 1.0)
    }

    pub fn get_laplacian(&self, cs: ChemicalSpecies, x: isize, y: isize) -> f32 {
        0.05 * self.get(cs, x - 1, y - 1)
            + 0.2 * self.get(cs, x - 1, y)
            + 0.05 * self.get(cs, x - 1, y + 1)
            + 0.2 * self.get(cs, x, y - 1)
            + -1.0 * self.get(cs, x, y)
            + 0.2 * self.get(cs, x, y + 1)
            + 0.05 * self.get(cs, x + 1, y - 1)
            + 0.2 * self.get(cs, x + 1, y)
            + 0.05 * self.get(cs, x + 1, y + 1)
    }

    pub fn update(&mut self, delta_t: f32) {
        // delta_t is in seconds, so we convert to milliseconds (which the diffusion calculation expects)
        let delta_t = delta_t * 100.0;

        // The reaction goes nuts if delta_t is greater than 1, so if we need to go fast then
        // it has to be calculated multiple times in steps
        // Disabled for now
        // while delta_t > 1.0 {
        //     delta_t -= 1.0;
        //     self.reaction(1.0)
        // }

        self.reaction(delta_t)
    }

    fn reaction(&mut self, delta_t: f32) {
        // assert!(
        //     delta_t <= 1.0,
        //     "delta_t must never exceed 1.0 or the reaction gets borked"
        // );

        let (new_u, new_v): (Vec<f32>, Vec<f32>) = self
            .coords_list
            .par_iter()
            .fold(
                || (Vec::new(), Vec::new()),
                |mut acc, (x, y)| {
                    let x = *x as isize;
                    let y = *y as isize;
                    let u = self.get(ChemicalSpecies::U, x, y);
                    let v = self.get(ChemicalSpecies::V, x, y);
                    let reaction_rate = u * v.powi(2);

                    let delta_u = self.delta_u * self.get_laplacian(ChemicalSpecies::U, x, y)
                        - reaction_rate
                        + self.feed_rate * (1.0 - u);

                    let delta_v = self.delta_v * self.get_laplacian(ChemicalSpecies::V, x, y)
                        + reaction_rate
                        - (self.kill_rate + self.feed_rate) * v;

                    acc.1.push((v + delta_v * delta_t).clamp(0.0, 1.0));
                    acc.0.push((u + delta_u * delta_t).clamp(0.0, 1.0));
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
