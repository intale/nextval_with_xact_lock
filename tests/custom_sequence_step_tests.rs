#[path="support/mod.rs"]
mod support;

mod custom_sequence_step_tests {
    use super::*;

    mod when_step_is_positive {
        use super::*;

        async fn setup() {
            support::ensure_db_ready().await;
            let conn = support::connect().await;
            conn.batch_execute("DROP SEQUENCE IF EXISTS my_seq;").await.unwrap();
            conn.batch_execute("CREATE SEQUENCE my_seq INCREMENT BY 5;").await.unwrap();
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
        async fn it_respects_increment_by_setting() {
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
            assert_eq!(seq, vec![1, 6]);
        }
    }

    mod when_step_is_negative {
        use super::*;

        async fn setup() {
            support::ensure_db_ready().await;
            let conn = support::connect().await;
            conn.batch_execute("DROP SEQUENCE IF EXISTS my_seq;").await.unwrap();
            conn.batch_execute("CREATE SEQUENCE my_seq INCREMENT BY -5;").await.unwrap();
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
        async fn it_respects_increment_by_setting() {
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
            assert_eq!(seq, vec![-1, -6]);
        }
    }
}
