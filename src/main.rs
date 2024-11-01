use std::error::Error;
use std::sync::Arc;

use tokio_rusqlite::{params, Connection, Result as SQLResult};
use uuid::Uuid;
use bcrypt;
use jwt;
use poem::{get, post, handler, listener::TcpListener, web::Path, web::Json, web::Data, IntoResponse, Route, Server, EndpointExt, middleware::AddData, http::StatusCode};
// use tokio::sync::RwLock;
use tokio::sync::Mutex;
use serde::Deserialize;

type Result<T, E = Box<dyn Error>> = std::result::Result<T, E>;
type DbMutex = Arc<Mutex<Connection>>;

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
async fn signup(Data(db_lock): Data<&DbMutex>, Json(user): Json<UserLogin>) -> (StatusCode, String) {
    let mut db = db_lock.lock().await;
    let result = create_account(&db, &user.user, &user.password).await;
    match result {
        Ok(id) => (StatusCode::OK, id),
        Err(_) => (StatusCode::CONFLICT, "user already exists".to_owned()),
    }
}


#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    println!("tors backend starting");
    let db = Connection::open("./db.sqlite").await.expect("failed to connect to database");
    create_tables(&db).await.expect("failed to create tables");

    db.call(|db| {
        db.query_row("SELECT 1 FROM keys", [], |row| Ok(()))
     });
    let key_pair = ES384KeyPair::generate();

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
            CREATE TABLE IF NOT EXISTS keys (private BLOB, public BLOB);
            COMMIT;"
        )?;
        Ok(())
    }).await
}

async fn create_account(db: &Connection, username: &str, password: &str) -> Result<String> {
    if !db.call({
            let username = username.to_string();
            move |db| {
                Ok(db.prepare("SELECT 1 FROM users WHERE username = ?")?.query(params![username])?.next()?.is_none())
            }
        }).await.unwrap() {
        return Err(SignupError.into());
    }

    let id = Uuid::new_v4().to_string();
    let salted_hash = bcrypt::hash_with_result(password, bcrypt::DEFAULT_COST)?;
    db.call({
        let id = id.clone();
        let username = username.to_string();
        move |db| {
            db.execute("INSERT INTO users VALUES (?, ?)", params![id, username])?;
            db.execute("INSERT INTO login VALUES (?, ?)", params![id, salted_hash.format_for_version(bcrypt::Version::TwoB)])?;
        Ok(())
    }}).await?;

    Ok(id)
}

fn generate_auth_token_db(db: &Connection, id: &str) {
    let priv_key = db.call(|db| {

    });
}

fn generate_auth_token(priv_key: &str, id: &str) {}
