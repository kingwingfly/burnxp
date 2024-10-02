use std::fmt::Display;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph, Widget},
};

#[derive(Debug)]
pub(crate) struct Input {
    title: String,
    content: String,
}

impl Input {
    pub fn new(title: impl Display, content: impl Display) -> Self {
        Self {
            title: format!("{}", title),
            content: format!("{}", content),
        }
    }
}

impl Widget for Input {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Fill(1)])
            .split(area);
        let title = Paragraph::new(self.title).block(Block::default().borders(Borders::ALL));
        title.render(chunks[0], buf);
        let input = Paragraph::new(self.content).block(Block::default().borders(Borders::ALL));
        input.render(chunks[1], buf);
    }
}
