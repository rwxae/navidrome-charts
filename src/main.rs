use std::env;
use std::path::PathBuf;

use clap::Parser;
use reqwest::{Url, blocking::Client};
use rusqlite::{Connection, OpenFlags};

use crate::subsonic::{Playlist, SubsonicClient};

mod subsonic;

#[derive(Parser)]
struct Args {
    db_path: PathBuf,

    /// Playlist max size
    #[arg(long, default_value_t = 200)]
    max_size: u32,

    /// Most Played playlist title
    #[arg(long)]
    most_played_title: Option<String>,

    /// Recently Played playlist title
    #[arg(long)]
    recently_played_title: Option<String>,
}

struct App {
    client: SubsonicClient,
    connection: Connection,
    playlists: Vec<Playlist>,
    max_size: u32,
}

impl App {
    fn new(
        client: SubsonicClient,
        connection: Connection,
        playlists: Vec<Playlist>,
        max_size: u32,
    ) -> Self {
        Self {
            client,
            connection,
            playlists,
            max_size,
        }
    }

    fn get_playlist_id(&self, title: &str) -> Option<String> {
        self.playlists
            .iter()
            .find(|&playlist| playlist.name == title && playlist.owner == self.client.username())
            .map(|playlist| playlist.id.clone())
    }
}

fn get_most_played_songs(db: &Connection, max_size: u32) -> Result<Vec<String>, rusqlite::Error> {
    let mut statement = db.prepare(
        "
            SELECT
                mf.id,
                SUM(a.play_count) AS global_play_count
            FROM media_file mf
            JOIN annotation a ON mf.id = a.item_id AND a.item_type = 'media_file'
            GROUP BY mf.id
            HAVING global_play_count > 0
            ORDER BY global_play_count DESC
            LIMIT ?1;
        ",
    )?;
    let records = statement.query_map([max_size], |row| row.get(0) as Result<String, _>)?;

    let mut songs = Vec::with_capacity(max_size as usize);
    for record in records {
        let song = record?;
        songs.push(song);
    }

    Ok(songs)
}

fn get_recently_played_songs(
    db: &Connection,
    max_size: u32,
) -> Result<Vec<String>, rusqlite::Error> {
    let mut statement = db.prepare(
        "
            SELECT
                mf.id,
                MAX(a.play_date) AS play_date
            FROM media_file mf
            JOIN annotation a ON mf.id = a.item_id AND a.item_type = 'media_file'
            WHERE a.play_date IS NOT NULL
            GROUP BY mf.id
            ORDER BY play_date DESC
            LIMIT ?1;
        ",
    )?;
    let records = statement.query_map([max_size], |row| row.get(0) as Result<String, _>)?;

    let mut songs = Vec::with_capacity(max_size as usize);
    for record in records {
        let song = record?;
        songs.push(song);
    }

    Ok(songs)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Args {
        db_path,
        max_size,
        most_played_title,
        recently_played_title,
    } = Args::parse();

    let server_url: Url = env::var("SUBSONIC_SERVER_URL")
        .map_err(|_| "missing SUBSONIC_SERVER_URL env")?
        .parse()?;
    let username = env::var("SUBSONIC_USERNAME").map_err(|_| "missing SUBSONIC_USERNAME env")?;
    let password = env::var("SUBSONIC_PASSWORD").map_err(|_| "missing SUBSONIC_PASSWORD env")?;

    let db_connection = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let subsonic_client = SubsonicClient::new(server_url, username, password, Client::new());

    let playlists = subsonic_client.get_playlists()?;
    let app = App::new(subsonic_client, db_connection, playlists, max_size);

    let title = most_played_title.unwrap_or_else(|| format!("{} Most Played - Global", max_size));
    let songs = get_most_played_songs(&app.connection, app.max_size)?;
    let id = app.get_playlist_id(&title);
    app.client.create_playlist(title, songs, id)?;

    let title =
        recently_played_title.unwrap_or_else(|| format!("{} Recently Played - Global", max_size));
    let songs = get_recently_played_songs(&app.connection, app.max_size)?;
    let id = app.get_playlist_id(&title);
    app.client.create_playlist(title, songs, id)?;

    Ok(())
}
