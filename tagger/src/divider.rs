use crate::utils::{json_from, json_into, BitFlags, DataSetDesc, TagRecord};
use anyhow::Result;
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
        let mut train_set = DataSetDesc::new(all_tags.len());
        let mut valid_set = DataSetDesc::new(all_tags.len());
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
        let target = map_t.values().map(Vec::len).max().unwrap_or_default();
        let up_sample = map_t.keys().map(|k| (*k, target)).collect();
        train_set.up_sample = up_sample;
        train_set.binary_encodings = map_t;
        valid_set.binary_encodings = map_v;
        json_into(&self.train_path, &train_set)?;
        json_into(&self.valid_path, &valid_set)?;
        Ok(())
    }
}
