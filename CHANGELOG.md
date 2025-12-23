## [Unreleased]

## [0.1.2]

- Use MemoryContextSwitchTo() to switch memory context instead of direct context assignment
- Wrap grab_advisory_lock() into PgTryBuilder() and ensure buffer and/or sequence are closed if LockAcquire() crashes
- Add missing PreventCommandIfParallelMode() guard
- Throw an error instead error log when SearchSysCache1() fails to find sequence in cache
- Fix behavior when last update of the sequence was before checkpoint

## [0.1.1]

- Adjust signature of read_seq_tuple() to march C implementation
- Store SEQHASHTAB as a simple pointer instead RwLock
- Store SEQHASHTAB in top PostgreSQL memory
- Extract part of the functional into init_sequence() to closer match C implementation
- Fix read_seq_tuple() to throw error instead of just logging it

## [0.1.0]

- Initial release.
