use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{Bar, BarChart, BarGroup, Widget},
};
use std::fmt::Display;

pub(crate) struct Histogram<T: Display> {
    pub data: Vec<(T, usize)>,
}

impl<T: Display> Widget for Histogram<T> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [title, histogram] = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)])
            .spacing(1)
            .areas(area);

        "Histogram".bold().into_centered_line().render(title, buf);

        let height = area.height as usize;
        let col = self.data.len() / height + 1;
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Ratio(1, col as u32); col])
            .split(histogram);

        for (i, data) in self.data.chunks(height).enumerate() {
            let data = data
                .iter()
                .map(|(tag, num)| (tag.to_string(), *num))
                .collect::<Vec<_>>();
            barchart(&data).render(chunks[i], buf);
        }
    }
}

/// Create a vertical bar chart from the temperatures data.
fn barchart(data: &[(String, usize)]) -> BarChart {
    let bars: Vec<Bar> = data
        .iter()
        .map(|(score, num)| bar(score.clone(), num))
        .collect();
    BarChart::default()
        .data(BarGroup::default().bars(&bars))
        .bar_gap(0)
        .bar_width(1)
        .direction(Direction::Horizontal)
}

fn bar(score: String, num: &usize) -> Bar {
    let style = Style::default();
    Bar::default()
        .value(*num as u64)
        .label(Line::from(score))
        .text_value(format!("{num}"))
        .style(style)
        .value_style(style.reversed())
}
