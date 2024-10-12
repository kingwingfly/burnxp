#![cfg(any(feature = "tch", feature = "candle"))]

#[cfg(all(feature = "tch", feature = "candle"))]
compile_error!("features `tch` and `candle` are mutually exclusive");

mod cli;
mod data;
mod model;
mod predict;
mod train;

pub use cli::run;
pub use model::{ModelConfig, ResNetType};
pub use predict::{predict, Output, PredictConfig};
pub use train::{train, TrainingConfig};
