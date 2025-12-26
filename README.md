# nextval_with_xact_lock

Implementation of atomic function that performs `nextval()` + `pg_try_advisory_lock()`.

## Requirements

Supported PostgreSQL versions:
- 16
- 17
- 18

## Usage

Get the next sequence value and acquire transaction-level advisory lock:

```sql
SELECT nextval_with_xact_lock('my_seq'::regclass);
```

The lock is obtained before the sequence is released, thus no other sequence number can be issued meanwhile. The lock is non-blocking though. If something already holds advisory lock of the same value - `nextval_with_xact_lock()` doesn't wait for it to release.

Practical usage example:

```sql
ALTER TABLE my_table ALTER COLUMN id SET DEFAULT nextval_with_xact_lock('public.my_table_id_seq'::regclass);
```

Now you can see uncommited ids in `pg_locks`. Please note, because advisory locks are scoped to the specific database - you can't differentiate between locks, coming from different sequences. Thus, it is meaningless to use this extension for more than one sequence per database.

## Limitations

Because original sequence functions `nextval()`, `currval()`, `lastval()` and `setval()` are implemented in a way they rely on private global in-memory state - `nextval_with_xact_lock()`, implemented in this extension, is not compatible with them. Thus, the only info you can have is last persisted value of the sequence, retrieved by reading directly from your sequence table, e.g. `SELECT last_value FROM my_sequence LIMIT 1`. Potential `currval()`, `lastval()` and `setval()` alternatives that would work with `nextval_with_xact_lock()` should be implemented separately which is not planed yet, but the contribution is very welcome.  

## Installation

There is prepared `Dockerfile` that builds a PostgreSQL image with `nextval_with_xact_lock` extension installed. It builds PostgreSQL v18 by default. You can supply `PGVER` build argument to change PostreSQL version. Examples:

```bash
# Build PostgreSQL v18 image with nextval_with_xact_lock installed
docker build . -t "your_pg_repo/postgresql-nextval_with_xact_lock:18"
# Build PostgreSQL v17 image with nextval_with_xact_lock installed
docker build . -t "your_pg_repo/postgresql-nextval_with_xact_lock:17" --build-arg PGVER=17
```

Now you can push it to your own repo:

```bash
docker push your_pg_repo/postgresql-nextval_with_xact_lock:18
```

## Development

### Setup

- install Rust https://rust-lang.org/tools/install/
- install pgrx https://github.com/pgcentralfoundation/pgrx?tab=readme-ov-file#getting-started (install + initialize) - a framework for developing PostgreSQL extensions in Rust

### Playing around with the extension

- enter PostgreSQL console:
```bash
cargo pgrx run
```
Optionally you can provide `--features pg<version>` to run under specific version:
```bash
cargo pgrx run --features pg17
```
- create extension:
```sql
CREATE EXTENSION nextval_with_xact_lock;
```
- create some test sequence:
```sql
CREATE SEQUENCE my_seq;
```
- use `nextval_with_xact_lock()` to retrieve next sequence value:
```sql
SELECT nextval_with_xact_lock('my_seq'::regclass);
```

### Running tests

To run integration tests you can use `./bin/run_tests.sh` script. This will run tests for the latest supported PostgreSQL version. You can supply `PGVER` environment variable to provide different version. You can also provide a test pattern matching as a first argument. Examples:

```bash
# Run tests for the latest supported PostgreSQL version
./bin/run_tests.sh
# Run tests for PostgreSQL v16
PGVER=16 ./bin/run_tests.sh
# Run specific tests
./bin/run_tests.sh cycle_sequence_tests
```

To run tests for all versions you can use `./bin/run_all_tests.sh`. You can also provide a test pattern matching as a first argument.

```bash
# Run all tests
./bin/run_all_tests.sh
# Run specific tests
./bin/run_all_tests.sh cycle_sequence_tests
```

## Contributing

Bug reports and pull requests are welcome on GitHub at https://github.com/intale/nextval_with_xact_lock. This project is intended to be a safe, welcoming space for collaboration, and contributors are expected to adhere to the [code of conduct](https://github.com/intale/nextval_with_xact_lock/blob/main/CODE_OF_CONDUCT.md).

## License

The extension is available as open source under the terms of the [BSD 2-Clause License](https://github.com/intale/nextval_with_xact_lock/blob/main/LICENSE).

## Code of Conduct

Everyone interacting in the nextval_with_xact_lock project's codebases, issue trackers, chat rooms and mailing lists is expected to follow the [code of conduct](https://github.com/intale/nextval_with_xact_lock/blob/main/CODE_OF_CONDUCT.md).
