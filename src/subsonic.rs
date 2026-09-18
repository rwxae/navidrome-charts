use rand::distr::{Alphanumeric, SampleString};
use reqwest::{Url, blocking::Client};
use serde::Deserialize;

static CLIENT_NAME: &str = "navidrome-charts";
static SUBSONIC_VERSION: &str = "1.16.1";

pub struct SubsonicClient {
    base_url: Url,
    username: String,
    password: String,
    client: Client,
}

#[derive(Debug, Deserialize)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub owner: String,
}

#[derive(Debug, Deserialize)]
struct GetPlaylistsResponse {
    status: String,
    version: String,
    playlists: Playlists,
}

#[derive(Debug, Deserialize)]
struct Playlists {
    #[serde(default)]
    playlist: Vec<Playlist>,
}

#[derive(Debug, Deserialize)]
struct SubsonicResponse<T> {
    #[serde(rename = "subsonic-response")]
    subsonic_response: T,
}

fn get_salt() -> String {
    Alphanumeric.sample_string(&mut rand::rng(), 12)
}

fn get_token(password: &str, salt: &str) -> String {
    format!("{:x}", md5::compute(format!("{password}{salt}")))
}

impl SubsonicClient {
    pub fn new(base_url: Url, username: String, password: String, client: Client) -> Self {
        Self {
            base_url,
            username,
            password,
            client,
        }
    }

    fn get_params(&self) -> Vec<(&'static str, String)> {
        let salt = get_salt();
        let token = get_token(&self.password, &salt);
        vec![
            ("c", CLIENT_NAME.to_string()),
            ("v", SUBSONIC_VERSION.to_string()),
            ("f", "json".to_string()),
            ("u", self.username.clone()),
            ("s", salt),
            ("t", token),
        ]
    }

    fn endpoint(&self, path: &str) -> Url {
        let mut url = self.base_url.clone();
        url.set_path(format!("rest/{path}").as_str());
        url
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn create_playlist<T: IntoIterator<Item = String>>(
        &self,
        title: String,
        songs: T,
        playlist_id: Option<String>,
    ) -> Result<(), reqwest::Error> {
        let mut params = self.get_params();
        params.push(("name", title));
        params.extend(songs.into_iter().map(|id| ("songId", id)));
        if let Some(playlist_id) = playlist_id {
            params.push(("playlistId", playlist_id));
        }
        self.client
            .post(self.endpoint("createPlaylist"))
            .query(&params)
            .send()?;
        Ok(())
    }

    pub fn get_playlists(&self) -> Result<Vec<Playlist>, reqwest::Error> {
        let params = self.get_params();
        let response = self
            .client
            .get(self.endpoint("getPlaylists"))
            .query(&params)
            .send()?;
        response.error_for_status_ref()?;
        let json: SubsonicResponse<GetPlaylistsResponse> = response.json()?;
        Ok(json.subsonic_response.playlists.playlist)
    }
}
