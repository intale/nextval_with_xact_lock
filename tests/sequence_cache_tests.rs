#[path="support/mod.rs"]
mod support;

mod sequence_cache_tests {
    use super::*;

    async fn setup() {
        support::ensure_db_ready().await;
        let conn = support::connect().await;
        conn.batch_execute("DROP SEQUENCE IF EXISTS my_seq;").await.unwrap();
        conn.batch_execute("CREATE SEQUENCE my_seq CACHE 10;").await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn it_reserves_cache_number_of_values_in_memory() {
        setup().await;
        let conn = support::connect().await;
        let mut seq: Vec<i64> = vec![];

        seq.push(
            conn
                .query_one("SELECT nextval_with_xact_lock('my_seq'::regclass)", &[])
                .await
                .unwrap()
                .get(0)
        );
        seq.push(
            conn
                .query_one("SELECT nextval_with_xact_lock('my_seq'::regclass)", &[])
                .await
                .unwrap()
                .get(0)
        );
        seq.push(
            conn
                .query_one("SELECT nextval_with_xact_lock('my_seq'::regclass)", &[])
                .await
                .unwrap()
                .get(0)
        );
        let persisted_tuple =
            conn
            .query_one("SELECT * FROM pg_get_sequence_data('my_seq');", &[])
            .await
            .unwrap();

        assert_eq!(seq, vec![1, 2, 3], "seq is {:?}", seq);
        assert_eq!(
            persisted_tuple.get::<&str, i64>("last_value"), 10,
            "Last_value is {}", persisted_tuple.get::<&str, i64>("last_value")
        );
        assert!(persisted_tuple.get::<&str, bool>("is_called"));
    }
}
