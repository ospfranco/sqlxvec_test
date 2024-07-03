use serde::{ser::Serializer, Serialize};
use serde_json::Value as JsonValue;
use sqlite_vec::sqlite3_vec_init;
use sqlx::{Column, Row};
use sqlx::{Connection, SqliteConnection};
use std::collections::HashMap;

use crate::db::decode;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Sql(#[from] sqlx::Error),
    #[error(transparent)]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error("unsupported datatype: {0}")]
    UnsupportedDatatype(String),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

type Result<T> = std::result::Result<T, Error>;

pub async fn run_test() -> Result<()> {
    println!("started run_test");
    unsafe {
        libsqlite3_sys::sqlite3_auto_extension(Some(std::mem::transmute(
            sqlite3_vec_init as *const (),
        )));
    }

    let mut db =
        SqliteConnection::connect("sqlite:/Users/osp/Developer/sqlxtest/mydb.sqlite").await?;

    // let query = "select sqlite_version(), vec_version(), vec_to_json(?)";
    let query = "select sqlite_version(), vec_version()";
    let query = sqlx::query(&query);

    let rows = query.fetch_all(&mut db).await?;
    let mut values = Vec::new();
    for row in rows {
        let mut value: HashMap<String, JsonValue> = HashMap::default();
        for (i, column) in row.columns().iter().enumerate() {
            let v = row.try_get_raw(i)?;
            let v = decode::to_json(v)?;

            value.insert(column.name().to_string(), v);
        }

        values.push(value);
    }

    log::info!("versions: {:?}", values);

    // Create a sample table and add some embeddings into it
    let query = sqlx::query("DROP TABLE IF EXISTS vec_items");
    query.execute(&mut db).await?;
    log::info!("table dropped!");
    let query = sqlx::query("CREATE VIRTUAL TABLE vec_items USING vec0(embedding float[4])");
    query.execute(&mut db).await?;
    log::info!("table created!");

    let items: Vec<(i64, Vec<f32>)> = vec![
        (1, vec![0.1, 0.1, 0.1, 0.1]),
        (2, vec![0.2, 0.2, 0.2, 0.2]),
        (3, vec![0.3, 0.3, 0.3, 0.3]),
        (4, vec![0.4, 0.4, 0.4, 0.4]),
        (5, vec![0.5, 0.5, 0.5, 0.5]),
        (6, vec![0.6, 0.6, 0.6, 0.6]),
        (7, vec![0.7, 0.7, 0.7, 0.7]),
        (8, vec![0.8, 0.8, 0.8, 0.8]),
        (9, vec![0.9, 0.9, 0.9, 0.9]),
        (10, vec![0.10, 0.10, 0.10, 0.10]),
    ];

    for (id, embedding) in &items {
        sqlx::query("BEGIN").execute(&mut db).await?;
        let mut query = sqlx::query("INSERT INTO vec_items(rowid, embedding) VALUES (?, ?)");
        query = query.bind(id);
        let embedding_json = serde_json::to_string(embedding).unwrap();
        query = query.bind(embedding_json);
        query.execute(&mut db).await?;
        log::info!("inserted item {}", id);
        sqlx::query("COMMIT").execute(&mut db).await?;
    }

    // let query_values: Vec<f32> = vec![0.3, 0.3, 0.3, 0.3];
    // let mut query = sqlx::query(
    //     "SELECT rowid, distance FROM vec_items WHERE embedding MATCH ?1 ORDER BY distance LIMIT 3",
    // );
    let query = sqlx::query("SELECT rowid, distance FROM vec_items");
    // query = query.bind(serde_json::to_string(&query_values).unwrap());

    let rows = query.fetch_all(&mut db).await?;

    log::info!("query rows length {:?}", rows.len());

    let mut values = Vec::new();
    for row in rows {
        let mut value: HashMap<String, JsonValue> = HashMap::default();
        for (i, column) in row.columns().iter().enumerate() {
            let v = row.try_get_raw(i)?;
            let v = decode::to_json(v)?;

            value.insert(column.name().to_string(), v);
        }

        values.push(value);
    }

    log::info!("query result: {:?}", values);

    Ok(())
}
