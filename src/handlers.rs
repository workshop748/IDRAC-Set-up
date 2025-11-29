use actix_web::{web, HttpResponse};
use actix_session::Session;
use serde::{Deserialize, Serialize};
use log::info;
use std::sync::Arc;

use crate::database::Database;
use crate::idrac::IdracClient;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub confirm_password: String,
}

#[derive(Serialize)]
pub struct ApiResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Serialize)]
pub struct StatusResponse {
    pub success: bool,
    pub power_state: String,
}

pub async fn index(session: Session, db: web::Data<Arc<Database>>) -> HttpResponse {
    // Check if user is logged in
    if let Ok(Some(_user_id)) = session.get::<i64>("user_id") {
        HttpResponse::Ok()
            .content_type("text/html")
            .body(include_str!("../static/dashboard.html"))
    } else {
        // Check if any users exist
        match db.has_users() {
            Ok(true) => {
                // Users exist, show login page
                HttpResponse::Ok()
                    .content_type("text/html")
                    .body(include_str!("../static/login.html"))
            }
            Ok(false) => {
                // No users exist, show registration page
                HttpResponse::Ok()
                    .content_type("text/html")
                    .body(include_str!("../static/register.html"))
            }
            Err(e) => {
                HttpResponse::InternalServerError()
                    .body(format!("Database error: {}", e))
            }
        }
    }
}

pub async fn register(
    form: web::Json<RegisterRequest>,
    db: web::Data<Arc<Database>>,
    session: Session,
) -> HttpResponse {
    // Check if users already exist
    match db.has_users() {
        Ok(true) => {
            return HttpResponse::Forbidden().json(ApiResponse {
                success: false,
                message: "Registration is closed. An account already exists.".to_string(),
            });
        }
        Ok(false) => {}
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiResponse {
                success: false,
                message: format!("Database error: {}", e),
            });
        }
    }

    if form.username.trim().is_empty() || form.password.is_empty() {
        return HttpResponse::BadRequest().json(ApiResponse {
            success: false,
            message: "Username and password are required".to_string(),
        });
    }

    if form.password != form.confirm_password {
        return HttpResponse::BadRequest().json(ApiResponse {
            success: false,
            message: "Passwords do not match".to_string(),
        });
    }

    if form.password.len() < 8 {
        return HttpResponse::BadRequest().json(ApiResponse {
            success: false,
            message: "Password must be at least 8 characters".to_string(),
        });
    }

    match db.create_user(&form.username, &form.password) {
        Ok(user_id) => {
            // Auto-login after registration
            let _ = session.insert("user_id", user_id);
            info!("New user registered and logged in: {}", form.username);
            
            HttpResponse::Ok().json(ApiResponse {
                success: true,
                message: "Account created successfully".to_string(),
            })
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse {
            success: false,
            message: format!("Failed to create user: {}", e),
        }),
    }
}

pub async fn login(
    form: web::Json<LoginRequest>,
    db: web::Data<Arc<Database>>,
    session: Session,
) -> HttpResponse {
    if form.username.trim().is_empty() || form.password.is_empty() {
        return HttpResponse::BadRequest().json(ApiResponse {
            success: false,
            message: "Username and password are required".to_string(),
        });
    }

    match db.verify_user(&form.username, &form.password) {
        Ok(Some(user)) => {
            let _ = session.insert("user_id", user.id);
            info!("User logged in: {}", user.username);
            
            HttpResponse::Ok().json(ApiResponse {
                success: true,
                message: "Login successful".to_string(),
            })
        }
        Ok(None) => HttpResponse::Unauthorized().json(ApiResponse {
            success: false,
            message: "Invalid username or password".to_string(),
        }),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse {
            success: false,
            message: format!("Database error: {}", e),
        }),
    }
}

pub async fn logout(session: Session) -> HttpResponse {
    session.purge();
    info!("User logged out");
    
    HttpResponse::Ok().json(ApiResponse {
        success: true,
        message: "Logged out successfully".to_string(),
    })
}

// Middleware to check authentication
pub async fn check_auth(session: Session) -> Result<i64, HttpResponse> {
    match session.get::<i64>("user_id") {
        Ok(Some(user_id)) => Ok(user_id),
        _ => Err(HttpResponse::Unauthorized().json(ApiResponse {
            success: false,
            message: "Not authenticated".to_string(),
        })),
    }
}

pub async fn power_status(
    session: Session,
    idrac: web::Data<Arc<IdracClient>>,
) -> HttpResponse {
    if check_auth(session).await.is_err() {
        return HttpResponse::Unauthorized().json(ApiResponse {
            success: false,
            message: "Not authenticated".to_string(),
        });
    }

    match idrac.get_power_state().await {
        Ok(state) => HttpResponse::Ok().json(StatusResponse {
            success: true,
            power_state: state,
        }),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse {
            success: false,
            message: e,
        }),
    }
}

pub async fn power_on_handler(
    session: Session,
    idrac: web::Data<Arc<IdracClient>>,
) -> HttpResponse {
    if check_auth(session).await.is_err() {
        return HttpResponse::Unauthorized().json(ApiResponse {
            success: false,
            message: "Not authenticated".to_string(),
        });
    }

    match idrac.power_on().await {
        Ok(msg) => HttpResponse::Ok().json(ApiResponse {
            success: true,
            message: msg,
        }),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse {
            success: false,
            message: e,
        }),
    }
}

pub async fn power_off_handler(
    session: Session,
    idrac: web::Data<Arc<IdracClient>>,
) -> HttpResponse {
    if check_auth(session).await.is_err() {
        return HttpResponse::Unauthorized().json(ApiResponse {
            success: false,
            message: "Not authenticated".to_string(),
        });
    }

    match idrac.power_off().await {
        Ok(msg) => HttpResponse::Ok().json(ApiResponse {
            success: true,
            message: msg,
        }),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse {
            success: false,
            message: e,
        }),
    }
}

pub async fn graceful_shutdown_handler(
    session: Session,
    idrac: web::Data<Arc<IdracClient>>,
) -> HttpResponse {
    if check_auth(session).await.is_err() {
        return HttpResponse::Unauthorized().json(ApiResponse {
            success: false,
            message: "Not authenticated".to_string(),
        });
    }

    match idrac.graceful_shutdown().await {
        Ok(msg) => HttpResponse::Ok().json(ApiResponse {
            success: true,
            message: msg,
        }),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse {
            success: false,
            message: e,
        }),
    }
}
