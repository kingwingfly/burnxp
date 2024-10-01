use image::DynamicImage;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io;
use std::sync::atomic::Ordering;
use std::{fs::File, path::PathBuf};

use crate::state::PICKER_PROCESS;

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

pub(crate) fn picker() -> ratatui_image::picker::Picker {
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
            self.count = 0;
            self.inner = picker();
        } else {
            self.count += 1;
        }
        self.inner.new_resize_protocol(image)
    }
}

#[derive(Debug, Default)]
enum PreLoadDirection {
    #[default]
    Forward,
    Backward,
}

#[derive(Debug, Default)]
pub(crate) struct Items<T, const N: usize> {
    page: usize,
    direction: PreLoadDirection,
    items: Vec<T>,
}

impl<T, const N: usize> Items<T, N> {
    pub(crate) fn new(items: Vec<T>) -> Items<T, N> {
        PICKER_PROCESS
            .total
            .fetch_add(items.len(), Ordering::Relaxed);
        Self {
            items,
            page: 0,
            direction: PreLoadDirection::default(),
        }
    }

    pub(crate) fn page(&self) -> usize {
        self.page
    }

    pub(crate) fn inc_page(&mut self) -> bool {
        self.direction = PreLoadDirection::Forward;
        if (self.page + 1) * N < self.items.len() {
            self.page += 1;
            false
        } else {
            true
        }
    }

    pub(crate) fn dec_page(&mut self) {
        if self.page > 0 {
            self.page -= 1;
        }
        self.direction = PreLoadDirection::Backward;
    }

    pub(crate) fn set_page(&mut self, page: usize) {
        let page = page.min(self.items.len().saturating_sub(1) / N);
        self.page = page;
    }

    pub(crate) fn current_items(&self) -> &[T] {
        PICKER_PROCESS
            .finished
            .store(N * self.page, Ordering::Relaxed);
        let l = self.page * N;
        let r = (self.page + 1) * N;
        &self.items[l..r.min(self.items.len())]
    }

    pub(crate) fn preload_items(&self) -> &[T] {
        match self.direction {
            PreLoadDirection::Forward if (self.page + 1) * N < self.items.len() => {
                let l = (self.page + 1) * N;
                let r = (self.page + 2) * N;
                &self.items[l..r.min(self.items.len())]
            }
            PreLoadDirection::Backward if self.page > 1 => {
                let l = (self.page - 1) * N;
                let r = self.page * N;
                &self.items[l..r.min(self.items.len())]
            }
            _ => &[],
        }
    }
}
