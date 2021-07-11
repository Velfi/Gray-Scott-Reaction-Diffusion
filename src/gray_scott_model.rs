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
    uvs: Vec<(f32, f32)>,
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
            uvs: iter::repeat((1.0, 0.0)).take(vec_capacity).collect(),
        }
    }

    pub fn uvs(&self) -> &[(f32, f32)] {
        &self.uvs
    }

    pub fn get(&self, x: isize, y: isize) -> (f32, f32) {
        let index = get_wrapping_index(x, y, self.width as usize, self.height as usize);
        self.get_by_index(index)
    }

    pub fn get_by_index(&self, index: usize) -> (f32, f32) {
        assert!(
            (0..self.len()).contains(&index),
            "index {} is not in range 0..{}!",
            index,
            self.len()
        );

        *self.uvs.get(index).expect("couldn't get uv by index")
    }

    pub fn len(&self) -> usize {
        (self.height * self.width) as usize
    }

    pub fn set(&mut self, x: isize, y: isize, v: (f32, f32)) {
        let index = get_wrapping_index(x, y, self.width as usize, self.height as usize);
        let index = index % self.len();
        let v = (v.0.clamp(-1.0, 1.0), v.1.clamp(-1.0, 1.0));

        self.uvs[index] = v;
    }

    pub fn get_laplacian(&self, x: isize, y: isize) -> (f32, f32) {
        (
            (0.05 * self.get(x - 1, y - 1).0
                + 0.2 * self.get(x - 1, y).0
                + 0.05 * self.get(x - 1, y + 1).0
                + 0.2 * self.get(x, y - 1).0
                + -1.0 * self.get(x, y).0
                + 0.2 * self.get(x, y + 1).0
                + 0.05 * self.get(x + 1, y - 1).0
                + 0.2 * self.get(x + 1, y).0
                + 0.05 * self.get(x + 1, y + 1).0),
            (0.05 * self.get(x - 1, y - 1).1
                + 0.2 * self.get(x - 1, y).1
                + 0.05 * self.get(x - 1, y + 1).1
                + 0.2 * self.get(x, y - 1).1
                + -1.0 * self.get(x, y).1
                + 0.2 * self.get(x, y + 1).1
                + 0.05 * self.get(x + 1, y - 1).1
                + 0.2 * self.get(x + 1, y).1
                + 0.05 * self.get(x + 1, y + 1).1),
        )
    }

    pub fn update(&mut self, mut _delta_t: f32) {
        // delta_t *= 100.0;
        // trace!("updating reaction with total delta_t of {:.02?}", delta_t);
        // The reaction goes nuts if delta_t is greater than 1, so if we need to go fast then
        // it has to be calculated multiple times in steps
        // while delta_t > 1.0 {
        //     delta_t -= 1.0;
        //     trace!(
        //         "running reaction with fractional delta_t of {:.02?}",
        //         delta_t
        //     );
        //     self.reaction(1.0)
        // }

        self.reaction(1.0)
    }

    fn reaction(&mut self, delta_t: f32) {
        let uvs_length = self.uvs.len();
        let new_uvs = self
            .coords_list
            .par_iter()
            .fold(
                || Vec::new(),
                |mut acc, (x, y)| {
                    let x = *x as isize;
                    let y = *y as isize;
                    let (u, v) = self.get(x, y);
                    let reaction_rate = u * v.powi(2);
                    let (laplacian_u, laplacian_v) = self.get_laplacian(x, y);

                    let delta_u =
                        self.delta_u * laplacian_u - reaction_rate + self.feed_rate * (1.0 - u);

                    let delta_v = self.delta_v * laplacian_v + reaction_rate
                        - (self.kill_rate + self.feed_rate) * v;

                    acc.push((
                        (u + delta_u * delta_t).clamp(0.0, 1.0),
                        (v + delta_v * delta_t).clamp(0.0, 1.0),
                    ));
                    acc
                },
            )
            .reduce(
                || Vec::with_capacity(uvs_length),
                |mut acc: Vec<(f32, f32)>, mut uvs: Vec<(f32, f32)>| {
                    acc.append(&mut uvs);
                    acc
                },
            );

        self.uvs = new_uvs;
    }
}
