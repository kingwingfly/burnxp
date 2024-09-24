use crate::{
    data::{ImageBatcher, ImageDataSet},
    RnnType, ScoreModelConfig,
};
use burn::{
    config::Config,
    data::dataloader::DataLoaderBuilder,
    prelude::*,
    record::{CompactRecorder, Recorder},
};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, Clone, Default, ValueEnum, Serialize, Deserialize)]
pub enum Output {
    #[default]
    Tui,
    Tty,
    Json,
}

#[derive(Config, Debug)]
pub struct PredictConfig {
    model: RnnType,
    checkpoint: PathBuf,
    input: PathBuf,
    output: Output,
    #[config(default = 32)]
    batch_size: usize,
    #[config(default = 8)]
    num_workers: usize,
}

pub fn predict<B: Backend>(config: PredictConfig, device: B::Device) {
    let model = ScoreModelConfig::new(config.model)
        .init::<B>(&device)
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
                let scores = model
                    .forward(batch.datas)
                    .into_data()
                    .to_vec::<f32>()
                    .unwrap();
                for (path, score) in batch.paths.into_iter().zip(scores) {
                    println!("{}\t'{}'", score, path.to_string_lossy(),);
                }
            }
        }
        Output::Json => {
            let mut output = HashMap::new();
            for batch in dataloader_predict.iter() {
                let scores = model
                    .forward(batch.datas)
                    .into_data()
                    .to_vec::<f32>()
                    .unwrap();
                output.extend(batch.paths.into_iter().zip(scores));
            }
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
    }
}
