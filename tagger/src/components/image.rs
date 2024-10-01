use crate::{components::Title, utils::MyPicker};
use anyhow::{anyhow, Result};
use crossbeam::sync::Parker;
use lru::LruCache;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Widget},
};
use ratatui_image::{protocol::StatefulProtocol, FilterType, Resize};
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use std::sync::{LazyLock, RwLock};
use std::thread;

static CACHE: LazyLock<RwLock<LruCache<PathBuf, CacheLine>>> = LazyLock::new(|| {
    RwLock::new(LruCache::new(
        NonZeroUsize::new(CACHE_SIZE as usize).unwrap(),
    ))
});
static PICKER: LazyLock<RwLock<MyPicker>> = LazyLock::new(|| RwLock::new(MyPicker::new()));

const CACHE_SIZE: u8 = 45;
const RESIZE: Resize = Resize::Fit(Some(FilterType::Lanczos3));
const CACHE_ERR: &str = "Race condition of Cache RwLock";
const PICKER_ERR: &str = "Race condition of PICKER_ERR RwLock";

struct CacheLine {
    data: Option<Box<dyn StatefulProtocol>>,
    parker: Option<Parker>,
}

// Safety: we won't use the Parker's reference
unsafe impl Sync for CacheLine {}

pub(crate) struct Images<'a> {
    paths: [&'a PathBuf; 2],
}

impl<'a> Images<'a> {
    pub(crate) fn new(paths: [&'a PathBuf; 2]) -> Self {
        Self { paths }
    }
}

impl<'a> Widget for Images<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);
        let mut images = Vec::with_capacity(2);
        for (i, &path) in self.paths[..2].iter().enumerate() {
            images.push(Image::new(path.clone(), chunks[i]));
        }
        for (i, image) in images.into_iter().enumerate() {
            image.render(chunks[i], buf);
        }
    }
}

pub(crate) struct Grid<'a> {
    cur: &'a [PathBuf],
    pre: &'a [PathBuf],
    heighlight: [bool; 9],
}

impl<'a> Grid<'a> {
    pub(crate) fn new(cur: &'a [PathBuf], pre: &'a [PathBuf], heighlight: [bool; 9]) -> Self {
        Self {
            cur,
            pre,
            heighlight,
        }
    }
}

impl<'a> Widget for Grid<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let grid = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ])
            .split(area)
            .iter()
            .flat_map(|&line| {
                Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(33),
                        Constraint::Percentage(33),
                        Constraint::Percentage(33),
                    ])
                    .split(line)
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .enumerate()
            .map(|(i, chunk)| {
                let block = Block::default()
                    .title(format!("{}", i + 1))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(match self.heighlight[i] {
                        true => Color::Green,
                        false => Color::White,
                    }));
                let inner = block.inner(chunk);
                block.render(chunk, buf);
                inner
            })
            .collect::<Vec<_>>();
        let mut images = Vec::with_capacity(9);
        for (i, path) in self.cur.iter().enumerate() {
            images.push(Image::new(path.clone(), grid[i]));
        }
        for (i, image) in images.into_iter().enumerate() {
            image.render(grid[i], buf);
        }
        // Preload the next images
        for (i, path) in self.pre.iter().enumerate() {
            preload(path.clone(), grid[i % 9]);
        }
    }
}

struct Image {
    rx: Receiver<Box<dyn StatefulProtocol>>,
    path: PathBuf,
}

impl Image {
    pub(crate) fn new(path: PathBuf, chunk: Rect) -> Self {
        let (tx, rx) = channel::<Box<dyn StatefulProtocol>>();
        let path_c = path.clone();
        thread::spawn(move || -> Result<()> {
            let parker = {
                let mut cache = CACHE.write().map_err(|_| anyhow!(CACHE_ERR))?;
                cache.get_mut(&path).and_then(|line| line.parker.take())
            };
            if let Some(parker) = parker {
                parker.park();
            }
            let p = Parker::new();
            let u = p.unparker().clone();
            let hit = {
                let mut cache = CACHE.write().map_err(|_| anyhow!(CACHE_ERR))?;
                let hit = cache.pop(&path);
                cache.put(
                    path.clone(),
                    CacheLine {
                        data: None,
                        parker: Some(p),
                    },
                );
                hit
            };
            match hit {
                Some(mut line) => {
                    if let Some(data) = &mut line.data {
                        data.resize_encode(&RESIZE, None, chunk);
                        tx.send(data.clone())?;
                    }
                    let mut cache = CACHE.write().map_err(|_| anyhow!(CACHE_ERR))?;
                    cache.put(path, line);
                }
                _ => {
                    let image = image::open(&path).inspect_err(|_| u.unpark())?;
                    let mut image_fit_state = {
                        let mut picker = PICKER.write().map_err(|_| anyhow!(PICKER_ERR))?;
                        picker.new_resize_protocol(image)
                    };
                    image_fit_state.resize_encode(&RESIZE, None, chunk);
                    {
                        tx.send(image_fit_state.clone())?;
                        let mut cache = CACHE.write().map_err(|_| anyhow!(PICKER_ERR))?;
                        if let Some(cache_line) = cache.get_mut(&path) {
                            cache_line.data = Some(image_fit_state);
                        }
                    }
                }
            }
            u.unpark();
            Ok(())
        });
        Self { rx, path: path_c }
    }
}

impl Widget for Image {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.rx.recv() {
            Ok(mut protocol) => protocol.render(area, buf),
            Err(_) => Title {
                title: format!("Failed to render image: {}", self.path.display()),
            }
            .render(area, buf),
        }
    }
}

/// Preload image to cache
fn preload(path: PathBuf, chunk: Rect) {
    thread::spawn(move || -> Result<()> {
        let p = Parker::new();
        let u = p.unparker().clone();
        let hit = {
            let mut cache = CACHE.write().map_err(|_| anyhow!(CACHE_ERR))?;
            let hit = cache.pop(&path);
            cache.put(
                path.clone(),
                CacheLine {
                    data: None,
                    parker: Some(p),
                },
            );
            hit
        };
        match hit {
            Some(mut line) => {
                if let Some(data) = &mut line.data {
                    data.resize_encode(&RESIZE, None, chunk);
                }
                let mut cache = CACHE.write().map_err(|_| anyhow!(CACHE_ERR))?;
                cache.put(path, line);
            }
            None => {
                let image = image::open(&path).inspect_err(|_| u.unpark())?;
                let mut image_fit_state = {
                    let mut picker = PICKER.write().map_err(|_| anyhow!(PICKER_ERR))?;
                    picker.new_resize_protocol(image)
                };
                image_fit_state.resize_encode(&RESIZE, None, chunk);
                {
                    let mut cache = CACHE.write().map_err(|_| anyhow!(PICKER_ERR))?;
                    if let Some(cache_line) = cache.get_mut(&path) {
                        cache_line.data = Some(image_fit_state);
                    }
                }
            }
        }
        u.unpark();
        Ok(())
    });
}
