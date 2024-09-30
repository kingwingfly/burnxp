use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{Bar, BarChart, BarGroup, Widget},
};

pub(crate) struct Histogram {
    pub data: Vec<(i64, usize)>,
}

impl Widget for Histogram {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [title, histogram] = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)])
            .spacing(1)
            .areas(area);

        "Histogram".bold().into_centered_line().render(title, buf);

        let height = histogram.height as usize;
        if height == 0 {
            return;
        }
        let group_size = self.data.len() / height + 1;
        let mut data = vec![0; height];
        for (score, num) in self.data.iter() {
            data[(*score as usize / group_size).min(height - 1)] += num
        }
        let mut data = data
            .into_iter()
            .enumerate()
            .map(|(i, num)| {
                (
                    if group_size > 1 {
                        format!("{}-{}", i * group_size, (i + 1) * group_size - 1)
                    } else {
                        format!("{}", i)
                    },
                    num,
                )
            })
            .collect::<Vec<_>>();
        while let Some((_, 0)) = data.last() {
            data.pop();
        }
        barchart(&data).render(histogram, buf);
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
