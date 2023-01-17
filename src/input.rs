//! Manages input keys

use crossterm::event::{self, Event, KeyCode};
use mpd::Client;
use std::time::Duration;
use std::time::Instant;
use tui::backend::Backend;
use tui::Terminal;

use crate::config::Config;
use crate::ui;
use crate::ui::App;

pub fn input<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    mut client: Client,
    config: Config,
) -> crossterm::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui::draw(f, app, &config, &mut client))?;

        let timeout = app
            .tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('?') {
                    app.show_popup = !app.show_popup
                }
                if key.code == config.key_quit {
                    return Ok(());
                }
                if key.code == config.key_switch_tab {
                    app.tab.next();
                }
                if key.code == config.key_toggle_pause {
                    if let Err(e) = client.toggle_pause() {
                        println!("{:?}", e);
                    }
                }
                if key.code == config.key_vol_down {
                    change_volume(&app, &mut client, -5)
                }
                if key.code == config.key_vol_up {
                    change_volume(&app, &mut client, 5)
                }
                if key.code == config.key_queue_next {
                    app.queue.next()
                }
                if key.code == config.key_queue_prev {
                    app.queue.previous()
                }
                if key.code == config.key_switch_song {
                    app.queue.switch(&mut client)
                }
            }
        }
        if last_tick.elapsed() >= app.tick_rate {
            app.on_tick(&mut client);
            last_tick = Instant::now();
        }
    }
}

fn change_volume(app: &App, client: &mut Client, delta: i8) {
    let volume = app.status.volume;
    let changed = volume + delta;
    if changed >= 0 && changed <= 100 {
        if let Err(e) = client.volume(changed) {
            println!("{:?}", e);
        };
    }
}
