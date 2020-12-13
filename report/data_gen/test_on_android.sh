cargo build --release --target=armv7-linux-androideabi
adb -d shell rm -r /sdcard/broccoli/graphs
adb -d push target/armv7-linux-androideabi/release/dada_gen /data/local/tmp/data_gen
echo "Running test.."
adb -d shell /data/local/tmp/data_gen bench /sdcard/broccoli/graphs
echo "Stopped running test"
adb -d pull "/sdcard/broccoli/graphs" $1
