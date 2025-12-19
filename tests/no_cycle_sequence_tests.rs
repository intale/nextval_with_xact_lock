#[path="support/mod.rs"]
mod support;

mod no_cycle_sequence_tests {
    use super::*;
    use std::error::Error;

    mod when_increment_is_positive {
        use super::*;

        async fn setup() {
            support::ensure_db_ready().await;
            let conn = support::connect().await;
            conn.batch_execute("DROP SEQUENCE IF EXISTS my_seq;")
                .await
                .unwrap();
            conn.batch_execute("CREATE SEQUENCE my_seq INCREMENT BY 5 MAXVALUE 10 NO CYCLE;")
                .await
                .unwrap();
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
        async fn it_returns_error_after_reaching_max_value() {
            setup().await;
            let conn = support::connect().await;
            let mut seq: Vec<i64> = vec![];

            seq.push(
                conn.query_one("SELECT nextval_with_xact_lock('my_seq'::regclass)", &[])
                    .await
                    .unwrap()
                    .get(0),
            );
            seq.push(
                conn.query_one("SELECT nextval_with_xact_lock('my_seq'::regclass)", &[])
                    .await
                    .unwrap()
                    .get(0),
            );
            let res = conn
                .query_one("SELECT nextval_with_xact_lock('my_seq'::regclass)", &[])
                .await;

            assert_eq!(seq, vec![1, 6]);
            match res {
                Ok(row) => {
                    panic!(
                        "Sequence must not proceed further. Got {} instead!",
                        row.get::<usize, i64>(0)
                    )
                }
                Err(err) => {
                    assert!(
                        err
                            .source()
                            .unwrap()
                            .to_string()
                            .contains(
                                "nextval_with_xact_lock: reached maximum value of sequence"
                            )
                    );
                }
            }
        }
    }

    mod when_increment_is_negative {
        use super::*;

        async fn setup() {
            support::ensure_db_ready().await;
            let conn = support::connect().await;
            conn.batch_execute("DROP SEQUENCE IF EXISTS my_seq;")
                .await
                .unwrap();
            conn.batch_execute(
                "CREATE SEQUENCE my_seq INCREMENT BY -5 MINVALUE -10;"
            )
                .await
                .unwrap();
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
        async fn it_returns_error_after_reaching_min_value() {
            setup().await;
            let conn = support::connect().await;
            let mut seq: Vec<i64> = vec![];

            seq.push(
                conn.query_one("SELECT nextval_with_xact_lock('my_seq'::regclass)", &[])
                    .await
                    .unwrap()
                    .get(0),
            );
            seq.push(
                conn.query_one("SELECT nextval_with_xact_lock('my_seq'::regclass)", &[])
                    .await
                    .unwrap()
                    .get(0),
            );

            let res = conn
                .query_one("SELECT nextval_with_xact_lock('my_seq'::regclass)", &[])
                .await;

            assert_eq!(seq, vec![-1, -6]);
            match res {
                Ok(row) => {
                    panic!(
                        "Sequence must not proceed further. Got {} instead!",
                        row.get::<usize, i64>(0)
                    )
                }
                Err(err) => {
                    assert!(
                        err
                            .source()
                            .unwrap()
                            .to_string()
                            .contains(
                                "nextval_with_xact_lock: reached minimum value of sequence"
                            )
                    );
                }
            }
        }
    }
}
