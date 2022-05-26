#!/bin/bash
RUSTFLAGS="$RUSTFLAGS -A non_snake_case -A unused_imports -A dead_code" cargo run -r