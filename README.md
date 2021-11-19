

# How to build

## Preconditions
1. Install rust & cargo
2. Install wasm-pack from `https://rustwasm.github.io/wasm-pack/installer/`
3. `cargo install cargo-watch`

## Build server
`cargo build`. 

## Build client side

* `wasm-pack build --out-dir ../static --target web`
* for release `wasm-pack build --out-dir ../static --target web --release`
* watching changes: `cargo watch -s "wasm-pack build --out-dir ../static --target web"`

## Run
`cargo run --package web-server` 

# Docker 

Build with `docker build -t badbee .`

Run with specific base only `docker run -d --rm --name badbee -e DB_FILE=db.png -p 3030:3030 -v "$pwd/db:/usr/badbee/db"  badbee`

Run with all bases: `docker run -d --rm --name badbee -p 3030:3030 -v "$pwd/db:/usr/badbee/db"  badbee`

## For debug

Run bash to check pathes and other: `docker run --rm -it --entrypoint bash badbee`

Run cadvisor to monitor resources: `docker run -d --rm --name cadvisor -p 8080:8080 --volume=/:/rootfs:ro --volume=/var/run:/var/run:rw --volume=/sys:/sys:ro --volume=/var/lib/docker/:/var/lib/docker:ro google/cadvisor`

