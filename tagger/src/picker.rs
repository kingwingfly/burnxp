use crate::{
    components::{Grid, Input, PickerFooter, Quit, Title},
    state::{CurrentScreen, PROCESS},
    terminal::AutoDropTerminal,
    utils::{centered_rect, images_walk, json_from, json_into, Items},
};
use anyhow::Result;
use clap::ValueEnum;
use crossterm::event::{self, Event as TermEvent, KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Widget, WidgetRef},
};
use std::{collections::HashSet, fs, path::PathBuf, sync::atomic::Ordering};

#[derive(Debug, Default)]
pub struct Picker {
    current_screen: CurrentScreen,
    chosen: [bool; 9], // flags
    items: Items<PathBuf, 9>,
    /// file ops method
    method: Method,
    /// target directory to move images to
    to: PathBuf,
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

impl Picker {
    pub fn new(method: Method, cache: PathBuf, from: PathBuf, to: PathBuf) -> Self {
        let images = images_walk(from);
        PROCESS.total.fetch_add(images.len(), Ordering::Relaxed);
        Self {
            method,
            to,
            items: Items::new(images),
            cache: json_from(&cache).unwrap_or_default(),
            cache_path: cache,
            ..Default::default()
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let mut terminal = AutoDropTerminal::new()?;
        loop {
            PROCESS
                .finished
                .store(9 * self.items.page(), Ordering::Relaxed);
            for (i, p) in self.items.current_items().iter().enumerate() {
                if self.cache.contains(p) {
                    self.chosen[i] = true;
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
                                let cur = self.items.current_items();
                                let i = c.to_digit(10).unwrap() as usize;
                                if i > 0 && i <= cur.len() {
                                    self.chosen[i - 1] = !self.chosen[i - 1];
                                    if self.chosen[i - 1] {
                                        self.cache.insert(cur[i - 1].clone());
                                    } else {
                                        self.cache.remove(&cur[i - 1]);
                                    }
                                }
                            }
                            KeyCode::Char('j') => self.current_screen = CurrentScreen::Popup(0),
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
                                self.chosen = [false; 9];
                                break 'l;
                            }
                        },
                        CurrentScreen::Popup(_) => match key.code {
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
                        CurrentScreen::Finished => match key.code {
                            KeyCode::Char('q') => self.current_screen = CurrentScreen::Exiting,
                            _ => continue,
                        },
                        CurrentScreen::Exiting => match key.code {
                            KeyCode::Char('y') => {
                                json_into(&self.cache_path, &self.cache)?;
                                let mut cache = self
                                    .cache
                                    .iter()
                                    .filter_map(|p| p.canonicalize().ok())
                                    .collect::<HashSet<_>>();
                                fs::create_dir_all(&self.to)?;
                                walkdir::WalkDir::new(&self.to)
                                    .into_iter()
                                    .filter_map(|res| res.ok())
                                    .filter_map(|e| {
                                        let path = e.into_path();
                                        if path.is_symlink()
                                            && path
                                                .canonicalize()
                                                .map_or(true, |p| !cache.remove(&p))
                                        {
                                            return Some(path);
                                        }
                                        None
                                    })
                                    .for_each(|path| {
                                        fs::remove_file(path).ok();
                                    });
                                for from in cache.iter() {
                                    let mut to = self.to.join(from.file_name().unwrap());
                                    let mut i = 0;
                                    while to.exists() {
                                        let name = from.file_name().unwrap();
                                        let new_name = format!("{}_{}", i, name.to_string_lossy());
                                        to = self.to.join(new_name);
                                        i += 1;
                                    }
                                    match self.method {
                                        Method::Cp => fs::copy(from, to).map(|_| {})?,
                                        Method::SoftLink => {
                                            #[cfg(unix)]
                                            std::os::unix::fs::symlink(from, to)?;
                                            #[cfg(windows)]
                                            std::os::windows::fs::symlink_file(from, to)?;
                                        }
                                        Method::HardLink => fs::hard_link(from, to)?,
                                        Method::Move => fs::rename(from, to)?,
                                    }
                                }
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

impl WidgetRef for Picker {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        if CurrentScreen::Exiting == self.current_screen {
            let area = centered_rect(60, 25, area);
            Quit.render(area, buf);
            return;
        }
        if let CurrentScreen::Popup(_) = self.current_screen {
            let area = centered_rect(60, 25, area);
            Input::new("Page to go", self.items.page()).render(area, buf);
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
        if CurrentScreen::Main == self.current_screen {
            Grid::<'_, 3, 3, 9>::new(
                self.items.current_items(),
                self.items.preload_items(),
                self.chosen,
            )
            .render(chunks[1], buf)
        } else {
            Title {
                title: "The picking finished.".to_string(),
            }
            .render(chunks[1], buf);
        }
        PickerFooter {
            current_screen: self.current_screen,
        }
        .render(chunks[2], buf);
    }
}
