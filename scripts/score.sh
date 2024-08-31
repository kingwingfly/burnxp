export LD_LIBRARY_PATH="$(pwd)/libtorch/lib:$LD_LIBRAR_PATH"
    LIBTORCH="$(pwd)/libtorch" && cargo run -p score --release