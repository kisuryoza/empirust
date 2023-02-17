//! Manages input keys

use crate::{
    config::Config,
    mpd::Mpd,
    ui::{app::App, draw::draw},
};
use crossterm::event::{self, Event, KeyCode};
use std::time::{Duration, Instant};
use tui::{backend::Backend, Terminal};

pub(crate) fn input<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    mut client: Mpd,
    config: Config,
) -> crossterm::Result<()> {
    let mut last_tick = Instant::now();
    let quit = config.keys().quit();
    let switch_tab = config.keys().switch_tab();
    let toggle_pause = config.keys().toggle_pause();
    let vol_down = config.keys().vol_down();
    let vol_up = config.keys().vol_up();
    let queue_next = config.keys().queue_next();
    let queue_prev = config.keys().queue_prev();
    let switch_song = config.keys().switch_song();
    loop {
        // draw ui
        terminal.draw(|f| draw(f, &mut app, &config, &client))?;

        let timeout = app
            .tick_rate()
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        // catch input
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('?') {
                    app.show_popup = !app.show_popup
                }
                match key.code {
                    code if code == queue_next => app.next(),
                    code if code == queue_prev => app.previous(),
                    code if code == vol_down => change_volume(&mut client, -5),
                    code if code == vol_up => change_volume(&mut client, 5),
                    code if code == switch_tab => app.tab_next(),
                    code if code == toggle_pause => client.client_mut().toggle_pause().unwrap(),
                    code if code == switch_song => app.switch(&mut client),
                    code if code == quit => return Ok(()),
                    _ => (),
                }
            }
        }
        if last_tick.elapsed() >= app.tick_rate() {
            client.update();
            last_tick = Instant::now();
        }
    }
}

fn change_volume(client: &mut Mpd, delta: i8) {
    let volume = client.status().volume;
    let changed = volume + delta;
    if (0..=100).contains(&changed) {
        if let Err(e) = client.client_mut().volume(changed) {
            println!("{:?}", e);
        };
    }
}
