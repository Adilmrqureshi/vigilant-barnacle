#!/usr/bin/env bash

cp "../target/wasm32-unknown-unknown/release/$1.wasm" "./$1.wasm"
cp -r "../games/$1/assets" "./assets"
sed -i "s/load(\".*\")/load(\"$1.wasm\")/g" ./index.html 
basic-http-server
