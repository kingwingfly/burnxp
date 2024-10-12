use crate::utils::{json_from, json_into, BitFlags, DataSetDesc, TagRecord};
use anyhow::Result;
use argmin::{
    core::{CostFunction, Executor, Gradient},
    solver::{gradientdescent::SteepestDescent, linesearch::MoreThuenteLineSearch},
};
use nalgebra::{DMatrix, DVector};
use rand::{seq::SliceRandom, thread_rng};
use std::{collections::HashMap, path::PathBuf};

#[derive(Debug)]
pub struct Divider {
    to_divide: TagRecord<PathBuf>,
    ratio: (u32, u32),
    train_path: PathBuf,
    valid_path: PathBuf,
}

impl Divider {
    pub fn new(
        path: PathBuf,
        train: u32,
        valid: u32,
        train_path: PathBuf,
        valid_path: PathBuf,
    ) -> Result<Self> {
        assert!(train + valid > 0, "expected train + valid > 0");
        Ok(Self {
            to_divide: json_from(&path)?,
            ratio: (train, valid),
            train_path,
            valid_path,
        })
    }

    pub fn divide(self) -> Result<()> {
        let mut all_tags = self.to_divide.tags.keys().collect::<Vec<_>>();
        all_tags.sort();
        let num_classes = all_tags.len();
        let mut train_set = DataSetDesc::new(num_classes);
        let mut valid_set = DataSetDesc::new(num_classes);
        let mut rng = thread_rng();
        let mut map_v: HashMap<BitFlags, Vec<PathBuf>> = HashMap::new();
        for (path, tags) in self.to_divide.tagged {
            let mut flags = BitFlags::default();
            for (i, tag) in all_tags.iter().enumerate() {
                if tags.contains(tag) {
                    flags.enable(i as u64);
                }
            }
            map_v.entry(flags).or_default().push(path);
        }
        let mut map_t = HashMap::new();
        for (flags, paths) in map_v.iter_mut() {
            paths.shuffle(&mut rng);
            let ratio =
                self.ratio.1 as usize * paths.len() / (self.ratio.0 + self.ratio.1) as usize;
            let train = paths.drain(ratio..).collect::<Vec<_>>();
            map_t.insert(*flags, train);
        }
        let all_flags = map_t.keys().cloned().collect::<Vec<_>>();
        let flags = all_flags
            .iter()
            .map(|k| {
                let mut v = Vec::<f64>::from(*k);
                v.drain(num_classes..);
                DVector::<f64>::from_vec(v)
            })
            .collect::<Vec<_>>();
        let m: DMatrix<f64> = DMatrix::from_columns(&flags);
        let target = m.column_sum().max() * DVector::from_element(num_classes, 1.);

        let problem = Problem::new(m, target);
        let init = DVector::from_element(flags.len(), 0.);
        let linesearch = MoreThuenteLineSearch::new();
        let solver = SteepestDescent::new(linesearch);
        let res = Executor::new(problem, solver)
            .configure(|state| {
                state
                    .param(init)
                    .target_cost(128. * flags.len() as f64)
                    .max_iters(1000)
            })
            .run()?;

        train_set.up_sample = all_flags
            .into_iter()
            .zip(
                res.state
                    .best_param
                    .expect("Failed to solve")
                    .into_iter()
                    .map(|x| if x > &0. { *x as usize } else { 0 }),
            )
            .collect();
        train_set.binary_encodings = map_t;
        valid_set.binary_encodings = map_v;
        json_into(&self.train_path, &train_set)?;
        json_into(&self.valid_path, &valid_set)?;
        Ok(())
    }
}

struct Problem {
    m: DMatrix<f64>,
    target: DVector<f64>,
}

impl Problem {
    fn new(m: DMatrix<f64>, target: DVector<f64>) -> Self {
        Self { m, target }
    }
}

impl CostFunction for Problem {
    type Param = DVector<f64>;
    type Output = f64;

    fn cost(&self, param: &Self::Param) -> std::result::Result<Self::Output, anyhow::Error> {
        let cost = &self.m * param - &self.target;
        let cost = 0.5 * cost.transpose() * cost;
        Ok(cost[0]
            + param
                .iter()
                .map(|x| if x < &0. { 0.5 * x.powi(2) } else { 0. })
                .sum::<f64>())
    }
}

impl Gradient for Problem {
    type Param = DVector<f64>;
    type Gradient = DVector<f64>;

    fn gradient(&self, param: &Self::Param) -> std::result::Result<Self::Gradient, anyhow::Error> {
        Ok(&self.m.transpose() * (&self.m * param - &self.target)
            + DVector::from_iterator(
                param.len(),
                param.iter().map(|x| if x < &0. { *x } else { 0. }),
            ))
    }
}
