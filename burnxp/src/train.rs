use burn::{
    data::dataloader::DataLoaderBuilder,
    optim::AdamConfig,
    prelude::*,
    record::{CompactRecorder, Recorder},
    tensor::backend::AutodiffBackend,
    train::{
        metric::{
            store::{Aggregate, Direction, Split},
            CpuMemory, CpuTemperature, CpuUse, CudaMetric, LossMetric,
        },
        LearnerBuilder, MetricEarlyStoppingStrategy, StoppingCondition,
    },
};
use std::path::PathBuf;

use crate::{
    data::{ImageBatcher, ImageDataSet},
    model::ScoreModelConfig,
};

#[derive(Config)]
pub struct TrainingConfig {
    model: ScoreModelConfig,
    optimizer: AdamConfig,
    train_set: PathBuf,
    valid_set: PathBuf,
    pretrained: Option<PathBuf>,
    #[config(default = 128)]
    num_epochs: usize,
    #[config(default = 1)]
    batch_size: usize,
    #[config(default = 1)]
    num_workers: usize,
    #[config(default = 42)]
    seed: u64,
    #[config(default = 1.0e-4)]
    learning_rate: f64,
    #[config(default = 20)]
    early_stopping: usize,
}

fn create_artifact_dir(artifact_dir: &PathBuf) {
    // Remove existing artifacts before to get an accurate learner summary
    std::fs::remove_dir_all(artifact_dir).ok();
    std::fs::create_dir_all(artifact_dir).ok();
}

pub fn train<B: AutodiffBackend>(artifact_dir: PathBuf, config: TrainingConfig, device: B::Device) {
    create_artifact_dir(&artifact_dir);

    B::seed(config.seed);

    config
        .save(artifact_dir.join("train_config.json"))
        .expect("Config should be saved successfully");

    let batcher_train = ImageBatcher::<B>::new(device.clone());
    let batcher_valid = ImageBatcher::<B::InnerBackend>::new(device.clone());

    let dataloader_train = DataLoaderBuilder::new(batcher_train)
        .batch_size(config.batch_size)
        .shuffle(config.seed)
        .num_workers(config.num_workers)
        .build(ImageDataSet::train(config.train_set).expect("Training set failed to be loaded"));

    let dataloader_valid = DataLoaderBuilder::new(batcher_valid)
        .batch_size(config.batch_size)
        .shuffle(config.seed)
        .num_workers(config.num_workers)
        .build(ImageDataSet::valid(config.valid_set).expect("Validation set faild to be loaded"));

    let learner = LearnerBuilder::new(&artifact_dir)
        .metric_train_numeric(LossMetric::new())
        .metric_valid_numeric(LossMetric::new())
        .metric_train(CudaMetric::new())
        .metric_train(CpuUse::new())
        .metric_train(CpuTemperature::new())
        .metric_train(CpuMemory::new())
        .early_stopping(MetricEarlyStoppingStrategy::new::<LossMetric<B>>(
            Aggregate::Mean,
            Direction::Lowest,
            Split::Valid,
            StoppingCondition::NoImprovementSince { n_epochs: config.early_stopping },
        ))
        .with_file_checkpointer(CompactRecorder::new())
        .devices(vec![device.clone()])
        .num_epochs(config.num_epochs)
        .summary()
        .build(
            {
                let mut model = config.model.init::<B>(&device);
                if let Some(pretrain) = config.pretrained {
                    model = model.load_record(
                        CompactRecorder::new()
                            .load(pretrain, &device)
                            .expect("Please offer a valid pretrained model. If it's in the artifact directory, it is removed when recreating the directory."),
                    )
                }
                model
            },
            config.optimizer.init(),
            config.learning_rate,
        );

    let model_trained = learner.fit(dataloader_train, dataloader_valid);

    model_trained
        .save_file(artifact_dir.join("model"), &CompactRecorder::new())
        .expect("Trained model should be saved successfully");
}
