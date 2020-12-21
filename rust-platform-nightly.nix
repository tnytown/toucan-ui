{ callPackage, fetchFromGitHub, makeRustPlatform }:

# The date of the nighly version to use.
date:

let
  mozillaOverlay = fetchFromGitHub {
    owner = "mozilla";
    repo = "nixpkgs-mozilla";
    rev = "8c007b60731c07dd7a052cce508de3bb1ae849b4";
    sha256 = "1zybp62zz0h077zm2zmqs2wcg3whg6jqaah9hcl1gv4x8af4zhs6";
  };
  mozilla = callPackage "${mozillaOverlay.out}/package-set.nix" {};
  rustNightly = (mozilla.rustChannelOf { inherit date; channel = "nightly"; }).rust;
in makeRustPlatform {
  cargo = rustNightly;
  rustc = rustNightly;
}
