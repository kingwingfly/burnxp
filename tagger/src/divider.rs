use crate::utils::{json_from, json_into, BitFlags, DataSetDesc, TagRecord};
use anyhow::Result;
use argmin::{
    core::{observers::ObserverMode, CostFunction, Executor, Gradient},
    solver::{gradientdescent::SteepestDescent, linesearch::HagerZhangLineSearch},
};
use argmin_observer_slog::SlogLogger;
use nalgebra::{DMatrix, DVector};
use rand::{distributions::Uniform, seq::SliceRandom, thread_rng, Rng};
use std::{collections::HashMap, path::PathBuf};

#[derive(Debug)]
pub struct Divider {
    to_divide: TagRecord<PathBuf>,
    ratio: (u32, u32),
    train_path: PathBuf,
    valid_path: PathBuf,
    max_iters: u64,
}

impl Divider {
    pub fn new(
        path: PathBuf,
        train: u32,
        valid: u32,
        train_path: PathBuf,
        valid_path: PathBuf,
        max_iters: u64,
    ) -> Result<Self> {
        assert!(train + valid > 0, "expected train + valid > 0");
        Ok(Self {
            to_divide: json_from(&path)?,
            ratio: (train, valid),
            train_path,
            valid_path,
            max_iters,
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
            let split =
                self.ratio.1 as usize * paths.len() / (self.ratio.0 + self.ratio.1) as usize;
            let train = paths.drain(split..).collect::<Vec<_>>();
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
        let max = map_t.values().map(Vec::len).max().unwrap_or_default() as f64;
        let problem = Problem::new(m.clone(), target, max);
        let init = DVector::from_iterator(
            flags.len(),
            rng.sample_iter(Uniform::new(0., max)).take(flags.len()),
        );
        let linesearch = HagerZhangLineSearch::new();
        let solver = SteepestDescent::new(linesearch);
        let res = Executor::new(problem, solver)
            .configure(|state| {
                state
                    .param(init)
                    .max_iters(self.max_iters)
                    .target_cost((4 * all_tags.len()) as f64)
            })
            .add_observer(SlogLogger::term(), ObserverMode::Always)
            .run()?;
        println!("{}", res);
        let ans = res
            .state
            .best_param
            .expect("Failed to solve")
            .into_iter()
            .map(|x| x.max(1.) as usize)
            .collect::<Vec<_>>();
        let ans_matrix = DVector::from_iterator(ans.len(), ans.iter().map(|&x| x as f64));
        let verify = &m * &ans_matrix;
        let weights = verify.len() as f64
            * (DVector::from_element(verify.len(), 1.).component_div(&verify)).normalize();
        println!("{:<5}{:<8}Tag", "Num", "Weights");
        for (i, tag) in all_tags.iter().enumerate() {
            println!("{:<5}{:<8.2}{}", verify[i], weights[i], tag);
        }

        train_set.up_sample = all_flags.into_iter().zip(ans).collect();
        train_set.weights = Some(weights.into_iter().map(|x| *x as f32).collect());
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
    max: f64,
}

impl Problem {
    fn new(m: DMatrix<f64>, target: DVector<f64>, max: f64) -> Self {
        Self { m, target, max }
    }
}

impl CostFunction for Problem {
    type Param = DVector<f64>;
    type Output = f64;

    fn cost(&self, param: &Self::Param) -> std::result::Result<Self::Output, anyhow::Error> {
        let delta = &self.m * param - &self.target;
        let cost = 0.5 * delta.transpose() * delta;
        Ok(cost[0]
            + param
                .iter()
                .map(|&x| {
                    if x < 1. {
                        2e2 * (x - 1.).powi(2)
                    } else if x > self.max {
                        1e1 * (x - self.max).powi(2)
                    } else {
                        0.
                    }
                })
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
                param.iter().map(|&x| {
                    if x < 1. {
                        4e2 * (x - 1.)
                    } else if x > self.max {
                        2e1 * (x - self.max)
                    } else {
                        0.
                    }
                }),
            ))
    }
}
