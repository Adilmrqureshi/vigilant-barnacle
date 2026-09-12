#!/usr/bin/env bash
# Build the full site exactly like .github/workflows/deploy.yml and serve it
# locally. Usage: ./local/serve.sh   (PORT=9000 ./local/serve.sh to change port)
set -euo pipefail

cd "$(dirname "$0")/.."

RUSTFLAGS="-C link-arg=--allow-undefined" cargo build --release --target wasm32-unknown-unknown

SITE=target/site
rm -rf "$SITE"
mkdir -p "$SITE"
cp -a web/. "$SITE"/

for wasm in target/wasm32-unknown-unknown/release/*.wasm; do
    name=$(basename "$wasm" .wasm)
    # only ship real games, not stray extra build targets
    [ -d "games/$name" ] || continue
    mkdir -p "$SITE/games/$name"
    cp "$wasm" "$SITE/games/$name/"
    if [ -d "games/$name/assets" ]; then
        cp -r "games/$name/assets" "$SITE/games/$name/assets"
    fi
    cp web/template.txt "$SITE/games/$name/index.html"
    echo "<script>load(\"$name.wasm\");</script></body></html>" >> "$SITE/games/$name/index.html"
    echo "built $name"
done

python3 local/server.py "$SITE" "${PORT:-8000}"
