{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let 
        pkgs = import nixpkgs { 
          inherit system; 
          overlays = [ (import rust-overlay) ];
        };

        rustToolchain = pkgs.rust-bin.stable."1.70.0".default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

      in 
      {
        devShells.default = pkgs.mkShell { 
          packages = with pkgs; [ 
            rustToolchain
            google-cloud-sdk

            pkg-config
            openssl
            sqlite
            tokio-console

            (python311.withPackages(ps: with ps; [ google-api-python-client ]))
          ]; 
          
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          DATABASE_URL = "./trends.db";
        };
      });
}
