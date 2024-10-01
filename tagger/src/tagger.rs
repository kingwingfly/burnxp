use crate::{
    components::{Image, Input, Quit, TaggerFooter, Title},
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
use std::hash::Hash;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::{collections::HashMap, time::Duration};

type Name = String;
type Score = i64;

#[derive(Debug, Clone, PartialEq)]
struct Tag {
    name: Name,
    score: Score,
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
}

impl<T, const N: usize> From<&Cache<T>> for Items<Tag, N>
where
    T: Eq + Hash,
{
    fn from(value: &Cache<T>) -> Self {
        Items::new(value.tags.iter().map(Into::into).collect())
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

pub struct Tagger {
    current_screen: CurrentScreen,
    items: Items<PathBuf, 1>,
    tags: Items<Tag, 9>,
    chosen: [bool; 9],
    cache: Cache<PathBuf>,
    cache_path: PathBuf,
}

impl Tagger {
    pub fn new(root: PathBuf, cache_path: PathBuf) -> Self {
        let images = images_walk(&root);
        PROCESS.total.fetch_add(images.len(), Ordering::Relaxed);
        let items = Items::new(images);
        let cache: Cache<PathBuf> = json_from(&cache_path).unwrap_or_default();
        Self {
            current_screen: CurrentScreen::Main,
            items,
            tags: (&cache).into(),
            chosen: [false; 9],
            cache,
            cache_path,
        }
    }

    fn new_tag(&mut self, tag: Tag) {
        self.tags.push(tag.clone());
        self.cache.tags.insert(tag.name, tag.score);
    }

    fn delete_tag(&mut self, tag: &Tag) {
        self.tags.remove(tag);
        self.cache.tags.remove(&tag.name);
        for tags in self.cache.tagged.values_mut() {
            tags.retain(|x| x != &tag.name);
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let mut terminal = AutoDropTerminal::new()?;
        loop {
            PROCESS.finished.store(self.items.page(), Ordering::Relaxed);
            if let Some(tags) = self.cache.tagged.get(&self.items.current_items()[0]) {
                for (i, t) in self.tags.current_items().iter().enumerate() {
                    self.chosen[i] = tags.contains(&t.name);
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
                    if event::poll(Duration::from_millis(50))? {
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
                            KeyCode::Char('d') => self.current_screen = CurrentScreen::Popup(2), // delete tag
                            _ => {
                                match key.code {
                                    KeyCode::Enter | KeyCode::Right => {
                                        if self.items.inc_page() {
                                            self.current_screen = CurrentScreen::Finished;
                                            break;
                                        }
                                    }
                                    KeyCode::Left => self.items.dec_page(),
                                    KeyCode::Up => {
                                        self.tags.inc_page();
                                    }
                                    KeyCode::Down => self.tags.dec_page(),
                                    _ => continue,
                                }
                                self.chosen = [false; 9];
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
                                break 'l;
                            }
                            _ => continue,
                        },
                        // new tag
                        CurrentScreen::Popup(1) => match key.code {
                            KeyCode::Enter => {
                                self.current_screen = CurrentScreen::Main;
                                self.chosen = [false; 9];
                                break 'l;
                            }
                            _ => continue,
                        },
                        // delete tag
                        CurrentScreen::Popup(2) => match key.code {
                            KeyCode::Enter => {
                                self.current_screen = CurrentScreen::Main;
                                self.chosen = [false; 9];
                                break 'l;
                            }
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
                1 => Input::new("New tag name", self.tags.page()).render(area, buf),
                // delete tag
                2 => Input::new("", self.tags.page()).render(area, buf),
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
            Image::new(self.items.current_items()[0].clone(), chunks[1]).render(chunks[1], buf)
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
