use burn::{
    backend::{libtorch::LibTorchDevice, Autodiff, LibTorch},
    optim::AdamConfig,
};
use clap::{Parser, Subcommand, ValueEnum};
use score::{train, RnnType, ScoreModelConfig, TrainingConfig};
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
        /// Directory to save artifacts
        #[arg(short, long)]
        artifact_dir: PathBuf,
        #[arg(short, long, default_value = "128")]
        num_epochs: usize,
        #[arg(short, long, default_value = "1")]
        batch_size: usize,
        #[arg(short, long, default_value = "1")]
        num_workers: usize,
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
    },
}

#[derive(Debug, Clone, Default, ValueEnum)]
enum Output {
    #[default]
    Tui,
    Tty,
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
                .with_num_workers(num_workers),
                device,
            );
        }
        SubCmd::Predict {
            model,
            checkpoint,
            input,
            output,
        } => {
            println!("Predicting with model: {:?}", model);
            println!("Predicting with checkpoint: {:?}", checkpoint);
            println!("Input image: {:?}", input);
            println!("Output image: {:?}", output);
        }
    }
}
