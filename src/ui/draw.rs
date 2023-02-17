//! UI crate

use crate::{
    config::Config,
    mpd::Mpd,
    ui::{app::App, human_formated_time},
};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Gauge, List, ListItem, Row, Table, Tabs},
    Frame,
};

/// Renders UI
pub(crate) fn draw<B>(f: &mut Frame<B>, app: &mut App, config: &Config, mpd: &Mpd)
where
    B: Backend,
{
    let size = f.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)].as_ref())
        .split(size);

    let tab_titles = app
        .tab_titles()
        .iter()
        .map(|t| Spans::from(Span::styled(*t, Style::default())))
        .collect();
    let tabs = Tabs::new(tab_titles)
        .select(app.tab_index())
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(config.styles().tab_selected());
    f.render_widget(tabs, chunks[0]);

    match app.tab_index() {
        0 => draw_tab_one(f, app, chunks[1], config, mpd),
        1 => draw_tab_two(f, app, chunks[1], config, mpd),
        _ => {}
    }

    if app.show_popup {
        let area = calculate_area_for_popup(40, 40, size);
        f.render_widget(tui::widgets::Clear, area); //this clears out the background

        let rows = config.keys().keys().iter().map(|row| {
            let cells = row.iter().map(|cell| Cell::from(&**cell));
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
fn draw_tab_one<B>(f: &mut Frame<B>, app: &mut App, area: Rect, config: &Config, mpd: &Mpd)
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

    draw_queue(f, app, chunks[0], config, mpd);
    draw_progressbar(f, app, chunks[2], config, mpd);
}

// FIXME: this func is very slow and is CPU eater
fn draw_queue<B>(f: &mut Frame<B>, app: &mut App, area: Rect, config: &Config, mpd: &Mpd)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5)].as_ref())
        .split(area);

    let table = app.gen_table(mpd, config);
    // let table = app.table.clone();
    let mut state = app.state().clone();
    f.render_stateful_widget(table, chunks[0], &mut state);
    app.set_state(state);
}

fn draw_progressbar<B>(f: &mut Frame<B>, _app: &mut App, area: Rect, config: &Config, mpd: &Mpd)
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

    let label: (String, String) = match &mpd.curr_song() {
        Some(song) => {
            let artist = song
                .tags
                .get("Artist")
                .unwrap_or(&String::new())
                .to_string();
            let title = match &song.title {
                Some(arg) => arg.clone(),
                None => String::new(),
            };
            (artist, title)
        }
        None => (String::new(), String::new()),
    };

    let label = Block::default()
        .title(Span::raw(format!("{} - {}", label.0, label.1)))
        .borders(Borders::TOP);
    f.render_widget(label, chunks[0]);

    let volume = mpd.status().volume;
    let status = Block::default().title(Span::styled(
        format!("Volume: {}%", volume),
        Style::default().fg(Color::Gray),
    ));
    f.render_widget(status, chunks[1]);

    let progress: (String, u16) = match mpd.status().time {
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
        .gauge_style(config.styles().progress())
        .label(progress.0)
        .percent(progress.1);
    f.render_widget(progress, chunks[2]);
}
// }}}

// {{{ 2st tab
fn draw_tab_two<B>(f: &mut Frame<B>, _app: &mut App, area: Rect, _config: &Config, mpd: &Mpd)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)].as_ref())
        .split(area);

    if let Some(items) = mpd.playlists() {
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
