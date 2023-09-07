//! MPD data holder

use mpd::{Client, Playlist, Song, Status};
use std::error::Error;

#[derive(Debug)]
/// Holds MPD's data
pub struct Mpd {
    client: Client,
    status: Status,
    playlists: Option<Vec<Playlist>>,
    queue: Option<Vec<Song>>,
    curr_song: Option<Song>,
    prev_playing_pos: u32,
    curr_playing_pos: u32,
    curr_song_duration: u16,
}

impl Mpd {
    pub(crate) fn new(mut client: Client) -> Result<Self, Box<dyn Error>> {
        let status = client.status()?;
        let playlists = client.playlists().ok();
        let queue = client.queue().ok();
        let curr_song = client.currentsong().map_or(None, |arg| arg);
        let curr_playing_pos = status.song.map_or(0, |arg| arg.pos);
        let curr_song_duration: u16 = status
            .time
            .map_or(1, |time| time.1.num_seconds().try_into().unwrap_or(1));

        Ok(Self {
            client,
            status,
            playlists,
            queue,
            curr_song,
            prev_playing_pos: curr_playing_pos,
            curr_playing_pos,
            curr_song_duration,
        })
    }

    pub(crate) fn update(&mut self) {
        self.status = self.client.status().unwrap();
        self.curr_song = self.client.currentsong().map_or(None, |arg| arg);
        self.curr_playing_pos = self.status.song.map_or(0, |arg| arg.pos);

        // update data of the new song
        if self.curr_playing_pos != self.prev_playing_pos {
            self.prev_playing_pos = self.curr_playing_pos;
            self.curr_song_duration = self
                .status
                .time
                .map_or(0, |time| time.1.num_seconds().try_into().unwrap_or(0));
        }
    }

    pub(crate) fn client_mut(&mut self) -> &mut Client {
        &mut self.client
    }

    pub(crate) const fn status(&self) -> &Status {
        &self.status
    }

    pub(crate) const fn playlists(&self) -> Option<&Vec<Playlist>> {
        self.playlists.as_ref()
    }

    pub(crate) const fn queue(&self) -> Option<&Vec<Song>> {
        self.queue.as_ref()
    }

    pub(crate) const fn curr_song(&self) -> Option<&Song> {
        self.curr_song.as_ref()
    }

    pub(crate) const fn curr_playing_pos(&self) -> u32 {
        self.curr_playing_pos
    }
}
