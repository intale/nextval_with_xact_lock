# nextval_with_xact_lock

Implementation of atomic function of SELECT pg_advisory_xact_lock(nextval('my_seq'::regclass)) query.

## Requirements

Supported PostgreSQL versions:
- 16
- 17
- 18

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
