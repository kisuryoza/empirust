pub mod app;
pub mod draw;

/// Formats time (in seconds) to human readable {min:sec}
///
/// * `time`: in seconds
pub(crate) fn human_formated_time(time: u16) -> String {
    let min = time / 60;
    let sec = time % 60;
    if sec < 10 {
        format!("{}:0{}", min, sec)
    } else {
        format!("{}:{}", min, sec)
    }
}
