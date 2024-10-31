use std::error::Error;

use rusqlite::{params, Connection, Result as SQLResult};
use uuid::Uuid;
use bcrypt;

type Result<T, E = Box<dyn Error>> = std::result::Result<T, E>;

fn main() -> Result<()> {
    println!("tors backend starting");
    let db = Connection::open("./db.sqlite")?;
    create_tables(&db).expect("failed to create tables");

    Ok(())
}

fn create_tables(db: &Connection) -> SQLResult<()> {
    db.execute_batch(
        "BEGIN;
        CREATE TABLE IF NOT EXISTS users (id TEXT NOT NULL UNIQUE, username TEXT PRIMARY KEY);
        CREATE TABLE IF NOT EXISTS login (id TEXT PRIMARY KEY, passHash TEXT NOT NULL);
        CREATE TABLE IF NOT EXISTS todos (id TEXT PRIMARY KEY, task TEXT NOT NULL, dateCreated TEXT NOT NULL);
        COMMIT;"
    )
}

fn create_account(db: &Connection, username: &str, password: &str) -> Result<String> {
    let id = Uuid::new_v4().to_string();
    let salted_hash = bcrypt::hash_with_result(password, bcrypt::DEFAULT_COST)?;
    db.execute("INSERT INTO users VALUES (?, ?)", params![id, username])?;
    db.execute("INSERT INTO login VALUES (?, ?)", params![id, salted_hash.format_for_version(bcrypt::Version::TwoB)])?;

    Ok(id)
}
