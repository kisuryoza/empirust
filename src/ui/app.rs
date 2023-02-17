use crate::{
    config::{self, Config},
    mpd::Mpd,
};
use std::time::Duration;
use tui::{
    layout::Constraint,
    style::{Color, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
};

#[derive(Debug)]
/// Holds data of the application's ui
///
/// * `show_popup`: is popup opened
/// * `tick_rate`: how often to update ui
/// * `tab_titles`: vector of tab's names
/// * `tab_index`: number of selected tab
/// * `queue`: holds list of songs to display
pub(crate) struct App<'app> {
    pub(crate) show_popup: bool,
    tick_rate: Duration,
    tab_titles: Vec<&'app str>,
    tab_index: usize,
    state: TableState,
    max_items: usize,
    header: Row<'app>,
    // TODO: extend the logic of calculating of columns's widths
    widths: Vec<Constraint>,
    // table: Table<'app>,
}

impl<'app> App<'app> {
    pub(crate) fn new(mpd: &Mpd, config: &Config) -> App<'app> {
        // setup state
        let pos: usize = match mpd.status().song {
            Some(arg) => arg.pos as usize,
            None => 0,
        };
        let mut state = TableState::default();
        state.select(Some(pos));

        // setup header
        let header_cells: Vec<Cell> = config
            .playlist_layout()
            .iter()
            .map(|layout| {
                let cell = match layout.0 {
                    config::PlaylistLayout::File => "File",
                    config::PlaylistLayout::Title => "Title",
                    config::PlaylistLayout::Duration => "Duration",
                    config::PlaylistLayout::Album => "Album",
                    config::PlaylistLayout::Artist => "Artist",
                    config::PlaylistLayout::Track => "Track",
                };
                Cell::from(cell).style(Style::default().fg(Color::Cyan))
            })
            .collect();
        let header = Row::new(header_cells)
            .style(config.styles().normal())
            .height(1)
            .bottom_margin(1);

        // setup widths
        let widths: Vec<Constraint> = config
            .playlist_layout()
            .iter()
            .map(|layout| Constraint::Percentage(layout.1))
            .collect();

        // let table = Table::new(vec![Row::default()]);
        let max_items = mpd.status().queue_len as usize;
        Self {
            show_popup: false,
            tick_rate: Duration::from_millis(250),
            tab_titles: vec!["Queue", "Browse"],
            tab_index: 0,
            state,
            max_items,
            header,
            widths,
            // table,
        }
    }

    pub(crate) fn switch(&mut self, mpd: &mut Mpd) {
        let selected = self.state.selected().unwrap() as u32;
        mpd.client_mut().switch(selected).unwrap();
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

    pub(crate) fn tick_rate(&self) -> Duration {
        self.tick_rate
    }

    // pub(crate) fn table_update(&'app mut self, mpd: &Mpd, config: &Config) {
    //     let table = self.gen_table(mpd, config);
    //     self.table = table;
    // }

    pub(crate) fn gen_table(&'app self, mpd: &Mpd, config: &Config) -> Table<'app> {
        // setup rows
        let songs = mpd.queue().unwrap();

        let style_playing = config.styles().playing();
        let style_normal = config.styles().normal();
        let curr_playing_pos = mpd.curr_playing_pos();

        let mut i = 0;
        let rows = songs.iter().map(|song| -> Row {
            let row = config.playlist_layout().iter().map(|layout| -> String {
                match layout.0 {
                    config::PlaylistLayout::File => song.file.to_owned(),
                    config::PlaylistLayout::Title => song.title.to_owned().unwrap_or_default(),
                    config::PlaylistLayout::Duration => song
                        .duration
                        .map(|item| {
                            crate::ui::human_formated_time(item.num_seconds().try_into().unwrap())
                        })
                        .unwrap_or_default(),
                    config::PlaylistLayout::Album => song
                        .tags
                        .get("Album")
                        .map(|item| item.to_owned())
                        .unwrap_or_default(),
                    config::PlaylistLayout::Artist => song
                        .tags
                        .get("Artist")
                        .map(|item| item.to_owned())
                        .unwrap_or_default(),
                    config::PlaylistLayout::Track => song
                        .tags
                        .get("Track")
                        .map(|item| item.to_owned())
                        .unwrap_or_default(),
                }
            });
            let style = if curr_playing_pos == i {
                style_playing
            } else {
                style_normal
            };
            i += 1;
            Row::new(row).style(style)
        });

        Table::new(rows)
            .header(self.header.clone())
            .block(Block::default().borders(Borders::TOP))
            .highlight_style(config.styles().selected())
            .widths(&self.widths)
    }

    /// Selecet next item in Queue
    pub(crate) fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.max_items - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    /// Selecet previous item in Queue
    pub(crate) fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.max_items - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub(crate) fn tab_titles(&self) -> &[&str] {
        self.tab_titles.as_ref()
    }

    pub(crate) fn tab_index(&self) -> usize {
        self.tab_index
    }

    pub(crate) fn state(&self) -> &TableState {
        &self.state
    }

    pub(crate) fn set_state(&mut self, state: TableState) {
        self.state = state;
    }
}
