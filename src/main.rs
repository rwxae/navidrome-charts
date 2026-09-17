use std::path::PathBuf;

use clap::Parser;
use rand::distr::{Alphanumeric, SampleString};
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

    /// Most Played playlist title
    #[arg(long)]
    most_played_title: Option<String>,
}

struct App {
    params: Args,
    db_connection: Connection,
    subsonic_client: Client,
}

impl App {
    fn new(params: Args) -> Result<Self, rusqlite::Error> {
        let db_connection =
            Connection::open_with_flags(&params.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        Ok(Self {
            params,
            db_connection,
            subsonic_client: Client::new(),
        })
    }

    fn prepare_url(&self) -> Url {
        let mut url = self.params.server_url.clone();
        let salt = Alphanumeric.sample_string(&mut rand::rng(), 10);
        let digest = md5::compute(self.params.password.clone() + &salt);
        let token = format!("{:x}", digest);
        url.query_pairs_mut()
            .append_pair("c", CLIENT_NAME)
            .append_pair("v", SUBSONIC_VERSION)
            .append_pair("f", "json")
            .append_pair("u", &self.params.username)
            .append_pair("s", &salt)
            .append_pair("t", &token);
        url
    }
}

// struct Playlist {
//     id: String,
//     title: String
// }

static CLIENT_NAME: &str = "navidrome-charts";
static SUBSONIC_VERSION: &str = "1.16.1";

fn create_or_update_most_played(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let title = app
        .params
        .most_played_title
        .clone()
        .unwrap_or(format!("{} Most Played - Global", app.params.max_size));

    let mut statement = app.db_connection.prepare(
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

    let songs =
        statement.query_map([app.params.max_size], |row| row.get(0) as Result<String, _>)?;

    let mut url = app.prepare_url();
    url.set_path("rest/createPlaylist");
    let mut url_query = url.query_pairs_mut();
    url_query.append_pair("name", &title);

    // TODO: fetch actual id from subsonic
    let maybe_playlist_id = Some("ZArLXt85UARuq8Qo4ncY5r");

    if let Some(playlist_id) = maybe_playlist_id {
        url_query.append_pair("playlistId", playlist_id);
    };

    for song in songs {
        url_query.append_pair("songId", song?.as_str());
    }
    drop(url_query);

    app.subsonic_client.get(url).send()?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let app = App::new(args)?;

    create_or_update_most_played(&app)?;

    Ok(())
}
