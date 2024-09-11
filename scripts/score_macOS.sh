#! /bin/bash

source .venv/bin/activate

export LIBTORCH_USE_PYTORCH=1 \
    DYLD_LIBRARY_PATH="$(find .venv -type d -name "lib" | grep /torch):$DYLD_LIBRARY_PATH" && \
    cargo run -p score --release -- -h

deactivate
