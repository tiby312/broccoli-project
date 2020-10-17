set -ex

cargo build --release --manifest-path data_gen/Cargo.toml

rm -rf src/raw
../target/release/data_gen bench src/raw
#../target/release/data_gen theory src/raw

rm -rf book/graphs
../target/release/data_gen graph src/raw book/graphs



#xdg-open http://localhost:3000/ &
#mdbook serve
xdg-open book/index.html
