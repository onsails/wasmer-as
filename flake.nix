{
  description = "A very basic flake";

  inputs = {
    utils.url = github:numtide/flake-utils;
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, utils, rust-overlay }: utils.lib.eachDefaultSystem (
    system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlay ];
        };
      in
        {

          checks.ci = pkgs.stdenv.mkDerivation {
            name = "ci";

            src = ./.;

            buildInputs = with pkgs;
              [
                rust-bin.stable.latest.default
                nodejs-14_x

                cacert
              ];

            buildPhase = ''
              cd test-wasm
              export HOME=$TMP
              npm install
              npm run asbuild

              cd ..
              cargo test

              mkdir $out
            '';

            dontInstall = true;
            dontFixup = true;
          };

          devShell = pkgs.mkShell {
            buildInputs = with pkgs; [
              rust-bin.stable.latest.default
              nodejs-14_x

              cargo-release
            ];
          };
        }
  );
}
