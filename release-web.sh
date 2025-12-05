#!/bin/sh

set -e

~/.cargo/bin/wasm-pack build --target web
cargo install clap_web_code_gen
~/.cargo/bin/clap-web-gen

rsync -av --delete --exclude .git --exclude .gitignore pkg/ ../rhyme-checker-web-release/
cd ../rhyme-checker-web-release
git add -A .
git commit -m 'new release'
git push origin master
