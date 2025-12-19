#!/bin/bash

export PGVER="${PGVER:-18}"
cargo pgrx package --no-default-features --features pg$PGVER -c $(cargo pgrx info pg-config pg$PGVER)
cargo pgrx install --no-default-features --features pg$PGVER -c $(cargo pgrx info pg-config pg$PGVER)
cargo pgrx start pg$PGVER
cargo test $@ -- --test-threads=1
cargo pgrx stop pg$PGVER
