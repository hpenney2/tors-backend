use std::error::Error;
use std::sync::{Arc, Mutex};

use tokio_rusqlite::{params, Connection, Result as SQLResult};
use uuid::Uuid;
use bcrypt;
use poem::{get, post, handler, listener::TcpListener, web::Path, web::Json, web::Data, IntoResponse, Route, Server, EndpointExt, middleware::AddData};
// use tokio::sync::RwLock;
use serde::Deserialize;

type Result<T, E = Box<dyn Error>> = std::result::Result<T, E>;
type DbData<'a> = Data<&'a Arc<Mutex<Connection>>>;

#[derive(Debug, Clone)]
struct SignupError;

impl std::fmt::Display for SignupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "account creation failed")
    }
}

impl Error for SignupError {}

#[derive(Deserialize)]
struct UserLogin {
    user: String,
    password: String,
}

#[handler]
fn signup(Data(dbLock): DbData, Json(user): Json<UserLogin>) {
    let mut db = dbLock.lock().unwrap();
    create_account(&db, &user.user, &user.password);
}


#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    println!("tors backend starting");
    let db = Connection::open("./db.sqlite").await.expect("failed to connect to database");
    create_tables(&db).await.expect("failed to create tables");

    let app = Route::new()
        .at("/newaccount", post(signup))
        .with(AddData::new(Arc::new(Mutex::new(db))));
    Server::new(TcpListener::bind("0.0.0.0:3000")).run(app).await
}

async fn create_tables(db: &Connection) -> SQLResult<()> {
    db.call(|db| {
        db.execute_batch(
            "BEGIN;
            CREATE TABLE IF NOT EXISTS users (id TEXT NOT NULL UNIQUE, username TEXT PRIMARY KEY);
            CREATE TABLE IF NOT EXISTS login (id TEXT PRIMARY KEY, passHash TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS todos (id TEXT PRIMARY KEY, task TEXT NOT NULL, dateCreated TEXT NOT NULL);
            COMMIT;"
        )?;
        Ok(())
    }).await
}

async fn create_account(db: &Connection, username: &str, password: &str) -> Result<String> {
    if !db.call({
            // let username = username.clone();
            move |db| {
                Ok(db.prepare("SELECT 1 FROM users WHERE username = ?")?.query([username.clone()])?.next()?.is_none())
            }
        }).await.unwrap() {
        return Err(SignupError.into());
    }

    let id = Uuid::new_v4().to_string();
    let salted_hash = bcrypt::hash_with_result(password, bcrypt::DEFAULT_COST)?;
    db.call({
        let id = id.clone();
        let username = username.clone();
        move |db| {
            db.execute("INSERT INTO users VALUES (?, ?)", params![id, username])?;
            db.execute("INSERT INTO login VALUES (?, ?)", params![id, salted_hash.format_for_version(bcrypt::Version::TwoB)])?;
        Ok(())
    }}).await;

    Ok(id)
}

fn generate_auth_token(id: &str) {}
