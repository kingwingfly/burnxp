use crate::{
    data::{ImageBatcher, ImageDataSet},
    ModelConfig, ResNetType,
};
use burn::{
    config::Config,
    data::dataloader::DataLoaderBuilder,
    prelude::*,
    record::{CompactRecorder, Recorder},
};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt, fs::File, path::PathBuf};

#[derive(Debug, Clone, Default, ValueEnum, Serialize, Deserialize)]
pub enum Output {
    #[default]
    Tui,
    Tty,
    Json,
}

#[derive(Config, Debug)]
pub struct PredictConfig {
    model: ResNetType,
    checkpoint: PathBuf,
    input: PathBuf,
    output: Output,
    tags: PathBuf,
    #[config(default = 32)]
    batch_size: usize,
    #[config(default = 8)]
    num_workers: usize,
    #[config(default = 0.5)]
    confidence_threshold: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Tags {
    tags: HashMap<String, i64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Tag {
    name: String,
    weight: i64,
    possibility: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct ScoreResult {
    score: f32,
    tags: Vec<Tag>,
}

type JsonOutput = HashMap<PathBuf, ScoreResult>;

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\t{}\t{}", self.name, self.weight, self.possibility)
    }
}

pub fn predict<B: Backend>(config: PredictConfig, device: B::Device) {
    let all_tags: Tags = serde_json::from_reader(
        File::open(config.tags).expect("The file containing tags and weights should be accessible"),
    )
    .expect("The file containing tags and weights should be valid");
    let mut all_tags = all_tags.tags.into_iter().collect::<Vec<_>>();
    all_tags.sort_by_key(|(k, _)| k.clone());
    let model = ModelConfig::new(config.model)
        .init::<B>(&device, all_tags.len())
        .load_record(
            CompactRecorder::new()
                .load(config.checkpoint, &device)
                .expect("Failed to load checkpoint"),
        );
    let batcher_predict = ImageBatcher::<B>::new(device.clone());
    let dataloader_predict = DataLoaderBuilder::new(batcher_predict)
        .batch_size(config.batch_size)
        .num_workers(config.num_workers)
        .build(ImageDataSet::predict(config.input).expect("Training set failed to be loaded"));

    match config.output {
        Output::Tui => {
            todo!()
        }
        Output::Tty => {
            for batch in dataloader_predict.iter() {
                let tags = model
                    .forward(batch.datas)
                    .into_data()
                    .to_vec::<f32>()
                    .unwrap();
                for (path, tags) in batch.paths.into_iter().zip(tags.chunks(all_tags.len())) {
                    let tags = tags
                        .iter()
                        .zip(all_tags.iter())
                        .filter(|(p, _)| **p > 1. - config.confidence_threshold)
                        .map(|(p, (k, v))| Tag {
                            name: k.clone(),
                            weight: *v,
                            possibility: *p,
                        })
                        .collect::<Vec<_>>();
                    let total_score = tags
                        .iter()
                        .map(|t| t.possibility * t.weight as f32)
                        .sum::<f32>();
                    println!("{}: {}", path.display(), total_score);
                    eprintln!(
                        "{}",
                        tags.iter()
                            .map(|t| t.to_string())
                            .collect::<Vec<_>>()
                            .join("\n")
                    );
                }
            }
        }
        Output::Json => {
            let mut output: JsonOutput = HashMap::new();
            for batch in dataloader_predict.iter() {
                let tags = model
                    .forward(batch.datas)
                    .into_data()
                    .to_vec::<f32>()
                    .unwrap();
                for (path, tags) in batch.paths.into_iter().zip(tags.chunks(all_tags.len())) {
                    let tags = tags
                        .iter()
                        .zip(all_tags.iter())
                        .filter(|(p, _)| **p > 1. - config.confidence_threshold)
                        .map(|(p, (k, v))| Tag {
                            name: k.clone(),
                            weight: *v,
                            possibility: *p,
                        })
                        .collect::<Vec<_>>();
                    let total_score = tags.iter().map(|t| t.weight as f32).sum::<f32>();
                    output.insert(
                        path,
                        ScoreResult {
                            score: total_score,
                            tags,
                        },
                    );
                }
            }
            serde_json::to_writer_pretty(std::io::stdout(), &output).unwrap();
        }
    }
}
