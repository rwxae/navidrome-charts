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

fn create_or_update_most_played(
    db: &Connection,
    client: &SubsonicClient,
    title: String,
    max_size: u32,
    playlists: &[Playlist],
) -> Result<(), Box<dyn std::error::Error>> {
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

    let songs = statement
        .query_map([max_size], |row| row.get(0) as Result<String, _>)?
        .map(|song| song.unwrap());

    let maybe_playlist_id = playlists
        .iter()
        .find(|&playlist| playlist.name == title && playlist.owner == client.username())
        .map(|playlist| playlist.id.clone());

    client.create_playlist(title, songs, maybe_playlist_id)?;

    Ok(())
}

fn create_or_update_recently_played(
    db: &Connection,
    client: &SubsonicClient,
    title: String,
    max_size: u32,
    playlists: &[Playlist],
) -> Result<(), Box<dyn std::error::Error>> {
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

    let songs = statement
        .query_map([max_size], |row| row.get(0) as Result<String, _>)?
        .map(|song| song.unwrap());

    let maybe_playlist_id = playlists
        .iter()
        .find(|&playlist| playlist.name == title && playlist.owner == client.username())
        .map(|playlist| playlist.id.clone());

    client.create_playlist(title, songs, maybe_playlist_id)?;

    Ok(())
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

    let most_played_title =
        most_played_title.unwrap_or_else(|| format!("{} Most Played - Global", max_size));
    create_or_update_most_played(
        &db_connection,
        &subsonic_client,
        most_played_title,
        max_size,
        &playlists,
    )?;

    let recently_played_title =
        recently_played_title.unwrap_or_else(|| format!("{} Recently Played - Global", max_size));
    create_or_update_recently_played(
        &db_connection,
        &subsonic_client,
        recently_played_title,
        max_size,
        &playlists,
    )?;

    Ok(())
}
