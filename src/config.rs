//! Parses config

use std::error::Error;

use crossterm::event::KeyCode;
use tui::style::{Color, Modifier, Style};

#[allow(dead_code)]
#[derive(Debug)]
pub enum PlaylistLayout {
    File,
    Title,
    Duration,
    Album,
    Artist,
    Track,
}

#[derive(Debug)]
pub struct Config {
    playlist_layout: Vec<(PlaylistLayout, u16)>,
    tab_selected_style: Style,
    normal_style: Style,
    selected_style: Style,
    playing_style: Style,
    progress_style: Style,
    pub key_quit: KeyCode,
    pub key_switch_tab: KeyCode,
    pub key_toggle_pause: KeyCode,
    pub key_vol_down: KeyCode,
    pub key_vol_up: KeyCode,
    pub key_queue_next: KeyCode,
    pub key_queue_prev: KeyCode,
    pub key_switch_song: KeyCode,
}

impl Default for Config {
    fn default() -> Self {
        let normal_style = Style::default().fg(Color::Reset).bg(Color::Reset);
        let tab_selected_style = Style::default().fg(Color::Cyan).bg(Color::Reset);
        let selected_style = Style::default().fg(Color::Black).bg(Color::Magenta);
        let playing_style = Style::default().fg(Color::Cyan).bg(Color::Black);
        let progress_style = Style::default()
            .bg(Color::Black)
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD);

        Self {
            playlist_layout: vec![
                (PlaylistLayout::Artist, 20),
                (PlaylistLayout::Track, 5),
                (PlaylistLayout::Title, 30),
                (PlaylistLayout::Album, 30),
                (PlaylistLayout::Duration, 5),
            ],
            tab_selected_style,
            normal_style,
            selected_style,
            playing_style,
            progress_style,
            key_quit: KeyCode::Char('q'),
            key_switch_tab: KeyCode::Tab,
            key_toggle_pause: KeyCode::Char('p'),
            key_vol_down: KeyCode::Left,
            key_vol_up: KeyCode::Right,
            key_queue_next: KeyCode::Char('j'),
            key_queue_prev: KeyCode::Char('k'),
            key_switch_song: KeyCode::Enter,
        }
    }
}

impl Config {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let config: Config = Config::default();

        Ok(config)
    }

    pub fn tab_selected_style(&self) -> Style {
        self.tab_selected_style
    }

    pub fn playlist_layout(&self) -> &[(PlaylistLayout, u16)] {
        self.playlist_layout.as_ref()
    }

    pub fn normal_style(&self) -> Style {
        self.normal_style
    }

    pub fn selected_style(&self) -> Style {
        self.selected_style
    }

    pub fn playing_style(&self) -> Style {
        self.playing_style
    }

    pub fn progress_style(&self) -> Style {
        self.progress_style
    }
}
