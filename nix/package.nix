{ rustPlatform }:

rustPlatform.buildRustPackage {
  src = ../.;

  name = with builtins; (fromTOML (readFile ../Cargo.toml)).package.name;

  cargoHash = "sha256-WDwghYxbz7pvzuEZpyqGZtVVKFi/xsbRELzTh6mg9zY=";

  meta.mainProgram = "navidrome-charts";
}
