use burn::prelude::*;
use burn::tensor::backend::AutodiffBackend;
use burn::{
    config::Config,
    module::Module,
    train::{ClassificationOutput, TrainOutput, TrainStep, ValidStep},
};
use nn::{loss::CrossEntropyLoss, Linear, LinearConfig, Relu};

use crate::data::PicBatch;

#[derive(Module, Debug)]
pub(crate) struct ScoreModel<B: Backend> {
    linear1: Linear<B>,
    linear2: Linear<B>,
    activation: Relu,
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
        target_scores: Tensor<B, 1, Int>,
    ) -> ClassificationOutput<B> {
        let output = self.forward(datas);
        let loss = CrossEntropyLoss::new(None, &output.device())
            .forward(output.clone(), target_scores.clone());

        ClassificationOutput::new(loss, output, target_scores)
    }
}

impl<B: AutodiffBackend> TrainStep<PicBatch<B>, ClassificationOutput<B>> for ScoreModel<B> {
    fn step(&self, batch: PicBatch<B>) -> TrainOutput<ClassificationOutput<B>> {
        let item = self.forward_classification(batch.datas, batch.target_scores);
        TrainOutput::new(self, item.loss.backward(), item)
    }
}

impl<B: Backend> ValidStep<PicBatch<B>, ClassificationOutput<B>> for ScoreModel<B> {
    fn step(&self, batch: PicBatch<B>) -> ClassificationOutput<B> {
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
            linear1: LinearConfig::new(8, self.hidden_size).init(device),
            linear2: LinearConfig::new(self.hidden_size, 11).init(device),
            activation: Relu,
        }
    }
}
