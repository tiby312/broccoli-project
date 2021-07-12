set -ex



cargo build --release  --manifest-path data_gen/Cargo.toml

rm -rf src/raw
../target/release/data_gen bench src/raw
../target/release/data_gen theory src/raw

mkdir -p src/graphs
cargo-deps deps --exclude demo,data_gen | dot -Tsvg> src/graphs/graph.svg


#RUSTFLAGS='-g'
#cargo flamegraph --bin data_gen profile
#RUSTFLAGS=``

mdbook serve -p 3001
