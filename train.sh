#! /bin/bash

export LD_LIBRARY_PATH="$(pwd)/libtorch/lib" LIBTORCH="$(pwd)/libtorch" && cargo run --release
