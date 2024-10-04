use crate::utils::{json_from, json_into};
use anyhow::Result;
use rand::{seq::SliceRandom as _, thread_rng};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, Default, Serialize, Deserialize)]
struct Cache {
    tags: HashMap<String, i64>,
    tagged: HashMap<PathBuf, Vec<String>>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Output {
    pub num_classes: usize,
    pub tagged: Vec<(PathBuf, Vec<i8>)>,
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
        let (mut all_tags, mut weights) = (vec![], vec![]);
        let to_divide = &self.to_divide;
        for (tag, weight) in to_divide.tags.iter() {
            all_tags.push(tag.clone());
            weights.push(*weight);
        }
        all_tags.sort();
        let mut tagged = vec![];
        for (item, tags) in self.to_divide.tagged.iter() {
            let tags = all_tags.iter().map(|t| tags.contains(t) as i8).collect();
            tagged.push((item.clone(), tags));
        }
        let mut train_set = Output::default();
        let mut valid_set = Output::default();
        tagged.shuffle(&mut thread_rng());
        let len = tagged.len();
        let valid_set_size = (len as f64 * self.ratio).floor() as usize;
        let (valid_elems, train_elems) = tagged.split_at(valid_set_size);
        train_set.num_classes = weights.len();
        train_set.tagged = train_elems.to_vec();
        valid_set.num_classes = weights.len();
        valid_set.tagged = valid_elems.to_vec();
        json_into(&self.train_path, &train_set)?;
        json_into(&self.valid_path, &valid_set)?;
        Ok(())
    }
}
