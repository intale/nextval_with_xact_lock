#[path="support/mod.rs"]
mod support;

mod custom_sequence_start_value_tests {
    use super::*;

    mod when_value_is_positive {
        use super::*;

        async fn setup() {
            support::ensure_db_ready().await;
            let conn = support::connect().await;
            conn.batch_execute("DROP SEQUENCE IF EXISTS my_seq;").await.unwrap();
            conn.batch_execute("CREATE SEQUENCE my_seq START WITH 5;").await.unwrap();
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
        async fn it_respects_start_with_setting() {
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
            assert_eq!(seq, vec![5, 6]);
        }
    }

    mod when_value_is_negative {
        use super::*;

        async fn setup() {
            support::ensure_db_ready().await;
            let conn = support::connect().await;
            conn.batch_execute("DROP SEQUENCE IF EXISTS my_seq;").await.unwrap();
            conn.batch_execute("CREATE SEQUENCE my_seq MINVALUE -5 START WITH -5;").await.unwrap();
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
        async fn it_respects_start_with_setting() {
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
            assert_eq!(seq, vec![-5, -4]);
        }
    }
}
