use crate::{
    components::{Grid, NumInput, PickerFooter, Quit, Title},
    state::{CurrentScreen, PICKER_PROCESS},
    terminal::AutoDropTerminal,
    utils::{centered_rect, json_from, json_into},
};
use anyhow::Result;
use clap::ValueEnum;
use crossterm::event::{self, Event as TermEvent, KeyCode, KeyEventKind};
use mime_guess::MimeGuess;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Widget, WidgetRef},
};
use std::{
    collections::HashSet, fs, path::PathBuf, sync::atomic::Ordering as AtomicOrdering,
    time::Duration,
};

#[derive(Debug, Default)]
pub struct Picker<'a> {
    current_screen: CurrentScreen,
    buffer: &'a [PathBuf],
    chosen: [bool; 9],
    page: usize,
    method: Method,
    to: PathBuf,
    images: Vec<PathBuf>,
    /// cache those picked
    cache: HashSet<PathBuf>,
    cache_path: PathBuf,
}

#[derive(Debug, Clone, Default, ValueEnum)]
pub enum Method {
    Cp,
    #[default]
    SoftLink,
    HardLink,
    Move,
}

impl<'a> Picker<'a> {
    pub fn new(method: Method, cache: PathBuf, from: PathBuf, to: PathBuf) -> Self {
        let images = walkdir::WalkDir::new(from)
            .into_iter()
            .filter_map(|res| res.ok())
            .filter_map(|e| match MimeGuess::from_path(e.path()).first() {
                Some(mime) if mime.type_() == "image" => e.into_path().canonicalize().ok(),
                _ => None,
            })
            .collect::<Vec<_>>();
        PICKER_PROCESS
            .total
            .store(images.len(), AtomicOrdering::Relaxed);

        Self {
            method,
            to,
            cache: json_from(&cache).unwrap_or_default(),
            cache_path: cache,
            images,
            ..Default::default()
        }
    }

    pub fn run(&'a mut self) -> Result<()> {
        let mut terminal = AutoDropTerminal::new()?;
        loop {
            PICKER_PROCESS
                .finished
                .store(9 * self.page, AtomicOrdering::Relaxed);
            if 9 * self.page + 1 > self.images.len() {
                self.current_screen = CurrentScreen::Finished;
            } else {
                let l = self.page * 9;
                let r = (l + 18).min(self.images.len());
                self.buffer = &self.images[l..r];
                for (i, p) in self.buffer[..9.min(self.buffer.len())].iter().enumerate() {
                    if self.cache.contains(p) {
                        self.chosen[i] = true;
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
                    if event::poll(Duration::from_millis(50))? {
                        continue;
                    }
                    match self.current_screen {
                        CurrentScreen::Main => match key.code {
                            KeyCode::Char('q') => {
                                self.current_screen = CurrentScreen::Exiting;
                            }
                            _ if !self.buffer.is_empty() => match key.code {
                                KeyCode::Char(c) if c.is_numeric() => {
                                    let i = c.to_digit(10).unwrap() as usize;
                                    if i > 0 && i <= self.buffer.len().min(9) {
                                        self.chosen[i - 1] = !self.chosen[i - 1];
                                        if self.chosen[i - 1] {
                                            self.cache.insert(self.buffer[i - 1].clone());
                                        } else {
                                            self.cache.remove(&self.buffer[i - 1]);
                                        }
                                    }
                                }
                                KeyCode::Char('j') => self.buffer = &[], // empty means jump page
                                _ => {
                                    match key.code {
                                        KeyCode::Enter | KeyCode::Right => self.page += 1,
                                        KeyCode::Left => self.page = self.page.saturating_sub(1),
                                        _ => continue,
                                    }
                                    self.chosen = [false; 9];
                                    self.buffer = &[];
                                    break 'l;
                                }
                            },
                            KeyCode::Char(c) if c.is_numeric() => {
                                self.page = self.page * 10 + c.to_digit(10).unwrap() as usize;
                            }
                            KeyCode::Backspace => self.page /= 10,
                            KeyCode::Enter => {
                                self.page =
                                    self.page.clamp(0, self.images.len().saturating_sub(1) / 9);
                                self.current_screen = CurrentScreen::Main;
                                self.chosen = [false; 9];
                                break 'l;
                            }
                            _ => continue,
                        },
                        CurrentScreen::Finished => match key.code {
                            KeyCode::Char('q') => self.current_screen = CurrentScreen::Exiting,
                            _ => continue,
                        },
                        CurrentScreen::Exiting => match key.code {
                            KeyCode::Char('y') => {
                                json_into(&self.cache_path, &self.cache)?;
                                fs::create_dir_all(&self.to)?;
                                walkdir::WalkDir::new(&self.to)
                                    .into_iter()
                                    .filter_map(|res| res.ok())
                                    .filter_map(|e| {
                                        let path = e.into_path();
                                        if path.is_symlink()
                                            && !self.cache.remove(&path.canonicalize().ok()?)
                                        {
                                            return Some(path);
                                        }
                                        None
                                    })
                                    .for_each(|path| {
                                        fs::remove_file(path).ok();
                                    });
                                for from in self.cache.iter() {
                                    let mut to = self.to.join(from.file_name().unwrap());
                                    let mut i = 0;
                                    while to.exists() {
                                        let name = from.file_name().unwrap();
                                        let new_name = format!("{}_{}", i, name.to_string_lossy());
                                        to = self.to.join(new_name);
                                        i += 1;
                                    }
                                    if let Ok(from) = from.canonicalize() {
                                        match self.method {
                                            Method::Cp => fs::copy(from, to).map(|_| {})?,
                                            Method::SoftLink => {
                                                #[cfg(target_family = "unix")]
                                                std::os::unix::fs::symlink(from, to)?;
                                                #[cfg(target_family = "windows")]
                                                std::os::windows::fs::symlink_file(from, to)?;
                                            }
                                            Method::HardLink => fs::hard_link(from, to)?,
                                            Method::Move => fs::rename(from, to)?,
                                        }
                                    }
                                }
                                return Ok(());
                            }
                            KeyCode::Char('Y') => {
                                return Ok(());
                            }
                            _ => {
                                self.current_screen = match self.buffer.is_empty() {
                                    false => CurrentScreen::Main,
                                    true => CurrentScreen::Finished,
                                }
                            }
                        },
                    }
                    break;
                }
            }
        }
    }
}

impl WidgetRef for Picker<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        if CurrentScreen::Exiting == self.current_screen {
            let area = centered_rect(60, 25, area);
            Quit.render(area, buf);
            return;
        }
        if CurrentScreen::Main == self.current_screen && self.buffer.is_empty() {
            let area = centered_rect(60, 25, area);
            NumInput { num: self.page }.render(area, buf);
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
            title: "Picker".to_string(),
        }
        .render(chunks[0], buf);
        match self.buffer.is_empty() {
            false => Grid::new(self.buffer, self.chosen).render(chunks[1], buf),
            true => {
                Title {
                    title: "The picking finished.".to_string(),
                }
                .render(chunks[1], buf);
            }
        }
        PickerFooter {
            current_screen: self.current_screen,
        }
        .render(chunks[2], buf);
    }
}
