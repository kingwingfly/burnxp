use core::fmt;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

pub(crate) struct TagGrid<'a, T, const R: usize, const C: usize>
where
    T: fmt::Display,
{
    items: &'a [T],
}

impl<'a, T, const R: usize, const C: usize> TagGrid<'a, T, R, C>
where
    T: fmt::Display,
{
    pub fn new(items: &'a [T]) -> Self {
        Self { items }
    }
}

impl<'a, T, const R: usize, const C: usize> Widget for TagGrid<'a, T, R, C>
where
    T: fmt::Display,
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        let grid = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Ratio(1, R as u32); R])
            .split(area)
            .iter()
            .flat_map(|&line| {
                Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Ratio(1, C as u32); C])
                    .split(line)
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .enumerate()
            .map(|(i, chunk)| {
                let block = Block::default()
                    .title(format!("{}", i + 1))
                    .borders(Borders::ALL);
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
