# Navidrome Charts

Generate local top-chart playlists for your [Navidrome](https://www.navidrome.org/).

## Motivation

There is a cool feature in Navidrome -
[Smart Playlists](https://www.navidrome.org/docs/usage/features/smart-playlists/),
but it is very limited. e.g. you can't create a playlist with the most played
songs among all users on the server. This program attempts to fill this gap.

Originally I wanted to implement it as a Navidrome Plugin, but turns out
plugins can't access navidrome database. And Subsonic API isn't able to perform
advanced aggregation queries either.

And this is why I ended up with the following idea:

1. Directly query `navidrome.db` on the server
2. Create or update playlists via Subsonic API
3. Repeat with some interval

## Installation

### NixOS (recommended)

Add this repository as a flake input and include the NixOS module:

`services.navidrome-charts` will create a systemd timer that will update
playlists with the specified interval.

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    navidrome-charts.url = "github:rwxae/navidrome-charts";
    navidrome-charts.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, navidrome-charts, ... }: {
    nixosConfigurations.myhost = nixpkgs.lib.nixosSystem {
      modules = [
        navidrome-charts.nixosModules.default
        {
          services.navidrome-charts = {
            enable = true;
            # environmentFile = "/run/secrets/navidrome-charts.env";
            # interval = "*:0/15";   # every 15 min
            # dbPath = "/var/lib/navidrome/navidrome.db";
          };
        }
      ];
    };
  };
}
```

To access Navidrome API you need to provide the following env variables:

```env
SUBSONIC_SERVER_URL=https://your.navidrome.com/
SUBSONIC_USERNAME=user
SUBSONIC_PASSWORD=secret
```

By default `dbPath` is inferred like this: `${DataFolder}/navidrome.db`.
If you haven't specified DataFolder for Navidrome, then you should manually
provide a path to database file.
