use std::path::PathBuf;
use rusqlite::{Connection, Params, Statement};
use crate::database::models::User;
use crate::environment::CONFIG;
use crate::error::Error;

pub fn open_sqlite_db_connection() -> Connection {
    let db_path = {
        let config = CONFIG.lock().unwrap().clone();
        config.sqlite
            .expect("There is no configuration for sqlite in config.toml")
            .file.parse::<PathBuf>()
    }.expect("Invalid path in section 'sqlite.file'");

    let mut connection = Connection::open(&db_path)
        .expect("Can't open sqlite database");

    create_database_schema(&mut connection);

    connection
}

fn create_database_schema(connection: &mut Connection) {
    connection.execute(
        "CREATE TABLE IF NOT EXISTS user (
            id          INTEGER PRIMARY KEY,
            username    TEXT NOT NULL,
            password    TEXT NOT NULL,
            email       TEXT NOT NULL,
            created_at  TEXT NOT NULL
            )",
        ()
    ).expect("Can't create user table schema for sqlite database");
}

pub struct SqliteDatabaseHandler<'a>(&'a Connection);

impl<'a> SqliteDatabaseHandler<'a> {
    pub fn new(connection: &'a Connection) -> Self {
        SqliteDatabaseHandler(connection)
    }

    pub fn find_user_by_id(&self, id: usize) -> Option<Result<User, Error>> {
        self.first_user_of_statement(
            "SELECT id, email, username, password, created_at FROM user WHERE id = ?",
            [&id],
        )
    }

    pub fn find_user_by_name(&self, name: &str) -> Option<Result<User, Error>> {
        self.first_user_of_statement(
            "SELECT id, email, username, password, created_at FROM user WHERE username = ?",
            [&name],
        )
    }

    pub fn find_user_by_email(&self, email: &str) -> Option<Result<User, Error>> {
        self.first_user_of_statement(
            "SELECT id, email, username, password, created_at FROM user WHERE email = ?",
            [&email],
        )
    }

    fn first_user_of_statement<P: Params>(&self, sql: &str, params: P) -> Option<Result<User, Error>> {
        let mut statement = match self.0.prepare(sql) {
            Ok(value) => value,
            Err(_) => return None,
        };

        let mut items = Self::user_iter_of(params, &mut statement);
        return if items.len() > 0 {
            Some(items.swap_remove(0))
        } else {
            None
        }
    }

    fn user_iter_of<P: Params>(params: P, statement: &mut Statement)
        -> Vec<Result<User, Error>>
    {
        statement.query_map(params, |row| {
            Ok(User {
                id: row.get(0)?,
                email: row.get(1)?,
                username: row.get(2)?,
                password: row.get(3)?,
                created_at: row.get(4)?,
            })
        }).unwrap().map(|v| {
            match v {
                Ok(user) => Ok(user),
                Err(e) => Err(Error::SqliteError(e))
            }
        }).collect()
    }

    pub fn create_user(&self, user: &User) -> Result<usize, Error> {
        if self.user_already_exists(&user)? {
            return Err(Error::UserAlreadyExists);
        }

        Ok(
            self.0.execute(
                "INSERT INTO user(email, username, password, created_at) VALUES (?1, ?2, ?3, ?4)",
                (&user.email, &user.username, &user.password, &user.created_at),
            )?
        )
    }

    fn user_already_exists(&self, user: &User) -> Result<bool, Error> {
        let mut statement = self.0.prepare("SELECT id FROM user WHERE email = ?1 OR username = ?2")?;
        Ok(statement.exists([&user.email, &user.username])?)
    }
}
