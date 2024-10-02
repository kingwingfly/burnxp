use crate::{
    components::{Grid, Input, Quit, TagGrid, TaggerFooter, Title},
    state::{CurrentScreen, PROCESS},
    terminal::AutoDropTerminal,
    utils::{centered_rect, images_walk, json_from, json_into, Items},
};
use anyhow::Result;
use crossterm::event::{self, Event as TermEvent, KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Widget, WidgetRef},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::path::PathBuf;
use std::sync::atomic::Ordering;

type Name = String;
type Score = i64;

#[derive(Debug, Clone, Eq)]
struct Tag {
    name: Name,
    score: Score,
}

impl PartialEq for Tag {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl PartialOrd for Tag {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Tag {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.score.cmp(&other.score) {
            std::cmp::Ordering::Equal => self.name.cmp(&other.name),
            res => res,
        }
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
        tags.sort_unstable();
        tags.reverse();
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
    items: Items<PathBuf, 4>,
    tags: Items<Tag, 9>,
    current_tag: Option<Tag>,
    // flags for tags current page
    chosen: [bool; 4],
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
            current_tag: None,
            // flags for tags current page
            chosen: [false; 4],
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
            PROCESS
                .finished
                .store(4 * self.items.page(), Ordering::Relaxed);
            if let Some(cur) = self.current_tag.as_ref() {
                for (i, item) in self.items.current_items().iter().enumerate() {
                    if let Some(tags) = self.cache.tagged.get(item) {
                        self.chosen[i] = tags.contains(&cur.name);
                    }
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
                                if let Some(cur) = self.current_tag.as_ref() {
                                    let item = self.items.current_items();
                                    let i = c.to_digit(10).unwrap() as usize;
                                    if i > 0 && i <= item.len() {
                                        self.chosen[i - 1] = !self.chosen[i - 1];
                                        if self.chosen[i - 1] {
                                            self.cache.tag(&item[i], cur);
                                        } else {
                                            self.cache.untag(&item[i], cur);
                                        }
                                    }
                                } else {
                                    self.current_screen = CurrentScreen::Popup(2);
                                }
                            }
                            KeyCode::Char('j') => self.current_screen = CurrentScreen::Popup(0), // page jump
                            KeyCode::Char('n') => self.current_screen = CurrentScreen::Popup(1), // new tag
                            KeyCode::Char('v') => self.current_screen = CurrentScreen::Popup(2), // view tags
                            _ => {
                                match key.code {
                                    KeyCode::Enter | KeyCode::Right | KeyCode::Down => {
                                        if self.items.inc_page() {
                                            self.current_screen = CurrentScreen::Finished;
                                            break;
                                        }
                                    }
                                    KeyCode::Left | KeyCode::Up => {
                                        self.items.dec_page();
                                    }
                                    _ => continue,
                                }
                                self.chosen = [false; 4];
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
                                self.chosen = [false; 4];
                                self.current_screen = CurrentScreen::Main;
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
                        // view tags
                        CurrentScreen::Popup(2) => match key.code {
                            KeyCode::Char(c) if c.is_numeric() => {
                                let tags = self.tags.current_items();
                                let i = c.to_digit(10).unwrap() as usize;
                                if i > 0 && i <= tags.len() {
                                    self.current_tag = Some(tags[i - 1].clone());
                                }
                                self.chosen = [false; 4];
                                self.current_screen = CurrentScreen::Main;
                                break 'l;
                            }
                            KeyCode::Up | KeyCode::Left => {
                                if self.tags.dec_page() {
                                    self.tags.set_page(usize::MAX);
                                }
                            }
                            KeyCode::Down | KeyCode::Right => {
                                if self.tags.inc_page() {
                                    self.tags.set_page(0);
                                }
                            }
                            KeyCode::Esc => self.current_screen = CurrentScreen::Main,
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
            let area = centered_rect(60, 30, area);
            match mode {
                // page jump
                0 => Input::new("Page to go", self.items.page()).render(area, buf),
                // new tag
                1 => Input::new(
                    "Modify tag (Leave score empty to delete)",
                    &self.input_buffer,
                )
                .render(area, buf),
                2 => TagGrid::<'_, Tag, 3, 3>::new(self.tags.current_items()).render(area, buf),
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
        {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Ratio(1, 2); 2])
                .split(chunks[0]);
            Title {
                title: "Tagger".to_string(),
            }
            .render(chunks[0], buf);
            if let Some(tag) = &self.current_tag {
                Title {
                    title: format!("Current Tag: {}", tag),
                }
                .render(chunks[1], buf);
            } else {
                Title {
                    title: "Press v to select tag".to_string(),
                }
                .render(chunks[1], buf);
            }
        }
        if CurrentScreen::Main == self.current_screen {
            Grid::<'_, 2, 2, 4>::new(
                self.items.current_items(),
                self.items.preload_items(),
                self.chosen,
            )
            .render(chunks[1], buf);
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
