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
    pub keys: Vec<Vec<String>>,
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

        let mut keys: Vec<Vec<String>> = Vec::new();

        let key_quit = gen_key_and_desc(&mut keys, "q", "Quit");
        let key_switch_tab = gen_key_and_desc(&mut keys, "tab", "Switch tab");
        let key_toggle_pause = gen_key_and_desc(&mut keys, "p", "Toggle pause");
        let key_vol_down = gen_key_and_desc(&mut keys, "left", "Volume down");
        let key_vol_up = gen_key_and_desc(&mut keys, "right", "Volume up");
        let key_queue_next = gen_key_and_desc(&mut keys, "j", "Move next");
        let key_queue_prev = gen_key_and_desc(&mut keys, "k", "Move back");
        let key_switch_song = gen_key_and_desc(&mut keys, "enter", "Switch to song under cursor");

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
            key_quit,
            key_switch_tab,
            key_toggle_pause,
            key_vol_down,
            key_vol_up,
            key_queue_next,
            key_queue_prev,
            key_switch_song,
            keys,
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

fn gen_key_and_desc(keys: &mut Vec<Vec<String>>, key: &str, desc: &str) -> KeyCode {
    let cell = vec![key.to_string(), desc.to_string()];
    keys.push(cell);
    to_keycode(key)
}

fn to_keycode(key: &str) -> KeyCode {
    if key.len() == 1 {
        KeyCode::Char(key.chars().next().unwrap())
    } else {
        match key {
            "backspace" => KeyCode::Backspace,
            "enter" => KeyCode::Enter,
            "left" => KeyCode::Left,
            "right" => KeyCode::Right,
            "up" => KeyCode::Up,
            "down" => KeyCode::Down,
            "home" => KeyCode::Home,
            "end" => KeyCode::End,
            "pageup" => KeyCode::PageUp,
            "tab" => KeyCode::Tab,
            "backtab" => KeyCode::BackTab,
            "delete" => KeyCode::Delete,
            "insert" => KeyCode::Insert,
            "esc" => KeyCode::Esc,
            _ => KeyCode::Null,
        }
    }
}
