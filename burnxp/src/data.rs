use crate::train::Input;
use anyhow::Result;
use burn::{
    data::dataloader::{batcher::Batcher, Dataset},
    prelude::*,
};
use image::imageops::FilterType;
use mime_guess::MimeGuess;
use std::path::Path;
use std::path::PathBuf;

const SIZE: usize = 720;
const HALF: i64 = (SIZE / 2) as i64;

#[derive(Debug, Clone)]
pub(crate) struct ImageData {
    data: Vec<f32>,
    tags: Vec<i8>,
    path: PathBuf,
}

impl ImageData {
    pub(crate) fn data<B: Backend>(&self) -> Tensor<B, 1> {
        Tensor::from_data(&self.data[..], &B::Device::default())
    }

    pub(crate) fn tags<B: Backend>(&self) -> Tensor<B, 1, Int> {
        Tensor::from_data(&self.tags[..], &B::Device::default())
    }
}

pub(crate) struct ImageDataSet {
    inner: Vec<(PathBuf, Vec<i8>)>,
}

impl ImageDataSet {
    pub(crate) fn train(input: Input) -> Result<Self> {
        Ok(Self {
            inner: input.tagged,
        })
    }

    pub(crate) fn valid(input: Input) -> Result<Self> {
        Ok(Self {
            inner: input.tagged,
        })
    }

    pub(crate) fn predict(path: PathBuf) -> Result<Self> {
        let inner = walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(|res| res.ok())
            .filter_map(|e| match MimeGuess::from_path(e.path()).first() {
                Some(mime) if mime.type_() == "image" => Some((e.into_path(), vec![])),
                _ => None,
            })
            .collect();
        Ok(Self { inner })
    }
}

impl Dataset<ImageData> for ImageDataSet {
    fn get(&self, index: usize) -> Option<ImageData> {
        self.inner.get(index).and_then(|(path, tags)| {
            Some(ImageData {
                data: open_image(path)?
                    .into_iter()
                    .map(|p| p as f32 / 255.0)
                    .collect(),
                tags: tags.clone(),
                path: path.clone(),
            })
        })
    }

    fn len(&self) -> usize {
        self.inner.len()
    }
}

#[derive(Clone)]
pub(crate) struct ImageBatcher<B: Backend> {
    device: B::Device,
}

#[derive(Debug, Clone)]
pub(crate) struct ImageBatch<B: Backend> {
    pub datas: Tensor<B, 4>,
    pub targets: Tensor<B, 2, Int>,
    pub paths: Vec<PathBuf>,
}

impl<B: Backend> ImageBatcher<B> {
    pub(crate) fn new(device: B::Device) -> Self {
        Self { device }
    }
}

impl<B: Backend> Batcher<ImageData, ImageBatch<B>> for ImageBatcher<B> {
    fn batch(&self, items: Vec<ImageData>) -> ImageBatch<B> {
        let datas = items
            .iter()
            .map(|item| item.data().reshape([1, 3, SIZE, SIZE]))
            .collect::<Vec<_>>();
        let targets = items
            .iter()
            .map(|item| item.tags().reshape([1, -1]))
            .collect::<Vec<_>>();

        let datas = Tensor::cat(datas, 0).to_device(&self.device);
        let targets = Tensor::cat(targets, 0).to_device(&self.device);
        let paths = items.iter().map(|item| item.path.clone()).collect();

        ImageBatch {
            datas,
            targets,
            paths,
        }
    }
}

pub fn open_image(path: impl AsRef<Path>) -> Option<Vec<u8>> {
    let size = SIZE as u32;
    let img = image::open(path.as_ref().canonicalize().ok()?).ok()?;
    let mut background = image::RgbImage::new(size, size);

    let factor = img.height().max(img.width()) / size;
    if factor == 0 {
        // an invalid image
        return None;
    }
    let nheight = (img.height() / factor).min(size);
    let nwidth = (img.width() / factor).min(size);

    let img = img.resize(nwidth, nheight, FilterType::Gaussian);
    let img = img.to_rgb8();

    image::imageops::overlay(
        &mut background,
        &img,
        HALF - (nwidth / 2) as i64,
        HALF - (nheight / 2) as i64,
    );

    Some(background.into_raw().to_vec())
}
