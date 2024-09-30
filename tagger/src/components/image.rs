use crate::components::Title;
use anyhow::{anyhow, Result};
use lru::LruCache;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Widget},
};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol, FilterType, Resize};
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, LazyLock, RwLock};
use std::thread;

static CACHE: LazyLock<RwLock<LruCache<PathBuf, Box<dyn StatefulProtocol>>>> =
    LazyLock::new(|| RwLock::new(LruCache::new(NonZeroUsize::new(45).unwrap())));
static RESIZE: Resize = Resize::Fit(Some(FilterType::Lanczos3));
const CACHE_ERR: &str = "Race condition of Cache RwLock";
const PICKER_ERR: &str = "Race condition of PICKER_ERR RwLock";

pub(crate) struct Images<'a> {
    picker: Arc<RwLock<Picker>>,
    paths: &'a [&'a PathBuf; 2],
}

impl<'a> Images<'a> {
    pub(crate) fn new(paths: &'a [&'a PathBuf; 2]) -> Self {
        #[cfg(not(target_os = "windows"))]
        let mut picker = Picker::from_termios()
            .map_err(|_| anyhow::anyhow!("Failed to get the picker"))
            .unwrap();
        #[cfg(target_os = "windows")]
        let mut picker = {
            let mut picker = Picker::new((12, 24));
            picker.protocol_type = ratatui_image::picker::ProtocolType::Iterm2;
            picker
        };
        picker.guess_protocol();
        Self {
            picker: Arc::new(RwLock::new(picker)),
            paths,
        }
    }
}

impl<'a> Widget for Images<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);
        let mut images = Vec::with_capacity(2);
        for i in 0..2 {
            images.push(Image::new(
                self.picker.clone(),
                self.paths[i].clone(),
                chunks[i],
            ));
        }
        for (i, image) in images.into_iter().enumerate() {
            image.render(chunks[i], buf);
        }
    }
}

pub struct Grid<'a> {
    picker: Arc<RwLock<Picker>>,
    paths: &'a [PathBuf],
    heighlight: [bool; 9],
}

impl<'a> Grid<'a> {
    pub(crate) fn new(paths: &'a [PathBuf], heighlight: [bool; 9]) -> Self {
        #[cfg(not(target_os = "windows"))]
        let mut picker = ratatui_image::picker::Picker::from_termios()
            .map_err(|_| anyhow::anyhow!("Failed to get the picker"))
            .unwrap();
        #[cfg(target_os = "windows")]
        let mut picker = {
            let mut picker = ratatui_image::picker::Picker::new((12, 24));
            picker.protocol_type = ratatui_image::picker::ProtocolType::Iterm2;
            picker
        };
        picker.guess_protocol();
        Self {
            picker: Arc::new(RwLock::new(picker)),
            paths,
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
        for (i, path) in self.paths.iter().enumerate() {
            images.push(Image::new(self.picker.clone(), path.clone(), grid[i]));
        }
        for (i, image) in images.into_iter().enumerate() {
            image.render(grid[i], buf);
        }
    }
}

struct Image {
    rx: Receiver<Box<dyn StatefulProtocol>>,
}

impl Image {
    pub(crate) fn new(picker: Arc<RwLock<Picker>>, path: PathBuf, chunk: Rect) -> Self {
        let (tx, rx) = channel::<Box<dyn StatefulProtocol>>();
        thread::spawn(move || -> Result<()> {
            let hit = {
                let cache = CACHE.read().map_err(|_| anyhow!(CACHE_ERR))?;
                cache.contains(&path)
            };
            match hit {
                true => {
                    let resize = {
                        let mut cache = CACHE.write().map_err(|_| anyhow!(CACHE_ERR))?;
                        cache
                            .get_mut(&path)
                            .unwrap()
                            .needs_resize(&RESIZE, chunk)
                            .is_some()
                    };
                    match resize {
                        true => {
                            let mut protocol = {
                                let mut cache = CACHE.write().map_err(|_| anyhow!(CACHE_ERR))?;
                                cache.pop(&path).unwrap()
                            };
                            protocol.resize_encode(&RESIZE, None, chunk);
                            let mut cache = CACHE.write().map_err(|_| anyhow!(CACHE_ERR))?;
                            cache.put(path, protocol.clone());
                            tx.send(protocol)?;
                        }
                        false => {
                            let mut cache = CACHE.write().map_err(|_| anyhow!(CACHE_ERR))?;
                            let protocol = cache.get(&path).unwrap();
                            tx.send(protocol.clone())?;
                        }
                    }
                }
                false => {
                    let image = image::open(&path)?;
                    let mut image_fit_state = {
                        let mut picker = picker.write().map_err(|_| anyhow!(PICKER_ERR))?;
                        picker.new_resize_protocol(image)
                    };
                    image_fit_state.resize_encode(&RESIZE, None, chunk);
                    {
                        let mut cache = CACHE.write().map_err(|_| anyhow!(PICKER_ERR))?;
                        cache.put(path, image_fit_state.clone());
                    }
                    tx.send(image_fit_state)?;
                }
            }
            Ok(())
        });
        Self { rx }
    }
}

impl Widget for Image {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.rx.recv() {
            Ok(mut protocol) => protocol.render(area, buf),
            Err(e) => Title {
                title: format!("Failed to render image: {e}"),
            }
            .render(area, buf),
        }
    }
}
