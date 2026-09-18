use rand::distr::{Alphanumeric, SampleString};
use reqwest::{Url, blocking::Client};

static CLIENT_NAME: &str = "navidrome-charts";
static SUBSONIC_VERSION: &str = "1.16.1";

pub struct SubsonicClient {
    base_url: Url,
    username: String,
    password: String,
    client: Client,
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

    pub fn create_playlist(
        &self,
        title: String,
        songs: impl Iterator<Item = String>,
        playlist_id: Option<String>,
    ) -> Result<(), reqwest::Error> {
        let mut url = self.base_url.clone();
        url.set_path("rest/createPlaylist");
        let mut params = self.get_params();
        params.push(("name", title));
        params.extend(songs.map(|id| ("songId", id)));
        if let Some(playlist_id) = playlist_id {
            params.push(("playlistId", playlist_id));
        }
        self.client.post(url).query(&params).send()?;
        Ok(())
    }
}
