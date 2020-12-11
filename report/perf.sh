cargo build --release --manifest-path data_gen/Cargo.toml
perf record --call-graph=dwarf -o ../target/release/perf.data ../target/release/data_gen
perf report --hierarchy -M intel -i ../target/release/perf.data
