#[path="support/mod.rs"]
mod support;

mod common_sequence_tests {
    use super::*;

    async fn setup() {
        support::ensure_db_ready().await;
        let conn = support::connect().await;
        conn.batch_execute("DROP SEQUENCE IF EXISTS my_seq;").await.unwrap();
        conn.batch_execute("CREATE SEQUENCE my_seq;").await.unwrap();
    }

    mod when_concurrent_transactions_retrieve_nextval {
        use std::time::Duration;
        use super::*;

        #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
        async fn it_assigns_nextval_correctly() {
            setup().await;
            let conn1 = support::connect().await;
            let conn2 = support::connect().await;

            let thread1 = async {
                conn1.batch_execute("BEGIN").await.unwrap();
                let v: i64 = conn1
                    .query_one("SELECT nextval_with_xact_lock('my_seq'::regclass)", &[])
                    .await
                    .unwrap()
                    .get(0);
                tokio::time::sleep(Duration::from_millis(100)).await;
                conn1.batch_execute("COMMIT").await.unwrap();
                v
            };

            let thread2 = async {
                conn2.batch_execute("BEGIN").await.unwrap();
                let v: i64 = conn2
                    .query_one("SELECT nextval_with_xact_lock('my_seq'::regclass)", &[])
                    .await
                    .unwrap()
                    .get(0);
                tokio::time::sleep(Duration::from_millis(100)).await;
                conn2.batch_execute("COMMIT").await.unwrap();
                v
            };
            let (val1, val2) = tokio::join!(thread1, thread2);
            let mut res = vec![val1, val2];
            res.sort();
            assert_eq!(res, [1, 2]);
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
        async fn it_produces_advisory_lock() {
            setup().await;
            let conn1 = support::connect().await;
            let conn2 = support::connect().await;

            let thread1 = async {
                conn1.batch_execute("BEGIN").await.unwrap();
                conn1
                    .batch_execute("SELECT nextval_with_xact_lock('my_seq'::regclass)")
                    .await
                    .unwrap();
                tokio::time::sleep(Duration::from_millis(100)).await;
                conn1.batch_execute("COMMIT").await.unwrap();
                vec![]
            };

            let thread2 = async {
                conn2.batch_execute("BEGIN").await.unwrap();
                conn2
                    .batch_execute("SELECT nextval_with_xact_lock('my_seq'::regclass)")
                    .await
                    .unwrap();
                tokio::time::sleep(Duration::from_millis(100)).await;
                conn2.batch_execute("COMMIT").await.unwrap();
                vec![]
            };

            let thread3 = async {
                tokio::time::sleep(Duration::from_millis(50)).await;
                let query = "\
                SELECT objid::int4 FROM pg_locks WHERE locktype = 'advisory' AND objsubid = 1 \
                ORDER BY objid ASC\
            ";
                let res: Vec<i32> = support::connect().await
                    .query(query, &[])
                    .await
                    .unwrap().iter().map(|row| row.get(0)).collect();
                res
            };

            let (_, _, res): (Vec<i32>, Vec<i32>, _) = tokio::join!(thread1, thread2, thread3);
            assert_eq!(res, [1, 2]);
        }
    }

    mod when_another_transaction_already_holds_advisory_lock_with_the_same_argument {
        use std::time::Duration;
        use crate::common_sequence_tests::setup;
        use crate::support;

        #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
        async fn it_does_wait_for_it() {
            setup().await;
            let conn1 = support::connect().await;
            let conn2 = support::connect().await;

            let thread1 = async {
                conn1.batch_execute("BEGIN").await.unwrap();
                conn1.batch_execute("SELECT pg_advisory_xact_lock(1);").await.unwrap();
                tokio::time::sleep(Duration::from_millis(100)).await;
                conn1.batch_execute("COMMIT").await.unwrap();
                std::time::SystemTime::now()
            };
            let thread2 = async {
                conn2.batch_execute("BEGIN").await.unwrap();
                // Wait for the first transaction to start and acquire the lock
                tokio::time::sleep(Duration::from_millis(50)).await;
                conn2.batch_execute(
                    "SELECT nextval_with_xact_lock('my_seq'::regclass)"
                ).await.unwrap();
                conn2.batch_execute("COMMIT").await.unwrap();
                std::time::SystemTime::now()
            };

            let (val1, val2) = tokio::join!(thread1, thread2);
            assert!(val2 < val1, "{:?} must be less than {:?}", val2, val1);
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
        async fn it_issues_next_sequence_value() {
            setup().await;
            let conn1 = support::connect().await;
            let conn2 = support::connect().await;

            let thread1 = async {
                conn1.batch_execute("BEGIN").await.unwrap();
                conn1.batch_execute("SELECT pg_advisory_xact_lock(1);").await.unwrap();
                tokio::time::sleep(Duration::from_millis(100)).await;
                conn1.batch_execute("COMMIT").await.unwrap();
                -1
            };
            let thread2 = async {
                conn2.batch_execute("BEGIN").await.unwrap();
                // Wait for the first transaction to start and acquire the lock
                tokio::time::sleep(Duration::from_millis(50)).await;
                let v: i64 = conn2
                    .query_one("SELECT nextval_with_xact_lock('my_seq'::regclass)", &[])
                    .await
                    .unwrap()
                    .get(0);
                conn2.batch_execute("COMMIT").await.unwrap();
                v
            };

            let (_, next_val) = tokio::join!(thread1, thread2);
            assert_eq!(next_val, 1, "next val is: {}", next_val);
        }
    }

    mod when_transaction_gets_commited {
        use std::time::Duration;
        use super::*;

        #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
        async fn it_continues_the_sequence() {
            setup().await;

            let conn = support::connect().await;
            conn.batch_execute("BEGIN").await.unwrap();
            conn
                .batch_execute("SELECT nextval_with_xact_lock('my_seq'::regclass)")
                .await
                .unwrap();
            conn.batch_execute("COMMIT").await.unwrap();
            let seq: i64 = conn
                .query_one("SELECT nextval_with_xact_lock('my_seq'::regclass)", &[])
                .await
                .unwrap().get(0);

            assert_eq!(seq, 2);
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
        async fn it_produces_advisory_lock_while_transaction_is_in_progress() {
            setup().await;
            let thread1 = async {
                let conn = support::connect().await;
                conn.batch_execute("BEGIN").await.unwrap();
                conn
                    .batch_execute("SELECT nextval_with_xact_lock('my_seq'::regclass)")
                    .await
                    .unwrap();
                tokio::time::sleep(Duration::from_millis(100)).await;
                conn.batch_execute("COMMIT").await.unwrap();
                vec![]
            };

            let thread2 = async {
                tokio::time::sleep(Duration::from_millis(50)).await;
                let query = "\
                SELECT objid::int4 FROM pg_locks WHERE locktype = 'advisory' AND objsubid = 1 \
                ORDER BY objid ASC\
            ";
                let res: Vec<i32> = support::connect().await
                    .query(query, &[])
                    .await
                    .unwrap().iter().map(|row| row.get(0)).collect();
                res
            };

            let thread3 = async {
                tokio::time::sleep(Duration::from_millis(150)).await;
                let query = "\
                SELECT objid::int4 FROM pg_locks WHERE locktype = 'advisory' AND objsubid = 1 \
                ORDER BY objid ASC\
            ";
                let res: Vec<i32> = support::connect().await
                    .query(query, &[])
                    .await
                    .unwrap().iter().map(|row| row.get(0)).collect();
                res
            };

            let (_, during_trx, after_trx): (Vec<i32>, _, _) =
                tokio::join!(thread1, thread2, thread3);
            assert_eq!(during_trx, [1]);
            assert_eq!(after_trx, []);
        }
    }

    mod when_transaction_gets_rolled_back {
        use std::time::Duration;
        use super::*;

        #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
        async fn it_continues_the_sequence() {
            setup().await;

            let conn = support::connect().await;
            conn.batch_execute("BEGIN").await.unwrap();
            conn
                .batch_execute("SELECT nextval_with_xact_lock('my_seq'::regclass)")
                .await
                .unwrap();
            conn.batch_execute("ROLLBACK").await.unwrap();
            let seq: i64 = conn
                .query_one("SELECT nextval_with_xact_lock('my_seq'::regclass)", &[])
                .await
                .unwrap().get(0);

            assert_eq!(seq, 2);
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
        async fn it_produces_advisory_lock_while_transaction_is_in_progress() {
            setup().await;

            let thread1 = async {
                let conn = support::connect().await;
                conn.batch_execute("BEGIN").await.unwrap();
                conn
                    .query_one("SELECT nextval_with_xact_lock('my_seq'::regclass)", &[])
                    .await
                    .unwrap();
                tokio::time::sleep(Duration::from_millis(100)).await;
                conn.batch_execute("ROLLBACK").await.unwrap();
                vec![]
            };

            let thread2 = async {
                tokio::time::sleep(Duration::from_millis(50)).await;
                let query = "\
                SELECT objid::int4 FROM pg_locks WHERE locktype = 'advisory' AND objsubid = 1 \
                ORDER BY objid ASC\
            ";
                let res: Vec<i32> = support::connect().await
                    .query(query, &[])
                    .await
                    .unwrap().iter().map(|row| row.get(0)).collect();
                res
            };

            let thread3 = async {
                tokio::time::sleep(Duration::from_millis(150)).await;
                let query = "\
                SELECT objid::int4 FROM pg_locks WHERE locktype = 'advisory' AND objsubid = 1 \
                ORDER BY objid ASC\
            ";
                let res: Vec<i32> = support::connect().await
                    .query(query, &[])
                    .await
                    .unwrap().iter().map(|row| row.get(0)).collect();
                res
            };

            let (_, during_trx, after_trx): (Vec<i32>, _, _) =
                tokio::join!(thread1, thread2, thread3);
            assert_eq!(during_trx, [1]);
            assert_eq!(after_trx, []);
        }
    }

    mod when_running_outside_the_transaction {
        use super::*;

        #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
        async fn it_assigns_nextval_correctly() {
            setup().await;
            let conn = support::connect().await;

            let seq: i64 = conn
                .query_one("SELECT nextval_with_xact_lock('my_seq'::regclass)", &[])
                .await
                .unwrap()
                .get(0);
            assert_eq!(seq, 1);
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
        async fn it_releases_advisory_lock_immediately() {
            setup().await;
            let conn = support::connect().await;

            conn
                .query_one("SELECT nextval_with_xact_lock('my_seq'::regclass)", &[])
                .await
                .unwrap();
            let locks_count: i64 = conn
                .query_one(
                    "SELECT count(*) FROM pg_locks WHERE locktype = 'advisory' AND objsubid = 1",
                    &[]
                )
                .await
                .unwrap()
                .get(0);
            assert_eq!(locks_count, 0);
        }
    }

    mod when_retrieving_nextval_in_read_only_transaction {
        use std::error::Error;
        use super::*;

        #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
        async fn it_returns_error() {
            setup().await;
            let conn = support::connect().await;
            conn.batch_execute("START TRANSACTION READ ONLY;").await.unwrap();
            let res = conn
                .query_one("SELECT nextval_with_xact_lock('my_seq'::regclass)", &[])
                .await;
            conn.batch_execute("ROLLBACK").await.unwrap();
            match res {
                Ok(row) => {
                    panic!(
                        "Sequence must not proceed at all. Got {} instead!",
                        row.get::<usize, i64>(0)
                    )
                }
                Err(err) => {
                    assert_eq!(
                        err.source().unwrap().to_string(),
                        "ERROR: cannot execute nextval_with_xact_lock() in a read-only transaction"
                    );
                }
            }
        }
    }
}
