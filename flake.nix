{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    futils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    futils,
  } @ inputs: let
    inherit (nixpkgs) lib;
    inherit (futils.lib) eachDefaultSystem defaultSystems;

    nixpkgsFor = lib.genAttrs defaultSystems (system:
      import nixpkgs {
        inherit system;
        overlays = [
          rust-overlay.overlays.default
        ];
      });
  in
    eachDefaultSystem (system: let
      pkgs = nixpkgsFor.${system};
    in {
      devShell = pkgs.mkShell {
        buildInputs = with pkgs; [
          go_1_23
          nodejs_22
          python312Full
          (lib.hiPrio rust-bin.nightly.latest.rustfmt)
          (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)

          bun
          gcc
          gettext
          glibc
          krb5.dev
          krb5.out
          libtool
          libxml2
          libxslt
          openssl
          pkg-config
          postgresql
          redis
          xmlsec
          zlib

          docker-compose
          git
          gnumake
          golangci-lint
          k6
          poetry
          ruff
          yarn
        ];

        RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        RUST_BACKTRACE = 1;
      };
    });
}
