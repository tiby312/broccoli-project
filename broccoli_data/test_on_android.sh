cargo build --release --target=armv7-linux-androideabi
adb -d shell rm -r /sdcard/dinotree/graphs
adb -d push target/armv7-linux-androideabi/release/dinotree_alg_data /data/local/tmp/dinotree_data
echo "Running test.."
adb -d shell /data/local/tmp/dinotree_data bench /sdcard/dinotree/graphs
echo "Stopped running test"
adb -d pull "/sdcard/dinotree/graphs" $1
