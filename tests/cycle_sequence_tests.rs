#[path="support/mod.rs"]
mod support;

mod cycle_sequence_tests {
    use super::*;

    mod when_increment_is_positive {
        use super::*;

        async fn setup() {
            support::ensure_db_ready().await;
            let conn = support::connect().await;
            conn.batch_execute("DROP SEQUENCE IF EXISTS my_seq;").await.unwrap();
            conn.batch_execute("CREATE SEQUENCE my_seq INCREMENT BY 5 MAXVALUE 10 CYCLE;").await.unwrap();
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
        async fn it_restarts_sequence_after_reaching_max_value() {
            setup().await;
            let conn = support::connect().await;
            let mut seq: Vec<i64> = vec![];

            seq.push(
                conn
                    .query_one("SELECT pg_nextval_with_xact_lock('my_seq'::regclass)", &[])
                    .await
                    .unwrap()
                    .get(0)
            );
            seq.push(
                conn
                    .query_one("SELECT pg_nextval_with_xact_lock('my_seq'::regclass)", &[])
                    .await
                    .unwrap()
                    .get(0)
            );
            seq.push(
                conn
                    .query_one("SELECT pg_nextval_with_xact_lock('my_seq'::regclass)", &[])
                    .await
                    .unwrap()
                    .get(0)
            );
            assert_eq!(seq, vec![1, 6, 1]);
        }
    }

    mod when_increment_is_negative {
        use super::*;

        async fn setup() {
            support::ensure_db_ready().await;
            let conn = support::connect().await;
            conn.batch_execute("DROP SEQUENCE IF EXISTS my_seq;").await.unwrap();
            conn.batch_execute("CREATE SEQUENCE my_seq INCREMENT BY -5 MINVALUE -10 CYCLE;").await.unwrap();
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
        async fn it_restarts_sequence_after_reaching_min_value() {
            setup().await;
            let conn = support::connect().await;
            let mut seq: Vec<i64> = vec![];

            seq.push(
                conn
                    .query_one("SELECT pg_nextval_with_xact_lock('my_seq'::regclass)", &[])
                    .await
                    .unwrap()
                    .get(0)
            );
            seq.push(
                conn
                    .query_one("SELECT pg_nextval_with_xact_lock('my_seq'::regclass)", &[])
                    .await
                    .unwrap()
                    .get(0)
            );
            seq.push(
                conn
                    .query_one("SELECT pg_nextval_with_xact_lock('my_seq'::regclass)", &[])
                    .await
                    .unwrap()
                    .get(0)
            );
            assert_eq!(seq, vec![-1, -6, -1]);
        }
    }
}
