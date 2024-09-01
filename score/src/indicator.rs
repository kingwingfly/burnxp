use burn::{prelude::Backend, tensor::Tensor};

#[derive(Debug, Clone, Copy)]
pub(crate) struct IndicatorSet {
    pub(crate) age: i16,
    pub(crate) body: i16,
    pub(crate) emotion: i16,
    pub(crate) face: i16,
    pub(crate) hair: i16,
    pub(crate) outfit: i16,
    pub(crate) pose: i16,
    pub(crate) xp: i16,
    pub(crate) score: i16,
}

impl IndicatorSet {
    pub(crate) fn to_tensor<B: Backend>(self) -> Tensor<B, 1> {
        Tensor::from_data(
            [
                self.age,
                self.body,
                self.emotion,
                self.face,
                self.hair,
                self.outfit,
                self.pose,
                self.xp,
            ],
            &B::Device::default(),
        )
    }

    pub(crate) fn score<B: Backend>(self) -> Tensor<B, 1> {
        Tensor::from_data([self.score], &B::Device::default())
    }
}
