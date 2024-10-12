use crate::{predict, train, ModelConfig, Output, PredictConfig, ResNetType, TrainingConfig};
use burn::{backend::Autodiff, optim::AdamConfig};
use clap::{CommandFactory as _, Parser, Subcommand};
use clap_complete::{generate, Shell};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Cli {
    #[clap(subcommand)]
    subcmd: SubCmd,
}

#[derive(Debug, Subcommand)]
enum SubCmd {
    /// Train a ResNet model with scores produced by the tagger.
    Train {
        /// Model type
        #[arg(short, long, default_value=ResNetType::default())]
        model: ResNetType,
        /// Path to the training set json file produced by the tagger divide
        #[arg(short, long, default_value = "train.json")]
        train_set: PathBuf,
        /// Path to the validation set json file produced by the tagger divide
        #[arg(short, long, default_value = "valid.json")]
        valid_set: PathBuf,
        /// Directory to save artifacts (The directory will be recreated if it exists)
        #[arg(short, long, default_value = "burnxp_artifact")]
        artifact_dir: PathBuf,
        #[arg(short, long, default_value = "64")]
        num_epochs: usize,
        #[arg(short, long, default_value = "1")]
        batch_size: usize,
        /// Number of workers for data loading
        #[arg(short = 'w', long, default_value = "1")]
        num_workers: usize,
        /// Learning rate for the optimizer, decreasing to 1/10 of the given value
        #[arg(short, long, default_value = "1.0e-3")]
        learning_rate: f64,
        /// Number of epochs before allowing early stopping
        #[arg(short, long, default_value = "10")]
        early_stopping: usize,
        /// Path to the pretrained model checkpoint
        #[arg(short, long)]
        pretrained: Option<PathBuf>,
        /// Random seed for reproducibility
        #[arg(short, long, default_value = "42")]
        seed: u64,
        /// Confidence threshold for computing HammingScore
        #[arg(short, long, default_value = "0.5")]
        confidence_threshold: f32,
    },
    /// Predict using a ResNet model checkpoint
    Predict {
        /// Model type
        #[arg(short, long, default_value=ResNetType::Layer101)]
        model: ResNetType,
        /// Path to the model checkpoint
        #[arg(short, long)]
        checkpoint: PathBuf,
        /// Method to output the scores
        #[arg(short, long, default_value = "tty")]
        output: Output,
        #[arg(short, long, default_value = "32")]
        batch_size: usize,
        /// Number of workers for data loading
        #[arg(short = 'w', long, default_value = "8")]
        num_workers: usize,
        /// Path to the file which contains the tags' weights
        #[arg(short, long, default_value = "tags.json")]
        tags: PathBuf,
        /// Confidence threshold for the prediction,
        /// only tags with possibility greater than (1 - this value) will be output
        #[arg(long, default_value = "0.5")]
        confidence_threshold: f32,
        /// Root of images directory
        input: PathBuf,
    },
    /// generate auto completion script
    GenCompletion {
        /// shell name
        shell: Shell,
    },
}

#[cfg(feature = "tch")]
type MyBackend = burn::backend::LibTorch<f32, i8>;
#[cfg(feature = "candle")]
type MyBackend = burn::backend::Candle<f32, u8>;

type MyAutodiffBackend = Autodiff<MyBackend>;

pub fn run() {
    #[cfg(all(feature = "tch", target_os = "macos"))]
    let device = burn::backend::libtorch::LibTorchDevice::Mps;
    #[cfg(all(feature = "tch", not(target_os = "macos")))]
    let device = burn::backend::libtorch::LibTorchDevice::Cuda(0);

    #[cfg(all(feature = "candle", target_os = "macos"))]
    let device = burn::backend::candle::CandleDevice::Metal(0);
    #[cfg(all(feature = "candle", not(target_os = "macos")))]
    let device = burn::backend::candle::CandleDevice::Cuda(0);

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
            early_stopping,
            pretrained,
            seed,
            confidence_threshold,
        } => {
            train::<MyAutodiffBackend>(
                artifact_dir,
                TrainingConfig::new(
                    ModelConfig::new(model),
                    AdamConfig::new(),
                    train_set,
                    valid_set,
                )
                .with_num_epochs(num_epochs)
                .with_batch_size(batch_size)
                .with_num_workers(num_workers)
                .with_learning_rate(learning_rate)
                .with_pretrained(pretrained)
                .with_early_stopping(early_stopping)
                .with_seed(seed)
                .with_confidence_threshold(confidence_threshold),
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
            tags,
            confidence_threshold,
        } => predict::<MyBackend>(
            PredictConfig::new(model, checkpoint, input, output, tags)
                .with_batch_size(batch_size)
                .with_num_workers(num_workers)
                .with_confidence_threshold(confidence_threshold),
            device,
        ),
        SubCmd::GenCompletion { shell } => {
            generate(shell, &mut Cli::command(), "burnxp", &mut std::io::stdout());
        }
    }
}
