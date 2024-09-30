use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph, Widget},
};

#[derive(Debug)]
pub(crate) struct NumInput {
    pub num: usize,
}

impl Widget for NumInput {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);
        let title = Paragraph::new("Page to go:").block(Block::default().borders(Borders::ALL));
        title.render(chunks[0], buf);
        let input =
            Paragraph::new(format!("{}", self.num)).block(Block::default().borders(Borders::ALL));
        input.render(chunks[1], buf);
    }
}
