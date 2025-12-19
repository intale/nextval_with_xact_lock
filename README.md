# nextval_with_xact_lock

Implementation of atomic function of SELECT pg_advisory_xact_lock(nextval('my_seq'::regclass)) query.

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
ALTER TABLE my_table ALTER COLUMN id SET DEFAULT pg_nextval_with_xact_lock('public.my_table_id_seq'::regclass);
```

Now you can see uncommited ids in `pg_locks`. Please note, because advisory locks are scoped to the specific database - you can't differentiate between locks, coming from different sequences. Thus, it is meaningless to use this extension for more than one sequence per database.

## Limitations

Because original sequence functions `nextval()`, `currval()`, `lastval()` and `setval()` are implemented in a way they rely on private global in-memory state - `nextval_with_xact_lock()`, implemented in this extension, is not compatible with them. Thus, the only info you can have is last persisted value of the sequence, retrieved by reading directly from your sequence table, e.g. `SELECT last_value FROM my_sequence LIMIT 1`. Potential `currval()`, `lastval()` and `setval()` alternatives that would work with `nextval_with_xact_lock()` should be implemented separately which is not planed yet, but the contribution is very welcome.  

## Development

### Running tests

To run integration tests you can use `./bin/run_tests.sh` script. This will run tests for the latest supported PostgreSQL version. You can supply `PGVER` environment variable to provide different version. You also can provide a test pattern matching as a first argument. Examples:

```bash
# Run tests for the latest supported PostgreSQL version
./bin/run_tests.sh
# Run tests for PostgreSQL v16
PGVER=16 ./bin/run_tests.sh
# Run specific tests
./bin/run_tests.sh cycle_sequence_tests
```

To run tests for all versions you can use `./bin/run_all_tests.sh`

```bash
./bin/run_all_tests.sh
```

## Contributing

Bug reports and pull requests are welcome on GitHub at https://github.com/intale/nextval_with_xact_lock. This project is intended to be a safe, welcoming space for collaboration, and contributors are expected to adhere to the [code of conduct](https://github.com/intale/nextval_with_xact_lock/blob/main/CODE_OF_CONDUCT.md).

## License

The extension is available as open source under the terms of the [BSD 2-Clause License](https://github.com/intale/nextval_with_xact_lock/blob/main/LICENSE).

## Code of Conduct

Everyone interacting in the nextval_with_xact_lock project's codebases, issue trackers, chat rooms and mailing lists is expected to follow the [code of conduct](https://github.com/intale/nextval_with_xact_lock/blob/main/CODE_OF_CONDUCT.md).
