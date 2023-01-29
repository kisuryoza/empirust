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

#[derive(Debug)]
/// Holds MPD's data
pub(crate) struct Mpd {
    pub(crate) client: Client,
    pub(crate) status: mpd::Status,
    pub(crate) playlists: Option<Vec<mpd::Playlist>>,
    pub(crate) queue: Option<Vec<mpd::Song>>,
    pub(crate) prev_song: Option<mpd::Song>,
    pub(crate) curr_song: Option<mpd::Song>,
    pub(crate) curr_playing_pos: u32,
    pub(crate) curr_song_duration: u16,
}

impl Mpd {
    pub(crate) fn new(mut client: mpd::Client) -> Result<Self, Box<dyn Error>> {
        let status = client.status()?;
        let playlists = match client.playlists() {
            Ok(arg) => Some(arg),
            Err(_) => None,
        };
        let queue = match client.queue() {
            Ok(arg) => Some(arg),
            Err(_) => None,
        };
        let curr_song = match client.currentsong() {
            Ok(arg) => arg,
            Err(_) => None,
        };
        let curr_song_duration: u16 = match status.time {
            Some(time) => time.1.num_seconds().try_into().unwrap_or(1),
            None => 1,
        };

        Ok(Self {
            client,
            status: status.clone(),
            playlists,
            queue,
            prev_song: curr_song.clone(),
            curr_song,
            curr_playing_pos: status.song.unwrap().pos,
            curr_song_duration,
        })
    }

    pub(crate) fn update(&mut self) {
        let status = self.client.status().unwrap();
        let currentsong = match self.client.currentsong() {
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
        }
    }
}

// {{{ struct App
#[derive(Debug)]
/// Holds data of the application's ui
///
/// * `tick_rate`: how often to update ui
/// * `tab_titles`: vector of tab's names
/// * `tab_index`: number of selected tab
/// * `show_popup`: is popup opened
/// * `queue`: holds list of songs to display
pub(crate) struct App<'a> {
    pub(crate) tick_rate: Duration,
    tab_titles: Vec<Spans<'a>>,
    tab_index: usize,
    pub(crate) show_popup: bool,
    pub(crate) queue: Queue<'a>,
}

impl<'a> App<'a> {
    pub(crate) fn build(client: &Mpd, config: &config::Config) -> Result<Self, Box<dyn Error>> {
        let tab_titles = ["Queue", "Browse"];

        let tab_titles: Vec<Spans> = tab_titles
            .iter()
            .map(|t| Spans::from(Span::styled(*t, Style::default())))
            .collect();

        let queue = Queue::new(client, config);

        Ok(Self {
            tick_rate: Duration::from_millis(250),
            tab_titles,
            tab_index: 0,
            show_popup: false,
            queue,
        })
    }

    pub(crate) fn switch(&mut self, client: &mut Mpd) {
        let selected = self.queue.state.selected().unwrap() as u32;
        client.client.switch(selected).unwrap();
    }

    pub(crate) fn tab_next(&mut self) {
        self.tab_index = (self.tab_index + 1) % self.tab_titles.len();
    }

    // pub(crate) fn tab_previous(&mut self) {
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
///
/// * `state`: selected item
/// * `header`:
/// * `rows`: vector of strings of rows
/// * `widths`:
pub(crate) struct Queue<'a> {
    state: TableState,
    header: Row<'a>,
    rows: Vec<Vec<String>>,
    // TODO: extend the logic of calculating of columns's widths
    widths: Vec<Constraint>,
}

impl<'a> Queue<'a> {
    fn new(client: &Mpd, config: &config::Config) -> Self {
        // setup state
        let pos: usize = match client.status.song {
            Some(arg) => arg.pos as usize,
            None => 0,
        };
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
        if let Some(songs) = &client.queue {
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

    pub(crate) fn next(&mut self) {
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

    pub(crate) fn previous(&mut self) {
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
pub(crate) fn draw<B>(f: &mut Frame<B>, app: &mut App, config: &config::Config, client: &Mpd)
where
    B: Backend,
{
    let size = f.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)].as_ref())
        .split(size);

    let tabs = Tabs::new(app.tab_titles.clone())
        .select(app.tab_index)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(config.tab_selected_style());
    f.render_widget(tabs, chunks[0]);

    match app.tab_index {
        0 => draw_tab_one(f, app, chunks[1], config, client),
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
fn draw_tab_one<B>(
    f: &mut Frame<B>,
    app: &mut App,
    area: Rect,
    config: &config::Config,
    client: &Mpd,
) where
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

    draw_queue(f, app, chunks[0], config, client);
    draw_progressbar(f, app, chunks[2], config, client);
}

fn draw_queue<B>(f: &mut Frame<B>, app: &mut App, area: Rect, config: &config::Config, client: &Mpd)
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
        let style = if client.curr_playing_pos == i {
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

fn draw_progressbar<B>(
    f: &mut Frame<B>,
    _app: &mut App,
    area: Rect,
    config: &config::Config,
    client: &Mpd,
) where
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

    let title: (String, String) = match &client.curr_song {
        Some(song) => {
            let artist = song
                .tags
                .get("Artist")
                .unwrap_or(&String::new())
                .to_string();
            let title = song.title.clone().unwrap_or_default();
            (artist, title)
        }
        None => (String::new(), String::new()),
    };

    let title = Block::default()
        .title(Span::raw(format!("{} - {}", title.0, title.1)))
        .borders(Borders::TOP);
    f.render_widget(title, chunks[0]);

    let volume = client.status.volume;
    let status = Block::default().title(Span::styled(
        format!("Volume: {}%", volume),
        Style::default().fg(Color::Gray),
    ));
    f.render_widget(status, chunks[1]);

    let progress: (String, u16) = match client.status.time {
        Some(time) => {
            let elapsed = time.0.num_seconds() as u16;
            let duration = time.1.num_seconds() as u16;
            let label = format!(
                "{}/{}",
                human_formated_time(elapsed),
                human_formated_time(duration),
            );
            (label, (elapsed * 100 / duration))
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
    client: &Mpd,
) where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)].as_ref())
        .split(area);

    if let Some(items) = &client.playlists {
        let items: Vec<ListItem> = items
            .iter()
            .map(|i| ListItem::new(i.name.clone()).style(Style::default()))
            .collect();
        let items = List::new(items)
            .block(Block::default().borders(Borders::TOP))
            .highlight_style(Style::default());
        f.render_widget(items, chunks[0]);
    }
}
// }}}
