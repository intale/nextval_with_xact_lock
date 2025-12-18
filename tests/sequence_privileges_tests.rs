#[path="support/mod.rs"]
mod support;

mod sequence_privileges_tests {
    use std::error::Error;
    use super::*;

    async fn setup() {
        support::ensure_db_ready().await;
        let conn = support::connect().await;
        conn.batch_execute("DROP SEQUENCE IF EXISTS my_seq;").await.unwrap();
        conn.batch_execute("DROP USER IF EXISTS seq_user;").await.unwrap();
        conn.batch_execute("CREATE USER seq_user;").await.unwrap();
        conn.batch_execute("CREATE SEQUENCE my_seq;").await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn it_allows_nextval_for_update_privilege() {
        setup().await;
        let conn = support::connect().await;
        conn.batch_execute("BEGIN;").await.unwrap();
        conn.batch_execute("REVOKE ALL ON my_seq FROM seq_user;").await.unwrap();
        conn.batch_execute("GRANT UPDATE ON my_seq TO seq_user;").await.unwrap();
        conn.batch_execute("SET LOCAL SESSION AUTHORIZATION seq_user;").await.unwrap();
        let res = conn.query_one(
            "SELECT pg_nextval_with_xact_lock('my_seq'::regclass);", &[]
        ).await;
        conn.batch_execute("ROLLBACK;").await.unwrap();

        match res {
            Ok(row) => {
                assert_eq!(row.get::<usize, i64>(0), 1);
            }
            Err(err) => {
                panic!(
                    "Expected UPDATE privilege to allow pg_nextval_with_xact_lock() but it didn't. \
                    Got error instead: {}",
                    err
                );
            }
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn it_allows_nextval_for_usage_privilege() {
        setup().await;
        let conn = support::connect().await;
        conn.batch_execute("BEGIN;").await.unwrap();
        conn.batch_execute("REVOKE ALL ON my_seq FROM seq_user;").await.unwrap();
        conn.batch_execute("GRANT USAGE ON my_seq TO seq_user;").await.unwrap();
        conn.batch_execute("SET LOCAL SESSION AUTHORIZATION seq_user;").await.unwrap();
        let res = conn.query_one(
            "SELECT pg_nextval_with_xact_lock('my_seq'::regclass);", &[]
        ).await;
        conn.batch_execute("ROLLBACK;").await.unwrap();

        match res {
            Ok(row) => {
                assert_eq!(row.get::<usize, i64>(0), 1);
            }
            Err(err) => {
                panic!(
                    "Expected USAGE privilege to allow pg_nextval_with_xact_lock() but it didn't. \
                    Got error instead: {}",
                    err
                );
            }
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn it_disallows_nextval_for_select_privilege() {
        setup().await;
        let conn = support::connect().await;
        conn.batch_execute("BEGIN;").await.unwrap();
        conn.batch_execute("REVOKE ALL ON my_seq FROM seq_user;").await.unwrap();
        conn.batch_execute("GRANT SELECT ON my_seq TO seq_user;").await.unwrap();
        conn.batch_execute("SET LOCAL SESSION AUTHORIZATION seq_user;").await.unwrap();
        let res = conn.query_one(
            "SELECT pg_nextval_with_xact_lock('my_seq'::regclass);", &[]
        ).await;
        conn.batch_execute("ROLLBACK;").await.unwrap();

        match res {
            Ok(row) => {
                panic!(
                    "User should not be able to grab nextval of the sequence with SELECT \
                    privilege. Got {} instead!",
                    row.get::<usize, i64>(0)
                );
            }
            Err(err) => {
                assert!(
                    err
                        .source()
                        .unwrap()
                        .to_string()
                        .contains(
                            "permission denied for sequence"
                        )
                );
            }
        }
    }
}
