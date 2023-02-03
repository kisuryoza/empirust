//! Manages input keys

use crate::mpd::Mpd;
use crate::ui::App;
use crossterm::event::{self, Event, KeyCode};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use tui::backend::Backend;
use tui::Terminal;

use crate::config::Config;

pub(crate) fn input<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    client: Mpd,
    config: Config,
) -> crossterm::Result<()> {
    let client = Arc::new(Mutex::new(client));

    // use seperate thread to update Mpd's data
    let client2 = Arc::clone(&client);
    let _handle = thread::spawn(move || loop {
        client2.lock().unwrap().update();
        thread::sleep(Duration::from_millis(200));
    });

    let mut last_tick = Instant::now();
    loop {
        // draw ui
        terminal.draw(|f| crate::ui::draw(f, &mut app, &config, &client.lock().unwrap()))?;

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
                if key.code == config.keys().quit() {
                    return Ok(());
                }
                if key.code == config.keys().switch_tab() {
                    app.tab_next();
                }
                if key.code == config.keys().toggle_pause() {
                    if let Err(e) = client.lock().unwrap().client_mut().toggle_pause() {
                        println!("{:?}", e);
                    }
                }
                if key.code == config.keys().vol_down() {
                    change_volume(&mut client.lock().unwrap(), -5)
                }
                if key.code == config.keys().vol_up() {
                    change_volume(&mut client.lock().unwrap(), 5)
                }
                if key.code == config.keys().queue_next() {
                    app.queue_next()
                }
                if key.code == config.keys().queue_prev() {
                    app.queue_previous()
                }
                if key.code == config.keys().switch_song() {
                    app.switch(&mut client.lock().unwrap());
                }
            }
        }
        if last_tick.elapsed() >= app.tick_rate() {
            // app.on_tick(&mut client);
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
