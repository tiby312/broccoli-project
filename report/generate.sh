set -ex



cargo build --release  --manifest-path data-gen/Cargo.toml

rm -rf src/raw
../target/release/data-gen bench src/raw
../target/release/data-gen theory src/raw

mkdir -p src/graphs
cargo-deps deps --exclude demo,data-gen | dot -Tsvg> src/graphs/graph.svg


#RUSTFLAGS='-g'
#cargo flamegraph --bin data_gen profile
#RUSTFLAGS=``

mdbook build -d ../target/book
mdbook serve -d ../target/book -p 3001
