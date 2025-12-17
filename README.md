# nextval_with_xact_lock
Implementation of atomic function of SELECT pg_advisory_xact_lock(nextval('my_seq'::regclass)) query

## Build
To build the extension for the specific pg version run:

```bash
export PGVER=16
cargo pgrx package --no-default-features --features pg$PGVER -c $(cargo pgrx info pg-config pg$PGVER)
```
