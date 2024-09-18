use burn::{
    backend::{libtorch::LibTorchDevice, Autodiff, LibTorch},
    optim::AdamConfig,
};
use clap::{Parser, Subcommand};
use score::{predict, train, Output, PredictConfig, RnnType, ScoreModelConfig, TrainingConfig};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Cli {
    #[clap(subcommand)]
    subcmd: SubCmd,
}

#[derive(Debug, Subcommand)]
enum SubCmd {
    /// Train a model
    Train {
        /// Model type
        #[arg(short, long, default_value=RnnType::Layer101)]
        model: RnnType,
        /// Path to the training set json file produced by the tagger
        #[arg(short, long)]
        train_set: PathBuf,
        /// Path to the validation set json file produced by the tagger
        #[arg(short, long)]
        valid_set: PathBuf,
        /// Directory to save artifacts (The directory will be recreated if it exists)
        #[arg(short, long)]
        artifact_dir: PathBuf,
        #[arg(short = 'e', long, default_value = "128")]
        num_epochs: usize,
        #[arg(short, long, default_value = "1")]
        batch_size: usize,
        #[arg(short = 'w', long, default_value = "1")]
        num_workers: usize,
        #[arg(short, long, default_value = "1.0e-4")]
        learning_rate: f64,
        /// Path to the pretrained model checkpoint
        #[arg(short, long)]
        pretrained: Option<PathBuf>,
    },
    /// Predict using a ResNet model checkpoint
    Predict {
        /// Model type
        #[arg(short, long, default_value=RnnType::Layer101)]
        model: RnnType,
        /// Path to the model checkpoint
        #[arg(short, long)]
        checkpoint: PathBuf,
        /// Path to the test set root directory
        #[arg(short, long)]
        input: PathBuf,
        /// Path to the output image
        #[arg(short, long)]
        output: Output,
        #[arg(short, long, default_value = "1")]
        batch_size: usize,
        #[arg(short = 'w', long, default_value = "1")]
        num_workers: usize,
    },
}

type MyBackend = LibTorch<f32, i8>;
type MyAutodiffBackend = Autodiff<MyBackend>;

fn main() {
    #[cfg(target_os = "macos")]
    let device = LibTorchDevice::Mps;
    #[cfg(not(target_os = "macos"))]
    let device = LibTorchDevice::Cuda(0);

    let args = Cli::parse();
    match args.subcmd {
        SubCmd::Train {
            model,
            train_set,
            valid_set,
            artifact_dir,
            num_epochs,
            batch_size,
            num_workers,
            learning_rate,
            pretrained,
        } => {
            train::<MyAutodiffBackend>(
                artifact_dir,
                TrainingConfig::new(
                    ScoreModelConfig::new(model),
                    AdamConfig::new(),
                    train_set,
                    valid_set,
                )
                .with_num_epochs(num_epochs)
                .with_batch_size(batch_size)
                .with_num_workers(num_workers)
                .with_learning_rate(learning_rate)
                .with_pretrained(pretrained),
                device,
            );
        }
        SubCmd::Predict {
            model,
            checkpoint,
            input,
            output,
            batch_size,
            num_workers,
        } => predict::<MyBackend>(
            PredictConfig::new(model, checkpoint, input, output)
                .with_batch_size(batch_size)
                .with_num_workers(num_workers),
            device,
        ),
    }
}
