set -ex
#RUSTFLAGS='-g'
#cargo flamegraph --bin data_gen profile
cargo build --release  --manifest-path data_gen/Cargo.toml

rm -rf src/raw
../target/release/data_gen bench src/raw
../target/release/data_gen theory src/raw

rm -rf book/graphs
../target/release/data_gen graph src/raw src/graphs

mdbook serve
