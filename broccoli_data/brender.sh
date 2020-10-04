rm -r $1 
cargo run --release bench $1/raw &&
cargo run --release theory $1/raw &&
cargo run --release graph $1/raw $1/rendered
