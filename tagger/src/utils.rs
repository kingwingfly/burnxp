use ratatui::layout::{Constraint, Direction, Layout, Rect};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io;
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

pub(crate) fn bincode_from<T: DeserializeOwned>(path: &PathBuf) -> io::Result<T> {
    File::open(path).and_then(|f| {
        bincode::deserialize_from(f).map_err(|_| io::Error::from(io::ErrorKind::InvalidData))
    })
}

#[allow(unused)]
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
