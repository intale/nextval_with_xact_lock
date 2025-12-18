use once_cell::sync::OnceCell;
use tokio_postgres::{Client, NoTls};

static DB_READY: OnceCell<()> = OnceCell::new();

pub async fn connect() -> Client {
    let home = std::env::var("HOME").unwrap();
    let pg_version = std::env::var("PGVER").expect("PGVER env var must be set!");
    let url = format!("postgres:///postgres?host={home}/.pgrx&port=288{}", pg_version);

    let (client, connection) = tokio_postgres::connect(&url, NoTls)
        .await
        .expect(format!("Failed to connect to PostgtrSQL by {url}").as_str());

    tokio::spawn(async move {
        // Keep connection management in background
        if let Err(e) = connection.await {
            eprintln!("Connection error: {e}");
        }
    });

    client
}

pub async fn ensure_db_ready() {
    if DB_READY.get().is_some() {
        return;
    }

    let c = connect().await;

    c.batch_execute("CREATE EXTENSION IF NOT EXISTS nextval_with_xact_lock;")
        .await
        .unwrap();
    c.batch_execute("ALTER SYSTEM SET log_statement = 'all';")
        .await
        .unwrap();
    c.batch_execute("SELECT pg_reload_conf();")
        .await
        .unwrap();

    let _ = DB_READY.set(());
}
