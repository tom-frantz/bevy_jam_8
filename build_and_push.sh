#!/usr/bin/env bash

PROJECT_NAME="philosophers-grave";

[ -d ".cargo/bin/wasm-bindgen" ] || echo "wasm-bindgen not found" && exit 1
[ -d ".cargo/bin/wasm-opt"] || echo "wasm-opt not found" && exit 1 
which butler || echo "butler not found" && exit 1


bevy build --release web --bundle
OUTPUT_DIR="target/bevy_web/web-release/bevy_jam"

rm -r "$OUTPUT_DIR/assets/raw_artwork"
rm "$OUTPUT_DIR/assets/floating_island.aseprite"

OUTPUT_ZIP="target/bevy_web/web-release/bevy_jam.zip"
zip -r $OUTPUT_ZIP "$OUTPUT_DIR/" 

butler push $OUTPUT_ZIP tom-frantz/$PROJECT_NAME:web
echo "Pushed to itch.io"
