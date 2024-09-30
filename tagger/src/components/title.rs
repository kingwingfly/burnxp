use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

pub(crate) struct Title {
    pub title: String,
}

impl Widget for Title {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default());
        let title = Paragraph::new(Text::styled(&self.title, Style::default().fg(Color::Green)))
            .block(title_block)
            .wrap(Wrap { trim: true });
        title.render(area, buf);
    }
}
