use crate::utils::{json_from, json_into};
use anyhow::Result;
use rand::{seq::SliceRandom as _, thread_rng};
use std::{collections::HashMap, path::PathBuf};

type Set = HashMap<Score, Vec<PathBuf>>;
type Score = i64;

#[derive(Debug)]
pub struct Divider {
    to_divide: Set,
    ratio: f64,
    train_path: PathBuf,
    valid_path: PathBuf,
}

impl Divider {
    pub fn new(
        path: PathBuf,
        train: usize,
        valid: usize,
        train_path: PathBuf,
        valid_path: PathBuf,
    ) -> Result<Self> {
        assert!(train + valid > 0, "expected train + valid > 0");
        let to_divide: Set = json_from(&path)?;
        Ok(Self {
            to_divide,
            ratio: valid as f64 / (train + valid) as f64,
            train_path,
            valid_path,
        })
    }

    pub fn divide(self) -> Result<()> {
        let mut rng = thread_rng();
        let to_divide = self.to_divide;
        let mut train_set = HashMap::new();
        let mut valid_set = HashMap::new();
        for (score, mut elems) in to_divide.into_iter() {
            let len = elems.len();
            let choose = (len as f64 * self.ratio).floor() as usize;
            elems.shuffle(&mut rng);
            let (valid_elems, train_elems) = elems.split_at(choose);
            train_set.insert(score, train_elems.to_vec());
            valid_set.insert(score, valid_elems.to_vec());
        }
        json_into(&self.train_path, &train_set)?;
        json_into(&self.valid_path, &valid_set)?;
        Ok(())
    }
}
