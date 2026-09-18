self:
{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.services.navidrome-charts;
  navidrome-charts = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
in
{
  options.services.navidrome-charts = {
    enable = lib.mkEnableOption "Navidrome playlist generator";

    assertions = [
      {
        assertion = config.services.navidrome.enable;
        message = "services.navidrome-charts requires services.navidrome.enable to be true.";
      }
    ];

    environmentFile = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      description = ''
        Path to a file containing environment variables for navidrome-charts.
        The file should contain lines like:

        SUBSONIC_SERVER_URL=https://example.com
        SUBSONIC_USERNAME=user
        SUBSONIC_PASSWORD=secret
      '';
    };

    interval = lib.mkOption {
      type = lib.types.str;
      default = "*:0/5";
      description = ''
        When or how often the periodic update should run.
        Must be the format described from systemd.time(7)
      '';
    };

    dbPath = lib.mkOption {
      type = lib.types.str;
      default = "${config.services.navidrome.settings.DataFolder}/navidrome.db";
      description = "Path to the Navidrome SQLite database file.";
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.services.navidrome-charts = {
      description = "Generate Navidrome playlists from play history";
      serviceConfig = {
        Type = "oneshot";
        User = config.services.navidrome.user;
        EnvironmentFile = lib.mkIf (cfg.environmentFile != null) [ cfg.environmentFile ];
        ExecStart = ''${lib.getExe navidrome-charts} "${cfg.dbPath}"'';
      };
      startAt = cfg.interval;
    };
  };
}
