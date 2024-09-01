use burn::{
    data::dataloader::DataLoaderBuilder,
    optim::AdamConfig,
    prelude::*,
    record::CompactRecorder,
    tensor::backend::AutodiffBackend,
    train::{
        metric::{
            store::{Aggregate, Direction, Split},
            LossMetric,
        },
        LearnerBuilder, MetricEarlyStoppingStrategy, StoppingCondition,
    },
};

use crate::{
    data::{PicBatcher, PicDataSet},
    model::ScoreModelConfig,
};

#[derive(Config)]
pub struct TrainingConfig {
    model: ScoreModelConfig,
    optimizer: AdamConfig,
    #[config(default = 500)]
    num_epochs: usize,
    #[config(default = 32)]
    batch_size: usize,
    #[config(default = 4)]
    num_workers: usize,
    #[config(default = 42)]
    seed: u64,
    #[config(default = 1.0e-4)]
    learning_rate: f64,
}

fn create_artifact_dir(artifact_dir: &str) {
    // Remove existing artifacts before to get an accurate learner summary
    std::fs::remove_dir_all(artifact_dir).ok();
    std::fs::create_dir_all(artifact_dir).ok();
}

pub fn train<B: AutodiffBackend>(artifact_dir: &str, config: TrainingConfig, device: B::Device) {
    create_artifact_dir(artifact_dir);

    B::seed(config.seed);

    config
        .save(format!("{artifact_dir}/config.json"))
        .expect("Config should be saved successfully");

    let batcher_train = PicBatcher::<B>::new(device.clone());
    let batcher_valid = PicBatcher::<B::InnerBackend>::new(device.clone());

    let dataloader_train = DataLoaderBuilder::new(batcher_train)
        .batch_size(config.batch_size)
        .shuffle(config.seed)
        .num_workers(config.num_workers)
        .build(PicDataSet::train());

    let dataloader_test = DataLoaderBuilder::new(batcher_valid)
        .batch_size(config.batch_size)
        .shuffle(config.seed)
        .num_workers(config.num_workers)
        .build(PicDataSet::test());

    let learner = LearnerBuilder::new(artifact_dir)
        .metric_train_numeric(LossMetric::new())
        .metric_valid_numeric(LossMetric::new())
        .early_stopping(MetricEarlyStoppingStrategy::new::<LossMetric<B>>(
            Aggregate::Mean,
            Direction::Lowest,
            Split::Valid,
            StoppingCondition::NoImprovementSince { n_epochs: 50 },
        ))
        .with_file_checkpointer(CompactRecorder::new())
        .devices(vec![device.clone()])
        .num_epochs(config.num_epochs)
        .summary()
        .build(
            config.model.init::<B>(&device),
            config.optimizer.init(),
            config.learning_rate,
        );

    let model_trained = learner.fit(dataloader_train, dataloader_test);

    model_trained
        .save_file(format!("{artifact_dir}/model"), &CompactRecorder::new())
        .expect("Trained model should be saved successfully");
}
