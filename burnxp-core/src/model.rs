use crate::data::ImageBatch;
use burn::prelude::*;
use burn::tensor::backend::AutodiffBackend;
use burn::train::{MultiLabelClassificationOutput, TrainOutput, TrainStep, ValidStep};
use clap::builder::OsStr;
use clap::ValueEnum;
use nn::loss::{BinaryCrossEntropyLoss, BinaryCrossEntropyLossConfig};
use nn::{Linear, LinearConfig, Relu, Sigmoid};
use resnet_burn::weights::{ResNet101, ResNet152, ResNet18, ResNet34, ResNet50};
use resnet_burn::ResNet;
use serde::{Deserialize, Serialize};

#[derive(Module, Debug)]
pub(crate) struct Model<B: Backend> {
    resnet: ResNet<B>,
    relu: Relu,
    linear: Linear<B>,
    sigmoid: Sigmoid,
    loss: BinaryCrossEntropyLoss<B>,
}

impl<B: Backend> Model<B> {
    /// # Shapes
    ///   - Images [batch_size, 3, SIZE, SIZE]
    ///   - Output [batch_size, num_classes]
    pub fn forward(&self, datas: Tensor<B, 4>) -> Tensor<B, 2> {
        let datas = datas.to_device(&self.devices()[0]);
        let x = self.resnet.forward(datas); // [batch_size, 1000]
        let x = self.relu.forward(x);
        let x = self.linear.forward(x); // [batch_size, num_classes]
        self.sigmoid.forward(x)
    }

    fn forward_multilabelclassification(
        &self,
        batch: ImageBatch<B>,
    ) -> MultiLabelClassificationOutput<B> {
        let output = self.forward(batch.datas);
        let taget = batch.targets.to_device(&self.devices()[0]);
        let loss = self.loss.forward(output.clone(), taget.clone());
        MultiLabelClassificationOutput::new(loss, output, taget)
    }
}

impl<B: AutodiffBackend> TrainStep<ImageBatch<B>, MultiLabelClassificationOutput<B>> for Model<B> {
    fn step(&self, batch: ImageBatch<B>) -> TrainOutput<MultiLabelClassificationOutput<B>> {
        let classify = self.forward_multilabelclassification(batch);
        TrainOutput::new(self, classify.loss.backward(), classify)
    }
}

impl<B: Backend> ValidStep<ImageBatch<B>, MultiLabelClassificationOutput<B>> for Model<B> {
    fn step(&self, batch: ImageBatch<B>) -> MultiLabelClassificationOutput<B> {
        self.forward_multilabelclassification(batch)
    }
}

#[derive(Config, Debug)]
pub struct ModelConfig {
    rnn_type: ResNetType,
    #[config(default = false)]
    download: bool,
    loss_weights: Option<Vec<f32>>,
}

impl ModelConfig {
    pub(crate) fn init<B: Backend>(&self, device: &B::Device, num_classes: usize) -> Model<B> {
        let resnet = if self.download {
            match self.rnn_type {
                ResNetType::Layer18 => ResNet::resnet18_pretrained(ResNet18::ImageNet1kV1, device),
                ResNetType::Layer34 => ResNet::resnet34_pretrained(ResNet34::ImageNet1kV1, device),
                ResNetType::Layer50 => ResNet::resnet50_pretrained(ResNet50::ImageNet1kV2, device),
                ResNetType::Layer101 => {
                    ResNet::resnet101_pretrained(ResNet101::ImageNet1kV2, device)
                }
                ResNetType::Layer152 => {
                    ResNet::resnet152_pretrained(ResNet152::ImageNet1kV2, device)
                }
            }
            .expect("Failed to download the model")
        } else {
            match self.rnn_type {
                ResNetType::Layer18 => ResNet::resnet18(1000, device),
                ResNetType::Layer34 => ResNet::resnet34(1000, device),
                ResNetType::Layer50 => ResNet::resnet50(1000, device),
                ResNetType::Layer101 => ResNet::resnet101(1000, device),
                ResNetType::Layer152 => ResNet::resnet152(1000, device),
            }
        };
        Model {
            resnet,
            relu: Relu,
            linear: LinearConfig::new(1000, num_classes).init(device),
            sigmoid: Sigmoid,
            loss: BinaryCrossEntropyLossConfig::new()
                .with_smoothing(Some(0.1))
                .with_weights(self.loss_weights.clone())
                .init(device),
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
