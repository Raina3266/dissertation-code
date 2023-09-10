# Translation dissertation

## Building/running

There are 3 binaries in this project: `scrape_bbc`, `score_urls`, and `analyze`.
These can be installed globally using `cargo install --path .`.
Alternatively, they can be run through cargo by running `cargo run --release --bin <binary_name> -- <arguments>`.

If you install the programs globally, they can be uninstalled by running `cargo uninstall dissertation`.

This code expects a file to be present at `./.env` containing API keys for various services.
An example `.env` file might be:
```
OPENAI_KEY=<openai key>
GCLOUD_KEY=<google cloud api key>
GCLOUD_PROJECT_ID=<google cloud project id>
```

This program was written with:
 - `cargo` version 1.70.0
 - `rustc` version 1.70.0
 - `gcloud` version 433.0.1

### Nix

This project is built and managed with [Nix][nix], a package manager and build environment that allows reproducible builds.
The `flake.nix` file provides acts as a specification for the project, and the `flake.lock` file sets specific versions of each tool that is required to run the project.

This means that, even if future versions of tools are released that are incompatible, any future researchers will be able to reliably reproduce the program as we have (though results of 3rd party APIs may change).

Nix users can enter a "development shell" by running `nix develop`, which will download the correct versions of any required tools, and make them available on the `$PATH`.

Nix is **entirely optional**. Non nix-users can safely ignore the contents of `flake.nix` and `flake.lock`. Users are free to use whatever package manager they want to install the required versions of each tool.

