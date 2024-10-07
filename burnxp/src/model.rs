use crate::data::ImageBatch;
use burn::prelude::*;
use burn::tensor::backend::AutodiffBackend;
use burn::train::{MultiLabelClassificationOutput, TrainOutput, TrainStep, ValidStep};
use clap::builder::OsStr;
use clap::ValueEnum;
use nn::loss::BinaryCrossEntropyLossConfig;
use nn::Sigmoid;
use resnet_burn::ResNet;
use serde::{Deserialize, Serialize};

#[derive(Module, Debug)]
pub(crate) struct ScoreModel<B: Backend> {
    resnet: ResNet<B>,
    sigmoid: Sigmoid,
}

impl<B: Backend> ScoreModel<B> {
    /// # Shapes
    ///   - Images [batch_size, 1024, 1024, 3]
    ///   - Output [batch_size, num_classes]
    pub fn forward(&self, datas: Tensor<B, 4>) -> Tensor<B, 2> {
        let x = self.resnet.forward(datas); // [batch_size, num_classes]
        self.sigmoid.forward(x)
    }

    fn forward_multilabelclassification(
        &self,
        batch: ImageBatch<B>,
    ) -> MultiLabelClassificationOutput<B> {
        let output = self.forward(batch.datas);
        let loss = BinaryCrossEntropyLossConfig::new()
            .with_smoothing(Some(0.1))
            .init(&B::Device::default())
            .forward(output.clone(), batch.targets.clone());
        MultiLabelClassificationOutput::new(loss, output, batch.targets)
    }
}

impl<B: AutodiffBackend> TrainStep<ImageBatch<B>, MultiLabelClassificationOutput<B>>
    for ScoreModel<B>
{
    fn step(&self, batch: ImageBatch<B>) -> TrainOutput<MultiLabelClassificationOutput<B>> {
        let reg = self.forward_multilabelclassification(batch);
        TrainOutput::new(self, reg.loss.backward(), reg)
    }
}

impl<B: Backend> ValidStep<ImageBatch<B>, MultiLabelClassificationOutput<B>> for ScoreModel<B> {
    fn step(&self, batch: ImageBatch<B>) -> MultiLabelClassificationOutput<B> {
        self.forward_multilabelclassification(batch)
    }
}

#[derive(Config, Debug)]
pub struct ScoreModelConfig {
    rnn_type: ResNetType,
}

impl ScoreModelConfig {
    pub(crate) fn init<B: Backend>(&self, device: &B::Device, num_classes: usize) -> ScoreModel<B> {
        let resnet = match self.rnn_type {
            ResNetType::Layer18 => ResNet::resnet18(num_classes, device),
            ResNetType::Layer34 => ResNet::resnet34(num_classes, device),
            ResNetType::Layer50 => ResNet::resnet50(num_classes, device),
            ResNetType::Layer101 => ResNet::resnet101(num_classes, device),
            ResNetType::Layer152 => ResNet::resnet152(num_classes, device),
        };
        ScoreModel {
            resnet,
            sigmoid: Sigmoid,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, ValueEnum)]
pub enum ResNetType {
    Layer18 = 18,
    Layer34 = 34,
    Layer50 = 50,
    #[default]
    Layer101 = 101,
    Layer152 = 152,
}

impl From<ResNetType> for OsStr {
    fn from(value: ResNetType) -> Self {
        format!("layer{:?}", value as usize).into()
    }
}
