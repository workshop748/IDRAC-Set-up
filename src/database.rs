use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Result;
use bcrypt::{hash, verify, DEFAULT_COST};
use log::info;

#[derive(Debug, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
}

pub type DbPool = Pool<SqliteConnectionManager>;

pub struct Database {
    pool: DbPool,
}

impl Database {
    pub fn new(db_path: &str) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = std::path::Path::new(db_path).parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        }

        // Create the database file if it doesn't exist by opening a connection first
        {
            let _conn = rusqlite::Connection::open(db_path)?;
            info!("Database file created/verified at {}", db_path);
        }

        let manager = SqliteConnectionManager::file(db_path);
        let pool = Pool::new(manager)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        
        // Initialize schema
        let conn = pool.get()
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        
        info!("Database initialized at {}", db_path);
        
        let db = Database { pool };
        
        // Create default admin account if no users exist
        if !db.has_users()? {
            info!("No users found, creating default admin account");
            db.create_user("admin", "")?;
            info!("Default admin account created (username: admin)");
        }
        
        Ok(db)
    }

    pub fn has_users(&self) -> Result<bool> {
        let conn = self.pool.get()
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM users",
            [],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn create_user(&self, username: &str, password: &str) -> Result<i64> {
        let password_hash = hash(password, DEFAULT_COST)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        
        let conn = self.pool.get()
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        
        conn.execute(
            "INSERT INTO users (username, password_hash) VALUES (?1, ?2)",
            [username, &password_hash],
        )?;
        
        info!("User created: {}", username);
        Ok(conn.last_insert_rowid())
    }

    pub fn verify_user(&self, username: &str, password: &str) -> Result<Option<User>> {
        let conn = self.pool.get()
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        
        let mut stmt = conn.prepare(
            "SELECT id, username, password_hash FROM users WHERE username = ?1"
        )?;
        
        let user = stmt.query_row([username], |row| {
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                password_hash: row.get(2)?,
            })
        });

        match user {
            Ok(user) => {
                let valid = verify(password, &user.password_hash)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
                
                if valid {
                    info!("User authenticated: {}", username);
                    Ok(Some(user))
                } else {
                    Ok(None)
                }
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    #[allow(dead_code)]
    pub fn get_user_by_id(&self, user_id: i64) -> Result<Option<User>> {
        let conn = self.pool.get()
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        
        let mut stmt = conn.prepare(
            "SELECT id, username, password_hash FROM users WHERE id = ?1"
        )?;
        
        let user = stmt.query_row([user_id], |row| {
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                password_hash: row.get(2)?,
            })
        });

        match user {
            Ok(user) => Ok(Some(user)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
