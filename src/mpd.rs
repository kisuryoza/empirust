//! MPD data holder

use mpd::{Client, Playlist, Song, Status};
use std::error::Error;

#[derive(Debug)]
/// Holds MPD's data
pub(crate) struct Mpd {
    client: Client,
    status: Status,
    playlists: Option<Vec<Playlist>>,
    queue: Option<Vec<Song>>,
    prev_song: Option<Song>,
    curr_song: Option<Song>,
    curr_playing_pos: u32,
    curr_song_duration: u16,
}

impl Mpd {
    pub(crate) fn new(mut client: Client) -> Result<Self, Box<dyn Error>> {
        let status = client.status()?;
        let playlists = match client.playlists() {
            Ok(arg) => Some(arg),
            Err(_) => None,
        };
        let queue = match client.queue() {
            Ok(arg) => Some(arg),
            Err(_) => None,
        };
        let curr_song = match client.currentsong() {
            Ok(arg) => arg,
            Err(_) => None,
        };
        let curr_playing_pos = match status.song {
            Some(arg) => arg.pos,
            None => 0,
        };
        let curr_song_duration: u16 = match status.time {
            Some(time) => time.1.num_seconds().try_into().unwrap_or(1),
            None => 1,
        };

        Ok(Self {
            client,
            status,
            playlists,
            queue,
            prev_song: curr_song.clone(),
            curr_song,
            curr_playing_pos,
            curr_song_duration,
        })
    }

    pub(crate) fn update(&mut self) {
        self.status = self.client.status().unwrap();
        self.curr_song = match self.client.currentsong() {
            Ok(arg) => arg,
            Err(_) => None,
        };
        self.curr_playing_pos = match self.status.song {
            Some(arg) => arg.pos,
            None => 0,
        };

        // update data of the new song
        if self.curr_song != self.prev_song {
            self.prev_song = self.curr_song.clone();
            self.curr_song_duration = match self.status.time {
                Some(time) => time.1.num_seconds().try_into().unwrap_or(0),
                None => 0,
            };
        }
    }

    pub(crate) fn client_mut(&mut self) -> &mut Client {
        &mut self.client
    }

    pub(crate) fn status(&self) -> &Status {
        &self.status
    }

    pub(crate) fn playlists(&self) -> Option<&Vec<Playlist>> {
        self.playlists.as_ref()
    }

    pub(crate) fn queue(&self) -> Option<&Vec<Song>> {
        self.queue.as_ref()
    }

    pub(crate) fn curr_song(&self) -> Option<&Song> {
        self.curr_song.as_ref()
    }

    pub(crate) fn curr_playing_pos(&self) -> u32 {
        self.curr_playing_pos
    }
}
