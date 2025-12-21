#!/bin/bash

export PGVER="${PGVER:-18}"
cargo pgrx install --no-default-features --features "lock_tests pg$PGVER" -c "$(cargo pgrx info pg-config "pg$PGVER")"
cargo pgrx start "pg$PGVER"
cargo test --no-default-features --features "lock_tests pg_test pg$PGVER" "$@" -- --test-threads=1
cargo pgrx stop "pg$PGVER"
