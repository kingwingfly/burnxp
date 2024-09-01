use burn::prelude::*;
use burn::tensor::backend::AutodiffBackend;
use burn::{
    config::Config,
    module::Module,
    train::{RegressionOutput, TrainOutput, TrainStep, ValidStep},
};
use nn::LeakyReluConfig;
use nn::{
    loss::{HuberLossConfig, Reduction},
    LeakyRelu, Linear, LinearConfig,
};

use crate::data::PicBatch;

#[derive(Module, Debug)]
pub(crate) struct ScoreModel<B: Backend> {
    linear1: Linear<B>,
    linear2: Linear<B>,
    activation: LeakyRelu,
}

impl<B: Backend> ScoreModel<B> {
    /// # Shapes
    ///   - Images [batch_size, indicator_size]
    ///   - Output [batch_size, score]
    fn forward(&self, datas: Tensor<B, 2>) -> Tensor<B, 2> {
        let x = self.linear1.forward(datas);
        let x = self.activation.forward(x);
        self.linear2.forward(x) // [batch_size, score]
    }

    fn forward_classification(
        &self,
        datas: Tensor<B, 2>,
        target_scores: Tensor<B, 2>,
    ) -> RegressionOutput<B> {
        let output = self.forward(datas);
        let loss = HuberLossConfig::new(0.5).init().forward(
            output.clone(),
            target_scores.clone(),
            Reduction::Auto,
        );
        RegressionOutput::new(loss, output, target_scores)
    }
}

impl<B: AutodiffBackend> TrainStep<PicBatch<B>, RegressionOutput<B>> for ScoreModel<B> {
    fn step(&self, batch: PicBatch<B>) -> TrainOutput<RegressionOutput<B>> {
        let item = self.forward_classification(batch.datas, batch.target_scores);
        TrainOutput::new(self, item.loss.backward(), item)
    }
}

impl<B: Backend> ValidStep<PicBatch<B>, RegressionOutput<B>> for ScoreModel<B> {
    fn step(&self, batch: PicBatch<B>) -> RegressionOutput<B> {
        self.forward_classification(batch.datas, batch.target_scores)
    }
}

#[derive(Config, Debug)]
pub struct ScoreModelConfig {
    #[config(default = 32)]
    hidden_size: usize,
}

impl ScoreModelConfig {
    pub(crate) fn init<B: Backend>(&self, device: &B::Device) -> ScoreModel<B> {
        ScoreModel {
            linear1: LinearConfig::new(8, self.hidden_size)
                .with_bias(true)
                .init(device),
            linear2: LinearConfig::new(self.hidden_size, 1)
                .with_bias(true)
                .init(device),
            activation: LeakyReluConfig::new().init(),
        }
    }
}
