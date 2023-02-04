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
                match key.code {
                    code if code == queue_next => app.queue_next(),
                    code if code == queue_prev => app.queue_previous(),
                    code if code == vol_down => change_volume(&mut client.lock().unwrap(), -5),
                    code if code == vol_up => change_volume(&mut client.lock().unwrap(), 5),
                    code if code == switch_tab => app.tab_next(),
                    code if code == toggle_pause => {
                        client.lock().unwrap().client_mut().toggle_pause().unwrap()
                    }
                    code if code == switch_song => app.switch(&mut client.lock().unwrap()),
                    code if code == quit => return Ok(()),
                    _ => (),
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
