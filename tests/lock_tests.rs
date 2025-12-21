#[path="support/mod.rs"]
mod support;

mod lock_tests {
    use std::time::Duration;
    use crate::support;
    use chrono::{Local};

    async fn setup() {
        support::ensure_db_ready().await;
        let conn = support::connect().await;
        conn.batch_execute("DROP SEQUENCE IF EXISTS my_seq;").await.unwrap();
        conn.batch_execute("DROP USER IF EXISTS lock_tests_thread1;").await.unwrap();
        conn.batch_execute("CREATE USER lock_tests_thread1;").await.unwrap();
        conn.batch_execute("DROP USER IF EXISTS lock_tests_thread2;").await.unwrap();
        conn.batch_execute("CREATE USER lock_tests_thread2;").await.unwrap();
        conn.batch_execute("CREATE SEQUENCE my_seq;").await.unwrap();
        conn.batch_execute("GRANT ALL ON my_seq TO lock_tests_thread1;").await.unwrap();
        conn.batch_execute("GRANT ALL ON my_seq TO lock_tests_thread2;").await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn it_does_not_issue_next_value_until_another_value_being_issued_at_the_moment() {
        setup().await;
        let conn1 = support::connect().await;
        let conn2 = support::connect().await;
        let start_time = Local::now();

        let thread1 = async {
            conn1.batch_execute("BEGIN;").await.unwrap();
            conn1.batch_execute("SET LOCAL SESSION AUTHORIZATION lock_tests_thread1;").await.unwrap();
            let seq = conn1.query_one(
                "SELECT nextval_with_xact_lock('my_seq'::regclass) as val;",
                &[]
            ).await.unwrap().get::<&str, i64>("val");
            conn1.batch_execute("COMMIT;").await.unwrap();
            (seq, Local::now())
        };
        let thread2 = async {
            conn2.batch_execute("BEGIN;").await.unwrap();
            conn2.batch_execute("SET LOCAL SESSION AUTHORIZATION lock_tests_thread2;").await.unwrap();
            // Let first thread to start first
            tokio::time::sleep(Duration::from_millis(20)).await;
            let seq = conn2.query_one(
                "SELECT nextval_with_xact_lock('my_seq'::regclass) as val;",
                &[]
            ).await.unwrap().get::<&str, i64>("val");
            conn2.batch_execute("COMMIT;").await.unwrap();
            (seq, Local::now())
        };

        let ((seq1, time1), (seq2, time2)) = tokio::join!(thread1, thread2);
        let end_time = Local::now();

        assert_eq!(seq2 - seq1, 1);
        assert!(
            (time1 - time2).num_milliseconds().abs() < 50,
            "Expected second nextval_with_xact_lock() to wait for the first one, but it didn't. \
            Time between them is {}ms, but should be closer to 0 instead.",
            (time1 - time2).num_milliseconds().abs()
        );
        assert!(
            (end_time - start_time).num_milliseconds() > 1000,
            "Execution took less than a second, but grab_advisory_lock() for test env should take \
            1 second. It seems the extension was installed without lock_tests feature."
        );
    }
}
