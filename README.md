# Rust games

Browser games written in Rust, compiled to WebAssembly and deployed to GitHub
Pages by `.github/workflows/deploy.yml`.

## Build

Requires the wasm target (`rustup target add wasm32-unknown-unknown`).

```
cargo build --release --target wasm32-unknown-unknown
```

## Run the whole site locally

Builds every game and assembles the site into `target/site` using the same
steps as the prod deploy workflow, then serves it on http://127.0.0.1:8000:

```
./local/serve.sh
```

Use `PORT=9000 ./local/serve.sh` to change the port. Re-run the script after
code changes; caching is disabled so a browser refresh picks up the new build.
