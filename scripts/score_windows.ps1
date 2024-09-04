$Env:LIBTORCH = "$(pwd)/libtorch/"
$Env:Path += ";$(pwd)/libtorch/;$(pwd)/libtorch/lib/"
cargo run -p score --release
