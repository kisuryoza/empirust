//! Parses config

use std::error::Error;

use crossterm::event::KeyCode;
use tui::style::{Color, Modifier, Style};

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) enum PlaylistLayout {
    File,
    Title,
    Duration,
    Album,
    Artist,
    Track,
}

#[derive(Debug)]
pub(crate) struct Config {
    styles: Styles,
    keys: Keys,
    playlist_layout: Vec<(PlaylistLayout, u16)>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            styles: Styles::default(),
            keys: Keys::default(),
            playlist_layout: vec![
                (PlaylistLayout::Artist, 20),
                (PlaylistLayout::Track, 5),
                (PlaylistLayout::Title, 30),
                (PlaylistLayout::Album, 30),
                (PlaylistLayout::Duration, 5),
            ],
        }
    }
}

impl Config {
    pub(crate) fn new() -> Result<Self, Box<dyn Error>> {
        let config: Config = Config::default();

        Ok(config)
    }

    pub(crate) fn styles(&self) -> &Styles {
        &self.styles
    }

    pub(crate) fn keys(&self) -> &Keys {
        &self.keys
    }

    pub(crate) fn playlist_layout(&self) -> &[(PlaylistLayout, u16)] {
        self.playlist_layout.as_ref()
    }
}

#[derive(Debug)]
pub(crate) struct Styles {
    tab_selected: Style,
    normal: Style,
    selected: Style,
    playing: Style,
    progress: Style,
}

impl Default for Styles {
    fn default() -> Self {
        Self {
            tab_selected: Style::default().fg(Color::Cyan).bg(Color::Reset),
            normal: Style::default().fg(Color::Reset).bg(Color::Reset),
            selected: Style::default().fg(Color::Black).bg(Color::Magenta),
            playing: Style::default().fg(Color::Cyan).bg(Color::Black),
            progress: Style::default()
                .bg(Color::Black)
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        }
    }
}

impl Styles {
    pub(crate) fn tab_selected(&self) -> Style {
        self.tab_selected
    }

    pub(crate) fn normal(&self) -> Style {
        self.normal
    }

    pub(crate) fn selected(&self) -> Style {
        self.selected
    }

    pub(crate) fn playing(&self) -> Style {
        self.playing
    }

    pub(crate) fn progress(&self) -> Style {
        self.progress
    }
}

#[derive(Debug)]
pub(crate) struct Keys {
    quit: KeyCode,
    switch_tab: KeyCode,
    toggle_pause: KeyCode,
    vol_down: KeyCode,
    vol_up: KeyCode,
    queue_next: KeyCode,
    queue_prev: KeyCode,
    switch_song: KeyCode,
    keys: Vec<Vec<String>>,
}

impl Default for Keys {
    fn default() -> Self {
        let mut keys: Vec<Vec<String>> = Vec::new();

        Self {
            quit: Keys::gen_key_and_desc(&mut keys, "q", "Quit"),
            switch_tab: Keys::gen_key_and_desc(&mut keys, "tab", "Switch tab"),
            toggle_pause: Keys::gen_key_and_desc(&mut keys, "p", "Toggle pause"),
            vol_down: Keys::gen_key_and_desc(&mut keys, "left", "Volume down"),
            vol_up: Keys::gen_key_and_desc(&mut keys, "right", "Volume up"),
            queue_next: Keys::gen_key_and_desc(&mut keys, "j", "Move next"),
            queue_prev: Keys::gen_key_and_desc(&mut keys, "k", "Move back"),
            switch_song: Keys::gen_key_and_desc(&mut keys, "enter", "Switch to song under cursor"),
            keys,
        }
    }
}

impl Keys {
    fn gen_key_and_desc(keys: &mut Vec<Vec<String>>, key: &str, desc: &str) -> KeyCode {
        let cell = vec![key.to_string(), desc.to_string()];
        keys.push(cell);
        Keys::to_keycode(key)
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

    pub(crate) fn quit(&self) -> KeyCode {
        self.quit
    }

    pub(crate) fn switch_tab(&self) -> KeyCode {
        self.switch_tab
    }

    pub(crate) fn toggle_pause(&self) -> KeyCode {
        self.toggle_pause
    }

    pub(crate) fn vol_down(&self) -> KeyCode {
        self.vol_down
    }

    pub(crate) fn vol_up(&self) -> KeyCode {
        self.vol_up
    }

    pub(crate) fn queue_next(&self) -> KeyCode {
        self.queue_next
    }

    pub(crate) fn queue_prev(&self) -> KeyCode {
        self.queue_prev
    }

    pub(crate) fn switch_song(&self) -> KeyCode {
        self.switch_song
    }

    pub(crate) fn keys(&self) -> &[Vec<String>] {
        self.keys.as_ref()
    }
}
