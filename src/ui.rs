//! UI crate

use std::{error::Error, time::Duration};

use mpd::Client;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Gauge, List, ListItem, Row, Table, TableState, Tabs},
    Frame,
};

use crate::config;

// {{{ struct App
#[derive(Debug)]
/// Holds data of the application's status
pub struct App<'a> {
    pub tick_rate: Duration,
    pub show_popup: bool,
    pub tab: Tab<'a>,
    pub queue: Queue<'a>,
    pub status: mpd::Status,
    prev_song: Option<mpd::Song>,
    curr_song: Option<mpd::Song>,
    curr_playing_pos: u32,
    curr_song_duration: u16,
    curr_song_duration_formated: String,
}

impl<'a> App<'a> {
    pub fn build(
        client: &mut mpd::Client,
        config: &config::Config,
    ) -> Result<Self, Box<dyn Error>> {
        let status = client.status()?;
        let curr_song = match client.currentsong() {
            Ok(currentsong) => currentsong,
            Err(_) => None,
        };

        let curr_song_duration: u16 = match status.time {
            Some(time) => time.1.num_seconds().try_into().unwrap_or(1),
            None => 1,
        };

        Ok(Self {
            tick_rate: Duration::from_millis(250),
            show_popup: false,
            tab: Tab::new(),
            queue: Queue::new(client, config),
            status: status.clone(),
            prev_song: curr_song.clone(),
            curr_song,
            curr_playing_pos: status.song.unwrap().pos,
            curr_song_duration,
            curr_song_duration_formated: human_formated_time(curr_song_duration),
        })
    }

    pub fn on_tick(&mut self, client: &mut mpd::Client) {
        let status = client.status().unwrap();
        let currentsong = match client.currentsong() {
            Ok(currentsong) => currentsong,
            Err(_) => None,
        };

        self.status = status.clone();
        self.curr_song = currentsong;
        self.curr_playing_pos = status.song.unwrap().pos;

        // update data of the new song
        if self.curr_song != self.prev_song {
            self.prev_song = self.curr_song.clone();
            let curr_song_duration: u16 = match status.time {
                Some(time) => time.1.num_seconds().try_into().unwrap_or(0),
                None => 0,
            };
            self.curr_song_duration = curr_song_duration;
            self.curr_song_duration_formated = human_formated_time(curr_song_duration);
        }
    }
}
// }}}

// {{{ struct Tab
#[derive(Debug)]
pub struct Tab<'a> {
    titles: Vec<Spans<'a>>,
    index: usize,
}

impl<'a> Tab<'a> {
    fn new() -> Self {
        let title = ["Queue", "Browse"];

        let titles: Vec<Spans> = title
            .iter()
            .map(|t| Spans::from(Span::styled(*t, Style::default())))
            .collect();

        Self { titles, index: 0 }
    }

    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    // pub fn previous(&mut self) {
    //     if self.index > 0 {
    //         self.index -= 1;
    //     } else {
    //         self.index = self.titles.len() - 1;
    //     }
    // }
}
// }}}

// {{{ struct Queue
#[derive(Debug)]
/// Queue widget
pub struct Queue<'a> {
    state: TableState,
    header: Row<'a>,
    rows: Vec<Vec<String>>,
    // TODO: extend the logic of calculating of columns's widths
    widths: Vec<Constraint>,
}

impl<'a> Queue<'a> {
    fn new(client: &mut mpd::Client, config: &config::Config) -> Self {
        // setup state
        let pos = client.status().unwrap().song.unwrap().pos as usize;
        let mut state = TableState::default();
        state.select(Some(pos));

        // setup header
        let mut header_cells = Vec::new();
        for layout in config.playlist_layout() {
            let cell = match layout.0 {
                config::PlaylistLayout::File => "File",
                config::PlaylistLayout::Title => "Title",
                config::PlaylistLayout::Duration => "Duration",
                config::PlaylistLayout::Album => "Album",
                config::PlaylistLayout::Artist => "Artist",
                config::PlaylistLayout::Track => "Track",
            };
            header_cells.push(cell);
        }
        let header_cells = header_cells
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::Cyan)));
        let header = Row::new(header_cells)
            .style(config.normal_style())
            .height(1)
            .bottom_margin(1);

        // setup rows
        let mut rows: Vec<Vec<String>> = Vec::new();
        if let Ok(songs) = client.queue() {
            for song in songs {
                let mut cells = Vec::new();
                for layout in config.playlist_layout() {
                    let cell: String = match layout.0 {
                        config::PlaylistLayout::File => song.file.clone(),
                        config::PlaylistLayout::Title => match &song.title {
                            Some(arg) => arg.clone(),
                            None => String::new(),
                        },
                        config::PlaylistLayout::Duration => match song.duration {
                            Some(arg) => human_formated_time(arg.num_seconds().try_into().unwrap()),
                            None => String::new(),
                        },
                        config::PlaylistLayout::Album => match song.tags.get("Album") {
                            Some(arg) => arg.clone(),
                            None => String::new(),
                        },
                        config::PlaylistLayout::Artist => match song.tags.get("Artist") {
                            Some(arg) => arg.clone(),
                            None => String::new(),
                        },
                        config::PlaylistLayout::Track => match song.tags.get("Track") {
                            Some(arg) => arg.clone(),
                            None => String::new(),
                        },
                    };
                    cells.push(cell);
                }
                rows.push(cells)
            }
        }

        // setup widths
        let playlist_layout = config.playlist_layout();
        // let len = 100 / playlist_layout.len() as u16;
        let mut widths: Vec<Constraint> = Vec::new();
        let mut i = 0;
        while i < config.playlist_layout().len() {
            // widths.push(Constraint::Percentage(len));
            widths.push(Constraint::Percentage(playlist_layout.get(i).unwrap().1));
            i += 1
        }

        Self {
            state,
            header,
            rows,
            widths,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.rows.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.rows.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn switch(&mut self, client: &mut mpd::Client) {
        let selected = self.state.selected().unwrap() as u32;
        client.switch(selected).unwrap();
    }
}
// }}}

fn human_formated_time(time: u16) -> String {
    let min = time / 60;
    let sec = time % 60;
    if sec < 10 {
        format!("{}:0{}", min, sec)
    } else {
        format!("{}:{}", min, sec)
    }
}

/// Renders UI
pub fn draw<B>(f: &mut Frame<B>, app: &mut App, config: &config::Config, client: &mut Client)
where
    B: Backend,
{
    let size = f.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)].as_ref())
        .split(size);

    let tabs = Tabs::new(app.tab.titles.clone())
        .select(app.tab.index)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(config.tab_selected_style());
    f.render_widget(tabs, chunks[0]);

    match app.tab.index {
        0 => draw_tab_one(f, app, chunks[1], config),
        1 => draw_tab_two(f, app, chunks[1], config, client),
        _ => {}
    }

    if app.show_popup {
        let area = calculate_area_for_popup(40, 40, size);
        f.render_widget(tui::widgets::Clear, area); //this clears out the background

        let rows = config.keys.iter().map(|i| {
            let cells = i.iter().map(|c| Cell::from(&**c));
            Row::new(cells)
        });

        let table = Table::new(rows)
            .block(Block::default().title("Help").borders(Borders::ALL))
            .widths(&[Constraint::Percentage(20), Constraint::Percentage(80)]);

        f.render_widget(table, area);
    }
}

fn calculate_area_for_popup(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

// {{{ 1st tab
fn draw_tab_one<B>(f: &mut Frame<B>, app: &mut App, area: Rect, config: &config::Config)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Min(5),
                Constraint::Length(1),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(area);

    draw_queue(f, app, chunks[0], config);
    draw_progressbar(f, app, chunks[2], config);
}

fn draw_queue<B>(f: &mut Frame<B>, app: &mut App, area: Rect, config: &config::Config)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5)].as_ref())
        .split(area);

    ////////////////////////////////////////////////////////
    // FIXME: this block looks ugly
    let mut i = 0;
    let rows = app.queue.rows.iter().map(|item| -> Row {
        let cells = item.iter().map(|c| Cell::from(&**c));
        let style = if app.curr_playing_pos == i {
            config.playing_style()
        } else {
            config.normal_style()
        };
        i += 1;
        // let style = config.normal_style();
        Row::new(cells).style(style)
    });
    ////////////////////////////////////////////////////////

    let table = Table::new(rows)
        .header(app.queue.header.clone())
        .block(Block::default().borders(Borders::TOP))
        .highlight_style(config.selected_style())
        .widths(&app.queue.widths);
    f.render_stateful_widget(table, chunks[0], &mut app.queue.state);
}

fn draw_progressbar<B>(f: &mut Frame<B>, app: &mut App, area: Rect, config: &config::Config)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(area);

    let title: (String, String) = match &app.curr_song {
        Some(song) => {
            let artist = song
                .tags
                .get("Artist")
                .unwrap_or(&String::new())
                .to_string();
            let title = song.title.clone().unwrap_or(String::new());
            (artist, title)
        }
        None => (String::new(), String::new()),
    };

    let title = Block::default()
        .title(Span::raw(format!("{} - {}", title.0, title.1)))
        .borders(Borders::TOP);
    f.render_widget(title, chunks[0]);

    let volume = app.status.volume;
    let status = Block::default().title(Span::styled(
        format!("Volume: {}%", volume),
        Style::default().fg(Color::Gray),
    ));
    f.render_widget(status, chunks[1]);

    let progress: (String, u16) = match app.status.time {
        Some(time) => {
            let elapsed = time.0.num_seconds() as u16;
            let label = format!(
                "{}/{}",
                human_formated_time(elapsed),
                app.curr_song_duration_formated
            );
            (label, (elapsed * 100 / app.curr_song_duration))
        }
        None => (String::new(), 0),
    };
    let progress = Gauge::default()
        .gauge_style(config.progress_style())
        .label(progress.0)
        .percent(progress.1);
    f.render_widget(progress, chunks[2]);
}
// }}}

// {{{ 2st tab
fn draw_tab_two<B>(
    f: &mut Frame<B>,
    _app: &mut App,
    area: Rect,
    _config: &config::Config,
    client: &mut Client,
) where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)].as_ref())
        .split(area);

    let items: Vec<ListItem> = client
        .playlists()
        .unwrap()
        .iter()
        .map(|i| ListItem::new(i.name.clone()).style(Style::default()))
        .collect();
    let items = List::new(items)
        .block(Block::default().borders(Borders::TOP))
        .highlight_style(Style::default());
    f.render_widget(items, chunks[0]);

    // draw_queue(f, app, chunks[0], config);
}
// }}}
