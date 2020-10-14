set -ex

cargo build --release --manifest-path data_gen/Cargo.toml

../target/release/data_gen bench src/raw
../target/release/data_gen theory src/raw
../target/release/data_gen graph src/raw src/graphs

xdg-open http://localhost:3000/ &
mdbook serve