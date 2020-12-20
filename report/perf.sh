cargo build --release --manifest-path data_gen/Cargo.toml
sudo perf record --call-graph=dwarf -o ../target/release/perf.data ../target/release/data_gen profile
sudo perf report --hierarchy -M intel -i ../target/release/perf.data
