use core::fmt;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

pub(crate) struct CheckBox<'a, T>
where
    T: fmt::Display,
{
    items: &'a [T],
    heighlight: &'a [bool],
}

impl<'a, T> CheckBox<'a, T>
where
    T: fmt::Display,
{
    pub fn new(items: &'a [T], heighlight: &'a [bool]) -> Self {
        Self { items, heighlight }
    }
}

impl<'a, T> Widget for CheckBox<'a, T>
where
    T: fmt::Display,
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        let grid = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3); 3])
            .split(area)
            .iter()
            .flat_map(|&line| {
                Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Ratio(1, 3); 3])
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
                    .style(Style::default().fg(match self.heighlight[i] {
                        true => Color::LightYellow,
                        false => Color::White,
                    }));
                let inner = block.inner(chunk);
                block.render(chunk, buf);
                inner
            })
            .collect::<Vec<_>>();
        for (i, item) in self.items.iter().enumerate() {
            let t = format!("{}", item);
            Paragraph::new(t)
                .wrap(Wrap { trim: true })
                .render(grid[i], buf);
        }
    }
}
