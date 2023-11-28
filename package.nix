{ rustPlatform, fetchFromGitHub, pkg-config, openssl, rust-bin, gcc, cargo-auditable-cargo-wrapper, ... }:
let
  name = "acmed-hook-rfc2136";
  v = "main";

  repo-src = fetchFromGitHub {
    owner = "famedly";
    repo = "${name}";
    rev = "${v}";
    hash = "sha256-0gyqXRUeHzNrywwCgbWHr2v3nxrWNAT4w0mNUb1Q3sk";
  };

in rustPlatform.buildRustPackage rec {
  pname = "${name}";
  version = "${v}";
  src = repo-src;
  cargoLock.lockFile = ( src + "/Cargo.lock" );
  cargoDeps = rustPlatform.importCargoLock {
    lockFile = ( src + "/Cargo.lock" );
  };
  nativeBuildInputs = [ rust-bin.stable.latest.complete pkg-config openssl ];
  buildInputs = [ pkg-config openssl ];
  buildType = "debug";
  auditable = true;
  logLevel = "TRACE";
}
