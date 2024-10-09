use burn::{
    data::dataloader::DataLoaderBuilder,
    lr_scheduler::linear::LinearLrSchedulerConfig,
    optim::AdamConfig,
    prelude::*,
    record::{CompactRecorder, Recorder},
    tensor::backend::AutodiffBackend,
    train::{
        metric::{
            store::{Aggregate, Direction, Split},
            CpuMemory, CpuUse, CudaMetric, HammingScore, LearningRateMetric, LossMetric,
        },
        LearnerBuilder, MetricEarlyStoppingStrategy, StoppingCondition,
    },
};
use serde::{Deserialize, Serialize};
use std::{fs::File, path::PathBuf};

use crate::{
    data::{ImageBatcher, ImageDataSet},
    model::ModelConfig,
};

#[derive(Config)]
pub struct TrainingConfig {
    model: ModelConfig,
    optimizer: AdamConfig,
    train_set: PathBuf,
    valid_set: PathBuf,
    pretrained: Option<PathBuf>,
    #[config(default = 64)]
    num_epochs: usize,
    #[config(default = 1)]
    batch_size: usize,
    #[config(default = 1)]
    num_workers: usize,
    #[config(default = 42)]
    seed: u64,
    #[config(default = 1.0e-3)]
    learning_rate: f64,
    #[config(default = 10)]
    early_stopping: usize,
    #[config(default = "0.5")]
    confidence_threshold: f32,
}

/// Should be the same as tagger/divider's Output
#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct Input {
    weights: Option<Vec<f32>>,
    tagged: Vec<(PathBuf, Vec<i8>)>,
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

    let train_input: Input = serde_json::from_reader(
        File::open(config.train_set).expect("Train set file should be accessible"),
    )
    .expect("Train set file should be illegal");
    let valid_input: Input = serde_json::from_reader(
        File::open(config.valid_set).expect("Validation set file should be accessible"),
    )
    .expect("Validation set file should be illegal");

    let num_classes = train_input
        .weights
        .as_ref()
        .expect("Train set file should contain label weights")
        .len();
    let num_iters = train_input.tagged.len() / config.batch_size * config.num_epochs;

    let dataset_train =
        ImageDataSet::train(train_input.tagged).expect("Training set failed to be loaded");
    let dataloader_train = DataLoaderBuilder::new(batcher_train)
        .batch_size(config.batch_size)
        .shuffle(config.seed)
        .num_workers(config.num_workers)
        .build(dataset_train);

    let dataloader_valid = DataLoaderBuilder::new(batcher_valid)
        .batch_size(config.batch_size)
        .shuffle(config.seed)
        .num_workers(config.num_workers)
        .build(ImageDataSet::valid(valid_input.tagged).expect("Validation set faild to be loaded"));

    let learner = LearnerBuilder::new(&artifact_dir)
        .metric_train_numeric(HammingScore::new().with_threshold(config.confidence_threshold))
        .metric_valid_numeric(HammingScore::new().with_threshold(config.confidence_threshold))
        .metric_train_numeric(LossMetric::new())
        .metric_train(LearningRateMetric::new())
        .metric_train(CudaMetric::new())
        .metric_train(CpuUse::new())
        .metric_train(CpuMemory::new())
        .early_stopping(MetricEarlyStoppingStrategy::new::<HammingScore<B>>(
            Aggregate::Mean,
            Direction::Highest,
            Split::Valid,
            StoppingCondition::NoImprovementSince { n_epochs: config.early_stopping },
        ))
        .with_file_checkpointer(CompactRecorder::new())
        .devices(vec![device.clone()])
        .num_epochs(config.num_epochs)
        .summary()
        .build(
            {
                let mut model = config.model.with_weights(train_input.weights).init::<B>(&device, num_classes);
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
            LinearLrSchedulerConfig::new(config.learning_rate, config.learning_rate/10., num_iters ).init(),
        );

    let model_trained = learner.fit(dataloader_train, dataloader_valid);

    model_trained
        .save_file(artifact_dir.join("model"), &CompactRecorder::new())
        .expect("Trained model should be saved successfully");
}
