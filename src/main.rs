use actix_web::{web, App, HttpServer, middleware};
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_session::config::PersistentSession;
use actix_web::cookie::{Key, time::Duration};
use std::sync::Arc;
use env_logger::Env;
use log::info;

mod database;
mod idrac;
mod handlers;

use database::Database;
use idrac::IdracClient;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logger
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    
    info!("Starting iDRAC Controller application");

    // Initialize database
    let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "./data/idrac.db".to_string());
    
    let db = match Database::new(&db_path) {
        Ok(db) => {
            info!("Database initialized successfully");
            Arc::new(db)
        }
        Err(e) => {
            eprintln!("Failed to initialize database: {}", e);
            std::process::exit(1);
        }
    };

    // Initialize iDRAC client
    let idrac_client = match IdracClient::from_env() {
        Ok(client) => {
            info!("iDRAC client initialized successfully");
            Arc::new(client)
        }
        Err(e) => {
            eprintln!("Failed to initialize iDRAC client: {}", e);
            eprintln!("Please ensure IDRAC_HOST, IDRAC_USERNAME, and IDRAC_PASSWORD environment variables are set");
            std::process::exit(1);
        }
    };

    // Generate a secret key for sessions
    let secret_key = Key::generate();
    
    let bind_address = "0.0.0.0:8080";
    info!("Starting HTTP server at {}", bind_address);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(idrac_client.clone()))
            .wrap(middleware::Logger::default())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .session_lifecycle(PersistentSession::default().session_ttl(Duration::hours(24)))
                    .build()
            )
            // Routes
            .route("/", web::get().to(handlers::index))
            .route("/api/register", web::post().to(handlers::register))
            .route("/api/login", web::post().to(handlers::login))
            .route("/api/logout", web::post().to(handlers::logout))
            .route("/api/power/status", web::get().to(handlers::power_status))
            .route("/api/power/on", web::post().to(handlers::power_on_handler))
            .route("/api/power/off", web::post().to(handlers::power_off_handler))
            .route("/api/power/shutdown", web::post().to(handlers::graceful_shutdown_handler))
    })
    .bind(bind_address)?
    .run()
    .await
}
