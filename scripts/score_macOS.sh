#! /bin/bash

source pytorch/bin/activate

export LIBTORCH_USE_PYTORCH=1 \
    DYLD_LIBRARY_PATH="$(find pytorch -type d -name "lib" | grep /torch):$DYLD_LIBRARY_PATH" && \
    cargo run -p score --release

deactivate
