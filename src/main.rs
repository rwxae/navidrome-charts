use std::path::PathBuf;

use clap::Parser;
use reqwest::{Url, blocking::Client};
use rusqlite::{Connection, OpenFlags};

use crate::subsonic::SubsonicClient;

mod subsonic;

#[derive(Parser)]
struct Args {
    db_path: PathBuf,

    #[arg(short, long)]
    server_url: Url,

    #[arg(short, long)]
    username: String,

    #[arg(short, long)]
    password: String,

    /// Playlist max size
    #[arg(long, default_value_t = 200)]
    max_size: u32,

    /// Most Played playlist title
    #[arg(long)]
    most_played_title: Option<String>,
}

fn create_or_update_most_played(
    db: &Connection,
    client: &SubsonicClient,
    title: String,
    max_size: u32,
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

    // TODO: fetch actual id from subsonic
    let maybe_playlist_id = Some("ZArLXt85UARuq8Qo4ncY5r".to_string());

    client.create_playlist(title, songs, maybe_playlist_id)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Args {
        db_path,
        server_url,
        username,
        password,
        max_size,
        most_played_title,
    } = Args::parse();

    let db_connection = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let subsonic_client = SubsonicClient::new(server_url, username, password, Client::new());

    let most_played_title =
        most_played_title.unwrap_or_else(|| format!("{} Most Played - Global", max_size));
    create_or_update_most_played(
        &db_connection,
        &subsonic_client,
        most_played_title,
        max_size,
    )?;

    Ok(())
}
