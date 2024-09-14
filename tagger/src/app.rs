use crate::components::{Footer, Images, Quit, Render, Title};
use crate::event::{ComparePair, Event, CMPDISPATCH};
use crate::matrix::Matrix;
use crate::sort::{CompareResult, OrdPaths};
use crate::state::{CurrentScreen, PROCESS};
use crate::terminal::AutoDropTerminal;
use crate::utils::{bincode_from, bincode_into, centered_rect, json_into};
use anyhow::Result;
use crossterm::event::{self, Event as TermEvent, KeyCode, KeyEventKind};
use mime_guess::MimeGuess;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::Frame;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::atomic::Ordering as AtomicOrdering;
use std::thread;
use std::time::Duration;

pub struct App {
    current_screen: CurrentScreen,
    cmp: Option<ComparePair>,
    cache: Matrix,
    cache_path: PathBuf,
    images: Vec<OrdPaths>,
}

impl App {
    pub fn new(root: PathBuf, output: PathBuf, cache: PathBuf) -> Self {
        let images = walkdir::WalkDir::new(root)
            .into_iter()
            .filter_map(|res| res.ok())
            .filter_map(|e| match MimeGuess::from_path(e.path()).first() {
                Some(mime) if mime.type_() == "image" => {
                    Some(OrdPaths::new([e.path().to_path_buf()]))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        PROCESS.total.store(images.len(), AtomicOrdering::Relaxed);
        let images_c = images.clone();
        thread::spawn(move || -> Result<()> {
            let mut btree: BTreeSet<OrdPaths> = BTreeSet::new();
            for paths in images_c.into_iter() {
                btree.insert(paths);
                PROCESS.finished.fetch_add(1, AtomicOrdering::Relaxed);
            }
            CMPDISPATCH.req_tx.send(Event::Finished)?;
            let res: Vec<(i64, OrdPaths)> = btree.into_iter().fold(vec![], |mut acc, node| {
                if acc.is_empty() {
                    vec![(0, node)]
                } else {
                    let last = acc.last().unwrap();
                    CMPDISPATCH
                        .req_tx
                        .send(Event::Compare([node.clone(), last.1.clone()]))
                        .unwrap();
                    let delta = CMPDISPATCH.resp_rx.recv().unwrap() as i64;
                    acc.push((last.0 + delta, node));
                    acc
                }
            });
            json_into(&output, &res)?;
            CMPDISPATCH.req_tx.send(Event::Finished)?;
            Ok(())
        });
        Self {
            current_screen: CurrentScreen::Sort,
            cmp: None,
            cache: bincode_from(&cache).unwrap_or_default(),
            cache_path: cache,
            images,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let mut terminal = AutoDropTerminal::new()?;
        // recv the first compare
        self.recv_event()?;
        'a: loop {
            terminal.draw(|f| {
                self.render(f, f.area()).ok();
            })?;
            while let TermEvent::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Release {
                    // Skip events that are not KeyEventKind::Press
                    continue;
                }
                if event::poll(Duration::from_millis(100))? {
                    continue;
                }
                match self.current_screen {
                    CurrentScreen::Sort => match key.code {
                        KeyCode::Char('q') => {
                            self.current_screen = CurrentScreen::Exiting;
                        }
                        _ if self.cmp.is_some() => {
                            match key.code {
                                KeyCode::Up => self.resp_event(CompareResult::MuchBetter)?,
                                KeyCode::Left => self.resp_event(CompareResult::Better)?,
                                KeyCode::Right => self.resp_event(CompareResult::Worse)?,
                                KeyCode::Down => self.resp_event(CompareResult::MuchWorse)?,
                                KeyCode::Char('=') | KeyCode::Enter => {
                                    self.resp_event(CompareResult::Same)?
                                }
                                _ => continue,
                            }
                            self.recv_event()?;
                        }
                        _ => continue,
                    },
                    CurrentScreen::Finished => match key.code {
                        KeyCode::Char('q') => self.current_screen = CurrentScreen::Exiting,
                        _ => continue,
                    },
                    CurrentScreen::Exiting => match key.code {
                        KeyCode::Char('y') => {
                            bincode_into(&self.cache_path, &self.cache)?;
                            break 'a Ok(());
                        }
                        KeyCode::Char('Y') => {
                            break 'a Ok(());
                        }
                        _ => {
                            self.current_screen = match self.cmp {
                                Some(_) => CurrentScreen::Sort,
                                None => CurrentScreen::Finished,
                            }
                        }
                    },
                }
                break;
            }
        }
    }

    fn recv_event(&mut self) -> Result<()> {
        loop {
            match CMPDISPATCH.req_rx.recv().unwrap() {
                Event::Compare([k1, k2]) => {
                    if let Some(ord) = self.cache.get(&k1, &k2) {
                        CMPDISPATCH.resp_tx.send(ord)?;
                        continue;
                    }
                    self.cmp = Some([k1, k2]);
                    break;
                }
                Event::Finished => {
                    while let Event::Compare([k1, k2]) = CMPDISPATCH.req_rx.recv().unwrap() {
                        CMPDISPATCH
                            .resp_tx
                            .send(self.cache.get(&k1, &k2).unwrap())?;
                    }
                    self.cmp = None;
                    self.current_screen = CurrentScreen::Finished;
                    break;
                }
            }
        }
        Ok(())
    }

    fn resp_event(&mut self, ord: CompareResult) -> Result<()> {
        let [k1, k2] = self.cmp.take().unwrap();
        self.cache.insert(&k1, &k2, ord);
        if ord == CompareResult::Same {
            // If they are the same, k1 will be replaced with k2 in BTree.
            // Extending ensures data is not lost.
            k2.extend(&k1)
        }
        CMPDISPATCH.resp_tx.send(ord)?;
        Ok(())
    }
}

impl Render for App {
    fn render(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        if CurrentScreen::Exiting == self.current_screen {
            let area = centered_rect(60, 25, f.area());
            Quit.render(f, area)?;
            return Ok(());
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
        .render(f, chunks[0])?;
        match self.cmp {
            Some([ref p1, ref p2]) => {
                Images::new(&[p1.random_one().unwrap(), &p2[0]])?.render(f, chunks[1])?;
            }
            None => {
                Title {
                    title: "The comparation finished.".to_string(),
                }
                .render(f, chunks[1])?;
            }
        }
        Footer {
            current_screen: self.current_screen,
        }
        .render(f, chunks[2])?;
        Ok(())
    }
}

impl Drop for App {
    fn drop(&mut self) {
        for paths in self.images.iter_mut() {
            paths.drop();
        }
    }
}
