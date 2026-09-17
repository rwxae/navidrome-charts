use std::path::PathBuf;

use clap::Parser;
use reqwest::{Url, blocking::Client};
use rusqlite::{Connection, OpenFlags};

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

    #[arg(long, default_value = "1.16.1")]
    subsonic_version: String,
}

static CLIENT_NAME: &str = "navidrome-charts";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Args {
        db_path,
        server_url,
        username,
        password,
        max_size,
        subsonic_version,
    } = Args::parse();
    let conn = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let client = Client::new();

    let mut statement = conn.prepare(
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

    let mut url = server_url.clone();
    url.set_path("rest/createPlaylist");
    let mut url_query = url.query_pairs_mut();
    url_query
        .append_pair("c", CLIENT_NAME)
        .append_pair("v", &subsonic_version)
        .append_pair("f", "json")
        .append_pair("u", &username)
        .append_pair("p", &password)
        .append_pair("name", "200 Most Played - Global");

    // TODO: fetch actual id from subsonic
    let maybe_playlist_id = Some("ZArLXt85UARuq8Qo4ncY5r");

    if let Some(playlist_id) = maybe_playlist_id {
        url_query.append_pair("playlistId", playlist_id);
    };

    let songs = statement.query_map([max_size], |row| row.get(0) as Result<String, _>)?;

    for song in songs {
        url_query.append_pair("songId", song?.as_str());
    }
    drop(url_query);

    client.get(url).send()?;

    Ok(())
}
