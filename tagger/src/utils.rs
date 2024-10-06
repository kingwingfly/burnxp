use image::DynamicImage;
use mime_guess::MimeGuess;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io;
use std::path::Path;
use std::{fs::File, path::PathBuf};

/// helper function to create a centered rect using up certain percentage of the available rect `r`
pub(crate) fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}

#[cfg(feature = "cmper")]
pub(crate) fn bincode_from<T: DeserializeOwned>(path: &PathBuf) -> io::Result<T> {
    File::open(path).and_then(|f| {
        bincode::deserialize_from(f).map_err(|_| io::Error::from(io::ErrorKind::InvalidData))
    })
}

pub(crate) fn json_from<T: DeserializeOwned>(path: &PathBuf) -> io::Result<T> {
    File::open(path).and_then(|f| {
        serde_json::from_reader(f).map_err(|_| io::Error::from(io::ErrorKind::InvalidData))
    })
}

#[cfg(feature = "cmper")]
pub(crate) fn bincode_into<T: Serialize>(path: &PathBuf, data: &T) -> io::Result<()> {
    File::create(path).and_then(|f| {
        bincode::serialize_into(f, data).map_err(|_| io::Error::from(io::ErrorKind::InvalidData))
    })
}

pub(crate) fn json_into<T: Serialize>(path: &PathBuf, data: &T) -> io::Result<()> {
    File::create(path).and_then(|f| {
        serde_json::to_writer_pretty(f, data)
            .map_err(|_| io::Error::from(io::ErrorKind::InvalidData))
    })
}

pub(crate) fn images_walk(root: impl AsRef<Path>) -> Vec<PathBuf> {
    walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(|res| res.ok())
        .filter_map(|e| match MimeGuess::from_path(e.path()).first() {
            Some(mime) if mime.type_() == "image" => Some(e.into_path()),
            _ => None,
        })
        .collect()
}

fn picker() -> ratatui_image::picker::Picker {
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
    picker
}

pub(crate) struct MyPicker {
    inner: Picker,
    count: u8,
}

impl MyPicker {
    pub(crate) fn new() -> Self {
        Self {
            inner: picker(),
            count: 0,
        }
    }

    pub(crate) fn new_resize_protocol(&mut self, image: DynamicImage) -> Box<dyn StatefulProtocol> {
        if self.count == 255 {
            self.count = 1;
            self.inner = picker();
        } else {
            self.count += 1;
        }
        self.inner.new_resize_protocol(image)
    }
}

#[derive(Debug, Default, Clone, Copy)]
enum PreLoadDirection {
    #[default]
    Forward,
    Backward,
}

impl std::ops::Not for PreLoadDirection {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            PreLoadDirection::Forward => PreLoadDirection::Backward,
            PreLoadDirection::Backward => PreLoadDirection::Forward,
        }
    }
}

/// N: the page size
#[derive(Debug, Default)]
pub(crate) struct Items<T, const N: usize> {
    page: usize,
    direction: PreLoadDirection,
    items: Vec<T>,
}

impl<T, const N: usize> Items<T, N> {
    pub(crate) fn new(items: Vec<T>) -> Items<T, N> {
        Self {
            items,
            page: 0,
            direction: PreLoadDirection::default(),
        }
    }

    pub(crate) fn page(&self) -> usize {
        self.page
    }

    // return true if over bound
    pub(crate) fn inc_page(&mut self) -> bool {
        self.direction = PreLoadDirection::Forward;
        if (self.page + 1) * N < self.items.len() {
            self.page += 1;
            false
        } else {
            true
        }
    }

    // return true if over bound
    pub(crate) fn dec_page(&mut self) -> bool {
        self.direction = PreLoadDirection::Backward;
        if self.page > 0 {
            self.page -= 1;
            false
        } else {
            true
        }
    }

    // set current page num, clamping between illegal range
    pub(crate) fn set_page(&mut self, page: usize) {
        let page = page.min(self.items.len().saturating_sub(1) / N);
        self.page = page;
    }

    /// Return Items in current page
    pub(crate) fn current_items(&self) -> &[T] {
        let l = self.page * N;
        let r = (self.page + 1) * N;
        &self.items[l..r.min(self.items.len())]
    }

    /// Return Items in next page according to the prediction
    pub(crate) fn preload_items(&self) -> &[T] {
        match self.direction {
            PreLoadDirection::Forward if (self.page + 1) * N < self.items.len() => {
                let l = (self.page + 1) * N;
                let r = (self.page + 2) * N;
                &self.items[l..r.min(self.items.len())]
            }
            PreLoadDirection::Backward if self.page > 0 => {
                let l = (self.page - 1) * N;
                let r = self.page * N;
                &self.items[l..r.min(self.items.len())]
            }
            _ => match !self.direction {
                PreLoadDirection::Forward if (self.page + 1) * N < self.items.len() => {
                    let l = (self.page + 1) * N;
                    let r = (self.page + 2) * N;
                    &self.items[l..r.min(self.items.len())]
                }
                PreLoadDirection::Backward if self.page > 0 => {
                    let l = (self.page - 1) * N;
                    let r = self.page * N;
                    &self.items[l..r.min(self.items.len())]
                }
                _ => &[],
            },
        }
    }

    pub(crate) fn push(&mut self, item: T)
    where
        T: PartialEq,
    {
        match self.items.iter_mut().find(|i| **i == item) {
            Some(i) => *i = item,
            None => self.items.push(item),
        }
    }

    pub(crate) fn remove(&mut self, item: &T)
    where
        T: PartialEq,
    {
        self.items.retain(|x| x != item);
    }
}
