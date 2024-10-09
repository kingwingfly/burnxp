use crate::utils::{json_from, json_into};
use anyhow::Result;
use rand::{seq::SliceRandom as _, thread_rng};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, Default, Serialize, Deserialize)]
struct Cache {
    tags: HashMap<String, i64>,
    tagged: HashMap<PathBuf, Vec<String>>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Output {
    weights: Option<Vec<f32>>,
    tagged: Vec<(PathBuf, Vec<i8>)>,
}

#[derive(Debug)]
pub struct Divider {
    to_divide: Cache,
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
        let to_divide: Cache = json_from(&path)?;
        Ok(Self {
            to_divide,
            ratio: valid as f64 / (train + valid) as f64,
            train_path,
            valid_path,
        })
    }

    pub fn divide(&self) -> Result<()> {
        let mut all_tags = vec![];
        let to_divide = &self.to_divide;
        for tag in to_divide.tags.keys() {
            all_tags.push(tag);
        }
        all_tags.sort();
        let mut tagged = to_divide
            .tagged
            .par_iter()
            .map(|(item, tags)| {
                let mut res = vec![0; all_tags.len()];
                for (i, tag) in all_tags.iter().enumerate() {
                    res[i] = tags.contains(tag) as i8;
                }
                (item.clone(), res)
            })
            .collect::<Vec<_>>();
        let mut weights = tagged
            .iter()
            .fold(vec![0.; all_tags.len()], |mut acc, (_, tags)| {
                for (i, tag) in tags.iter().enumerate() {
                    acc[i] += *tag as f32;
                }
                acc
            })
            .into_iter()
            .map(|x| 1. / x.clamp(2., 256.).log2())
            .collect::<Vec<_>>();
        let factory = weights.iter().sum::<f32>() / weights.len() as f32;
        weights.iter_mut().for_each(|x| *x /= factory);
        let mut train_set = Output::default();
        let mut valid_set = Output::default();
        tagged.shuffle(&mut thread_rng());
        let len = tagged.len();
        let valid_set_size = (len as f64 * self.ratio).floor() as usize;
        let (valid_elems, train_elems) = tagged.split_at(valid_set_size);
        train_set.weights = Some(weights);
        train_set.tagged = train_elems.to_vec();
        valid_set.tagged = valid_elems.to_vec();
        json_into(&self.train_path, &train_set)?;
        json_into(&self.valid_path, &valid_set)?;
        Ok(())
    }
}
