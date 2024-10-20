use anyhow::Result;
use burn::{
    data::dataloader::{batcher::Batcher, Dataset},
    prelude::*,
};
use image::{
    imageops::colorops::brighten_in_place, imageops::FilterType, DynamicImage, ImageBuffer, Rgb,
};
use imageproc::geometric_transformations::{rotate_about_center, Interpolation};
use mime_guess::MimeGuess;
use rand::{seq::SliceRandom, thread_rng, Rng};
use std::{collections::HashMap, path::Path};
use std::{f32::consts::PI, path::PathBuf};
use tagger::{BitFlags, DataSetDesc};

const SIZE: usize = 720;
const HALF: i64 = (SIZE / 2) as i64;

#[derive(Debug, Clone)]
pub(crate) struct ImageData<B: Backend> {
    data: Tensor<B, 1>,
    #[cfg(feature = "tch")]
    tags: Tensor<B, 1, Int>,
    #[cfg(feature = "candle")]
    tags: Tensor<B, 1, Int>,
    path: PathBuf,
}

pub(crate) struct ImageDataSet {
    num_classes: usize,
    pub len: usize,
    up_sample: Option<Vec<(usize, BitFlags)>>,
    binary_encodings: HashMap<BitFlags, Vec<PathBuf>>,
}

impl ImageDataSet {
    pub(crate) fn train(desc: DataSetDesc) -> Result<Self> {
        for path in desc.binary_encodings.iter().flat_map(|(_, v)| v) {
            assert!(
                path.canonicalize().is_ok(),
                "expected {} to exist",
                path.display()
            );
        }
        Ok(Self {
            num_classes: desc.num_classes,
            len: desc.up_sample.values().sum(),
            up_sample: Some(desc.up_sample.into_iter().fold(vec![], |mut acc, (k, v)| {
                match acc.last() {
                    None => acc.push((v, k)),
                    Some((v1, _)) => acc.push((v1 + v, k)),
                }
                acc
            })),
            binary_encodings: desc.binary_encodings,
        })
    }

    pub(crate) fn valid(desc: DataSetDesc) -> Result<Self> {
        for path in desc.binary_encodings.iter().flat_map(|(_, v)| v) {
            assert!(
                path.canonicalize().is_ok(),
                "expected {} to exist",
                path.display()
            );
        }
        Ok(Self {
            num_classes: desc.num_classes,
            len: desc.binary_encodings.values().map(|v| v.len()).sum(),
            up_sample: None,
            binary_encodings: desc.binary_encodings,
        })
    }

    pub(crate) fn predict(path: PathBuf) -> Result<Self> {
        let inner = walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(|res| res.ok())
            .filter_map(|e| match MimeGuess::from_path(e.path()).first() {
                Some(mime) if mime.type_() == "image" => Some(e.into_path()),
                _ => None,
            })
            .collect::<Vec<_>>();
        Ok(Self {
            num_classes: 0,
            len: inner.len(),
            up_sample: None,
            binary_encodings: [(0.into(), inner)].into(),
        })
    }
}

impl<B: Backend> Dataset<ImageData<B>> for ImageDataSet {
    fn get(&self, index: usize) -> Option<ImageData<B>> {
        match self.up_sample {
            None => self
                .binary_encodings
                .iter()
                .flat_map(|(k, v)| v.iter().map(move |path| (path, k)))
                .nth(index)
                .map(|(path, flags)| ImageData {
                    data: open_image_normalize(path)
                        .unwrap_or_else(|| panic!("Failed to load image {}", path.display())),
                    tags: {
                        let mut t: Vec<i8> = Vec::from(*flags);
                        t.truncate(self.num_classes);
                        Tensor::from_ints(&t[..], &B::Device::default())
                    },
                    path: path.clone(),
                }),
            Some(ref up_sample) => {
                let flags = up_sample
                    .iter()
                    .find(|(v, _)| index < *v)
                    .map(|(_, f)| f)
                    .expect("Index out of bounds");
                let path = self.binary_encodings[flags]
                    .choose(&mut thread_rng())
                    .expect("Index out of bounds");
                Some(ImageData {
                    data: open_image_proc(path).unwrap_or_else(|| {
                        panic!("Failed to load and process image {}", path.display())
                    }),
                    tags: {
                        let mut t: Vec<i8> = Vec::from(*flags);
                        t.truncate(self.num_classes);
                        Tensor::from_ints(&t[..], &B::Device::default())
                    },
                    path: path.clone(),
                })
            }
        }
    }

    fn len(&self) -> usize {
        self.len
    }
}

#[derive(Clone)]
pub(crate) struct ImageBatcher {}

#[derive(Debug, Clone)]
pub(crate) struct ImageBatch<B: Backend> {
    pub datas: Tensor<B, 4>,
    pub targets: Tensor<B, 2, Int>,
    pub paths: Vec<PathBuf>,
}

impl ImageBatcher {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl<B: Backend> Batcher<ImageData<B>, ImageBatch<B>> for ImageBatcher {
    fn batch(&self, items: Vec<ImageData<B>>) -> ImageBatch<B> {
        let (paths, datas, targets) = items.into_iter().fold(
            (vec![], vec![], vec![]),
            |(mut paths, mut datas, mut targets), item| {
                paths.push(item.path);
                datas.push(item.data.reshape([1, 3, SIZE, SIZE]));
                targets.push(item.tags.reshape([1, -1]));
                (paths, datas, targets)
            },
        );

        let datas = Tensor::cat(datas, 0);
        let targets = Tensor::cat(targets, 0);

        ImageBatch {
            datas,
            targets,
            paths,
        }
    }
}

fn open_image_proc<B: Backend>(path: impl AsRef<Path>) -> Option<Tensor<B, 1>> {
    let size = SIZE as u32;
    let mut img = open_image_resize(path)?;
    let mut rng = thread_rng();
    if rng.gen_bool(0.5) {
        img = img.fliph();
    }
    let theta = rng.gen_range(-1. / 6. ..1. / 6.);
    let buffer = img.to_rgb8().into_raw();
    let buffer = ImageBuffer::<Rgb<u8>, Vec<u8>>::from_vec(size, size, buffer)?;
    let mut buffer =
        rotate_about_center(&buffer, theta * PI, Interpolation::Nearest, Rgb([0, 0, 0]));
    brighten_in_place(&mut buffer, rng.gen_range(-64..64));
    Some(Tensor::from_data(&buffer.into_raw()[..], &B::Device::default()) / 255.0)
}

fn open_image_normalize<B: Backend>(path: impl AsRef<Path>) -> Option<Tensor<B, 1>> {
    Some(
        Tensor::from_data(
            &open_image_resize(path)?.to_rgb8().into_raw()[..],
            &B::Device::default(),
        ) / 255.0,
    )
}

fn open_image_resize(path: impl AsRef<Path>) -> Option<DynamicImage> {
    let size = SIZE as u32;
    let img = image::open(path.as_ref().canonicalize().ok()?).ok()?;
    let mut background = image::RgbImage::new(size, size);

    let factor = img.height().max(img.width()) as f64 / size as f64;
    if factor == 0. {
        // an invalid image
        return None;
    }
    let nheight = (img.height() as f64 / factor).min(size as f64) as u32;
    let nwidth = (img.width() as f64 / factor).min(size as f64) as u32;
    let img = img.resize(nwidth, nheight, FilterType::Gaussian);
    image::imageops::overlay(
        &mut background,
        &img.to_rgb8(),
        HALF - (nwidth / 2) as i64,
        HALF - (nheight / 2) as i64,
    );
    Some(DynamicImage::ImageRgb8(background))
}
