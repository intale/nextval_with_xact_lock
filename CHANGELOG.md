## [Unreleased]

## [0.1.1]

- Adjust signature of read_seq_tuple() to march C implementation
- Store SEQHASHTAB as a simple pointer instead RwLock
- Store SEQHASHTAB in top PostgreSQL memory
- Extract part of the functional into init_sequence() to closer match C implementation
- Fix read_seq_tuple() to throw error instead of just logging it

## [0.1.0]

- Initial release.
