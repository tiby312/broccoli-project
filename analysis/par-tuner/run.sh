mkdir ../target/graphs
cargo run --release bench-par-create 1 512 > ../target/graphs/create.svg
cargo run --release bench-par-query 1 512 > ../target/graphs/query.svg