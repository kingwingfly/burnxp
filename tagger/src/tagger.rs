use crate::{
    components::{preload, CheckBox, Image, Input, Quit, TaggerFooter, Title},
    state::{CurrentScreen, PROCESS},
    terminal::AutoDropTerminal,
    utils::{centered_rect, images_walk, json_from, json_into, Items},
};
use anyhow::Result;
use crossterm::event::{self, Event as TermEvent, KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Widget, WidgetRef},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::path::PathBuf;
use std::sync::atomic::Ordering;

type Name = String;
type Score = i64;

#[derive(Debug, Clone)]
struct Tag {
    name: Name,
    score: Score,
}

impl PartialEq for Tag {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Cache<T>
where
    T: Eq + Hash,
{
    tags: HashMap<Name, Score>,
    tagged: HashMap<T, Vec<Name>>,
}

impl<T> Cache<T>
where
    T: Eq + Hash + Clone,
{
    fn tag(&mut self, item: &T, tag: &Tag) {
        if let Some(x) = self.tagged.get_mut(item) {
            x.push(tag.name.clone());
            return;
        }
        self.tagged.insert(item.clone(), vec![tag.name.clone()]);
    }

    fn untag(&mut self, item: &T, tag: &Tag) {
        if let Some(x) = self.tagged.get_mut(item) {
            x.retain(|x| x != &tag.name);
        }
    }

    fn remove(&mut self, tag: &Name) {
        self.tags.remove(tag);
        for tags in self.tagged.values_mut() {
            tags.retain(|x| x != tag);
        }
    }

    fn score(&self) -> HashMap<i64, Vec<T>> {
        self.tagged.iter().fold(HashMap::new(), |mut acc, x| {
            let score: Score = x.1.iter().map(|tag| self.tags[tag]).sum();
            acc.entry(score).or_default().push(x.0.clone());
            acc
        })
    }
}

impl<T, const N: usize> From<&Cache<T>> for Items<Tag, N>
where
    T: Eq + Hash,
{
    /// Retrieve Items from the deserilized Cache
    fn from(value: &Cache<T>) -> Self {
        let mut tags: Vec<Tag> = value.tags.iter().map(Into::into).collect();
        tags.sort_by_key(|t| -t.score);
        Items::new(tags)
    }
}

impl From<(&Name, &Score)> for Tag {
    fn from(value: (&Name, &Score)) -> Self {
        Self {
            name: value.0.clone(),
            score: *value.1,
        }
    }
}

impl TryFrom<&[String; 2]> for Tag {
    type Error = ();

    fn try_from(value: &[String; 2]) -> std::result::Result<Self, Self::Error> {
        let score = value[1].parse().map_err(|_| ())?;
        Ok(Self {
            name: value[0].clone(),
            score,
        })
    }
}

impl TryFrom<&InputBuffer<2>> for Tag {
    type Error = ();

    fn try_from(value: &InputBuffer<2>) -> std::result::Result<Self, Self::Error> {
        (&value.values).try_into()
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}({})", self.name, self.score)
    }
}

struct InputBuffer<const N: usize> {
    cursor: usize,
    keys: [String; N],
    values: [String; N],
}

impl<const N: usize> InputBuffer<N> {
    fn new(keys: [String; N]) -> Self {
        let buffer = vec![String::new(); N];
        Self {
            cursor: 0,
            keys,
            values: buffer.try_into().unwrap(),
        }
    }

    fn push(&mut self, c: char) {
        self.values[self.cursor].push(c);
    }

    fn pop(&mut self) {
        self.values[self.cursor].pop();
    }

    fn clear(&mut self) {
        self.values[self.cursor].clear();
    }

    fn clear_all(&mut self) {
        for i in 0..N {
            self.values[i].clear();
        }
        self.cursor = 0;
    }

    fn next(&mut self) {
        self.cursor = (self.cursor + 1) % N;
    }

    fn prev(&mut self) {
        self.cursor = (self.cursor + N - 1) % N;
    }
}

impl<const N: usize> fmt::Display for InputBuffer<N> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..N {
            if i == self.cursor {
                writeln!(f, "> {:^7}: {}", self.keys[i], self.values[i])?;
            } else {
                writeln!(f, "  {:^7}: {}", self.keys[i], self.values[i])?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

pub struct Tagger {
    current_screen: CurrentScreen,
    items: Items<PathBuf, 1>,
    tags: Items<Tag, 9>,
    // flags for tags current page
    chosen: [bool; 9],
    // flags for tags next page
    chosen_n: [bool; 9],
    cache: Cache<PathBuf>,
    cache_path: PathBuf,
    input_buffer: InputBuffer<2>,
    // path of output
    output: PathBuf,
}

impl Tagger {
    pub fn new(root: PathBuf, cache_path: PathBuf, output: PathBuf) -> Self {
        let images = images_walk(&root);
        PROCESS.total.fetch_add(images.len(), Ordering::Relaxed);
        let items = Items::new(images);
        let cache: Cache<PathBuf> = json_from(&cache_path).unwrap_or_default();
        Self {
            current_screen: CurrentScreen::Main,
            items,
            tags: (&cache).into(),
            // flags for tags current page
            chosen: [false; 9],
            // flags for tags next page
            chosen_n: [false; 9],
            cache,
            cache_path,
            input_buffer: InputBuffer::new(["TagName".to_string(), "Score".to_string()]),
            output,
        }
    }

    fn new_tag(&mut self, tag: Tag) {
        self.tags.push(tag.clone());
        self.cache.tags.insert(tag.name, tag.score);
    }

    fn remove_tag(&mut self, tag: &Tag) {
        self.tags.remove(tag);
        self.cache.remove(&tag.name);
    }

    pub fn run(&mut self) -> Result<()> {
        let mut terminal = AutoDropTerminal::new()?;
        loop {
            PROCESS.finished.store(self.items.page(), Ordering::Relaxed);
            if let Some(tags) = self.cache.tagged.get(&self.items.current_items()[0]) {
                for (i, t) in self.tags.current_items().iter().enumerate() {
                    self.chosen[i] = tags.contains(&t.name);
                }
                for (i, t) in self.tags.preload_items().iter().enumerate() {
                    self.chosen_n[i] = tags.contains(&t.name);
                }
            }

            'l: loop {
                terminal.draw(|f| {
                    f.render_widget(&*self, f.area());
                })?;
                while let TermEvent::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Release {
                        // Skip events that are not KeyEventKind::Press
                        continue;
                    }
                    match self.current_screen {
                        CurrentScreen::Main => match key.code {
                            KeyCode::Char('q') => self.current_screen = CurrentScreen::Exiting,
                            KeyCode::Char(c) if c.is_numeric() => {
                                let tags = self.tags.current_items();
                                let item = &self.items.current_items()[0];
                                let i = c.to_digit(10).unwrap() as usize;
                                if i > 0 && i <= tags.len() {
                                    self.chosen[i - 1] = !self.chosen[i - 1];
                                    let tag = &tags[i - 1];
                                    if self.chosen[i - 1] {
                                        self.cache.tag(item, tag);
                                    } else {
                                        self.cache.untag(item, tag);
                                    }
                                }
                            }
                            KeyCode::Char('j') => self.current_screen = CurrentScreen::Popup(0), // page jump
                            KeyCode::Char('n') => self.current_screen = CurrentScreen::Popup(1), // new tag
                            _ => {
                                match key.code {
                                    KeyCode::Enter | KeyCode::Right => {
                                        if self.items.inc_page() {
                                            self.current_screen = CurrentScreen::Finished;
                                            break;
                                        }
                                    }
                                    KeyCode::Left => {
                                        self.items.dec_page();
                                    }
                                    KeyCode::Up => {
                                        if self.tags.dec_page() {
                                            self.tags.set_page(usize::MAX);
                                        }
                                    }
                                    KeyCode::Down => {
                                        if self.tags.inc_page() {
                                            self.tags.set_page(0);
                                        }
                                    }
                                    _ => continue,
                                }
                                self.chosen = [false; 9];
                                self.chosen_n = [false; 9];
                                break 'l;
                            }
                        },
                        // page jump
                        CurrentScreen::Popup(0) => match key.code {
                            KeyCode::Char(c) if c.is_numeric() => {
                                self.items.set_page(
                                    self.items.page() * 10 + c.to_digit(10).unwrap() as usize,
                                );
                            }
                            KeyCode::Backspace => self.items.set_page(self.items.page() / 10),
                            KeyCode::Enter => {
                                self.current_screen = CurrentScreen::Main;
                                self.chosen = [false; 9];
                                self.chosen_n = [false; 9];
                                break 'l;
                            }
                            _ => continue,
                        },
                        // modify tags
                        CurrentScreen::Popup(1) => match key.code {
                            KeyCode::Char(c) if !c.is_ascii_control() => self.input_buffer.push(c),
                            KeyCode::Tab => self.input_buffer.next(),
                            KeyCode::BackTab => self.input_buffer.prev(),
                            KeyCode::Backspace => self.input_buffer.pop(),
                            KeyCode::Esc => {
                                self.input_buffer.clear_all();
                                self.current_screen = CurrentScreen::Main;
                                break 'l;
                            }
                            KeyCode::Enter => match (&self.input_buffer).try_into() {
                                Ok(tag) => {
                                    self.new_tag(tag);
                                    self.input_buffer.clear_all();
                                    self.current_screen = CurrentScreen::Main;
                                    break 'l;
                                }
                                Err(_) if self.input_buffer.cursor == 0 => {
                                    self.input_buffer.cursor = 1;
                                    self.input_buffer.clear();
                                }
                                Err(_) => {
                                    let tag: Tag = (&self.input_buffer.values[0], &0).into();
                                    self.remove_tag(&tag);
                                    self.input_buffer.cursor = 0;
                                    self.input_buffer.clear_all();
                                    self.current_screen = CurrentScreen::Main;
                                    break 'l;
                                }
                            },
                            _ => continue,
                        },
                        CurrentScreen::Popup(_) => unreachable!(),
                        CurrentScreen::Finished => match key.code {
                            KeyCode::Char('q') => self.current_screen = CurrentScreen::Exiting,
                            _ => continue,
                        },
                        CurrentScreen::Exiting => match key.code {
                            KeyCode::Char('y') => {
                                json_into(&self.cache_path, &self.cache)?;
                                let output = self.cache.score();
                                json_into(&self.output, &output)?;
                                return Ok(());
                            }
                            KeyCode::Char('Y') => {
                                return Ok(());
                            }
                            _ => self.current_screen = CurrentScreen::Main,
                        },
                    }
                    break;
                }
            }
        }
    }
}

impl WidgetRef for Tagger {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        if CurrentScreen::Exiting == self.current_screen {
            let area = centered_rect(60, 25, area);
            Quit.render(area, buf);
            return;
        }
        if let CurrentScreen::Popup(mode) = self.current_screen {
            let area = centered_rect(60, 25, area);
            match mode {
                // page jump
                0 => Input::new("Page to go", self.items.page()).render(area, buf),
                // new tag
                1 => Input::new(
                    "Modify tag (Leave score empty to delete)",
                    &self.input_buffer,
                )
                .render(area, buf),
                _ => unreachable!(),
            }
            return;
        }
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(3),
            ])
            .split(area);
        Title {
            title: "Tagger".to_string(),
        }
        .render(chunks[0], buf);
        if CurrentScreen::Main == self.current_screen {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(70), Constraint::Fill(1)])
                .split(chunks[1]);
            Image::new(self.items.current_items()[0].clone(), chunks[0]).render(chunks[0], buf);
            preload(self.items.preload_items()[0].clone(), chunks[0]);
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Ratio(1, 2); 2])
                .split(chunks[1]);
            let b0 = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green));
            let b1 = Block::default().borders(Borders::ALL);
            CheckBox::new(self.tags.current_items(), &self.chosen).render(b1.inner(chunks[0]), buf);
            CheckBox::new(self.tags.preload_items(), &self.chosen_n)
                .render(b1.inner(chunks[1]), buf);
            b0.render(chunks[0], buf);
            b1.render(chunks[1], buf);
        } else {
            Title {
                title: "The tagging finished.".to_string(),
            }
            .render(chunks[1], buf);
        }
        TaggerFooter {
            current_screen: self.current_screen,
        }
        .render(chunks[2], buf);
    }
}
