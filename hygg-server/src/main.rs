use std::sync::Arc;
use axum::{
    routing::{get, post},
    Router, Json, extract::{State, Path},
    response::IntoResponse,
    http::StatusCode,
    body::Body,
};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use sqlx::Row;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use async_lock::Mutex;
use tower_http::cors::CorsLayer;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ReadingProgress {
    id: Uuid,
    file_path: String,
    position: usize,
    user_id: String,
    last_accessed: DateTime<Utc>,
    lock_holder: Option<String>,
    lock_expiry: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProgressEvent {
    id: Uuid,
    progress_id: Uuid,
    event_type: String,
    position: usize,
    user_id: String,
    timestamp: DateTime<Utc>,
}

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
    locks: Arc<Mutex<Vec<(Uuid, String, DateTime<Utc>)>>>,
}

async fn init_db(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("
        CREATE TABLE IF NOT EXISTS reading_progress (
            id TEXT PRIMARY KEY,
            file_path TEXT NOT NULL,
            position INTEGER NOT NULL,
            user_id TEXT NOT NULL,
            last_accessed TIMESTAMP NOT NULL,
            lock_holder TEXT,
            lock_expiry TIMESTAMP
        )"
    ).execute(pool).await?;

    sqlx::query("
        CREATE TABLE IF NOT EXISTS progress_events (
            id TEXT PRIMARY KEY,
            progress_id TEXT NOT NULL,
            event_type TEXT NOT NULL,
            position INTEGER NOT NULL,
            user_id TEXT NOT NULL,
            timestamp TIMESTAMP NOT NULL,
            FOREIGN KEY(progress_id) REFERENCES reading_progress(id)
        )"
    ).execute(pool).await?;

    Ok(())
}

async fn get_file_content(Path(file_path): Path<String>) -> impl IntoResponse {
    // For security reasons, restrict to files in the test-data directory
    let safe_path = format!("test-data/{}", file_path.trim_start_matches('/').split('/').last().unwrap_or("sample.txt"));
    
    println!("Attempting to read file: {}", safe_path);
    match tokio::fs::read_to_string(&safe_path).await {
        Ok(content) => (StatusCode::OK, content).into_response(),
        Err(err) => {
            eprintln!("Error reading file {}: {}", safe_path, err);
            (StatusCode::NOT_FOUND, format!("File not found: {}", file_path)).into_response()
        },
    }
}

async fn acquire_lock(
    State(state): State<AppState>,
    Json(progress): Json<ReadingProgress>,
) -> Result<Json<ReadingProgress>, (axum::http::StatusCode, String)> {
    println!("LOCK REQUEST: User {} is trying to acquire lock for progress {}", progress.user_id, progress.id);
    
    // Check if progress entry exists, create if not
    let exists = sqlx::query("SELECT id FROM reading_progress WHERE id = ?")
        .bind(progress.id.to_string())
        .fetch_optional(&state.db)
        .await
        .map_err(|e| {
            println!("DATABASE ERROR: Failed to query reading_progress: {}", e);
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;
        
    if exists.is_none() {
        println!("PROGRESS INIT: Creating new progress entry for {} with id {}", progress.file_path, progress.id);
        sqlx::query("
            INSERT INTO reading_progress (id, file_path, position, user_id, last_accessed, lock_holder, lock_expiry)
            VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(progress.id.to_string())
        .bind(&progress.file_path)
        .bind(progress.position as i64)
        .bind(&progress.user_id)
        .bind(Utc::now())
        .bind::<Option<&str>>(None)
        .bind::<Option<DateTime<Utc>>>(None)
        .execute(&state.db)
        .await
        .map_err(|e| {
            println!("DATABASE ERROR: Failed to create progress entry: {}", e);
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;
    }
    
    let mut locks = state.locks.lock().await;
    
    // Clean expired locks
    let now = Utc::now();
    locks.retain(|(_, _, expiry)| expiry > &now);
    
    // Check if progress is already locked
    if let Some((_, holder, _)) = locks.iter().find(|(id, _, _)| *id == progress.id) {
        println!("LOCK DENIED: Progress {} is already locked by {}", progress.id, holder);
        return Err((axum::http::StatusCode::CONFLICT, 
            format!("Progress is locked by {}", holder)));
    }
    
    // Acquire new lock
    let expiry = now + chrono::Duration::minutes(15);
    locks.push((progress.id, progress.user_id.clone(), expiry));
    println!("LOCK GRANTED: User {} acquired lock for progress {} until {}", progress.user_id, progress.id, expiry);
    
    // Update database
    sqlx::query("
        UPDATE reading_progress 
        SET lock_holder = ?, lock_expiry = ? 
        WHERE id = ?"
    )
    .bind(&progress.user_id)
    .bind(expiry)
    .bind(progress.id.to_string())
    .execute(&state.db)
    .await
    .map_err(|e| {
        println!("DATABASE ERROR: Failed to update lock in database: {}", e);
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;
    
    // Fetch the latest progress including current position from the database
    let db_progress = sqlx::query("
        SELECT id, file_path, position, user_id, last_accessed, lock_holder, lock_expiry 
        FROM reading_progress 
        WHERE id = ?"
    )
    .bind(progress.id.to_string())
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        println!("DATABASE ERROR: Failed to query updated progress: {}", e);
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;
    
    // Convert to ReadingProgress
    let id_str: String = db_progress.get(0);
    let id = Uuid::parse_str(&id_str)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Invalid UUID: {}", e)))?;
    
    let mut updated_progress = progress.clone();
    updated_progress.id = id;
    updated_progress.file_path = db_progress.get(1);
    updated_progress.position = db_progress.get::<i64, _>(2) as usize;
    updated_progress.user_id = db_progress.get(3);
    updated_progress.last_accessed = db_progress.get(4);
    updated_progress.lock_holder = Some(progress.user_id.clone()); // We just set this
    updated_progress.lock_expiry = Some(expiry); // We just set this
    
    println!("PROGRESS LOADED: User {} acquired lock with position {}", updated_progress.user_id, updated_progress.position);
    
    Ok(Json(updated_progress))
}

async fn release_lock(
    State(state): State<AppState>,
    Json(progress): Json<ReadingProgress>,
) -> Result<Json<ReadingProgress>, (axum::http::StatusCode, String)> {
    println!("LOCK RELEASE: User {} is trying to release lock for progress {}", progress.user_id, progress.id);
    let mut locks = state.locks.lock().await;
    
    // Verify lock is held by the user
    let lock_idx = locks.iter().position(|(id, holder, _)| 
        *id == progress.id && holder == &progress.user_id
    );
    
    match lock_idx {
        Some(idx) => {
            // Remove the lock
            locks.remove(idx);
            println!("LOCK RELEASED: User {} released lock for progress {}", progress.user_id, progress.id);
            
            // Update database to clear lock holder
            sqlx::query("
                UPDATE reading_progress 
                SET lock_holder = NULL, lock_expiry = NULL 
                WHERE id = ?"
            )
            .bind(progress.id.to_string())
            .execute(&state.db)
            .await
            .map_err(|e| {
                println!("DATABASE ERROR: Failed to update lock in database: {}", e);
                (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            })?;
            
            let mut updated_progress = progress.clone();
            updated_progress.lock_holder = None;
            updated_progress.lock_expiry = None;
            
            Ok(Json(updated_progress))
        },
        None => {
            println!("LOCK RELEASE DENIED: User {} doesn't hold lock for progress {}", progress.user_id, progress.id);
            Err((axum::http::StatusCode::FORBIDDEN, 
                "You don't hold the lock for this progress".to_string()))
        }
    }
}

async fn update_progress(
    State(state): State<AppState>,
    Json(progress): Json<ReadingProgress>,
) -> Result<Json<ReadingProgress>, (axum::http::StatusCode, String)> {
    println!("PROGRESS UPDATE: User {} is updating progress to position {}", progress.user_id, progress.position);
    let locks = state.locks.lock().await;
    
    // Verify lock
    if !locks.iter().any(|(id, holder, _)| 
        *id == progress.id && holder == &progress.user_id
    ) {
        println!("PROGRESS DENIED: User {} doesn't hold lock for progress {}", progress.user_id, progress.id);
        return Err((axum::http::StatusCode::FORBIDDEN, 
            "You don't hold the lock for this progress".to_string()));
    }
    
    // Record event
    let event = ProgressEvent {
        id: Uuid::new_v4(),
        progress_id: progress.id,
        event_type: "update".to_string(),
        position: progress.position,
        user_id: progress.user_id.clone(),
        timestamp: Utc::now(),
    };
    println!("PROGRESS EVENT: Recording event id {} for progress {}", event.id, event.progress_id);
    
    sqlx::query("
        INSERT INTO progress_events (id, progress_id, event_type, position, user_id, timestamp)
        VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(event.id.to_string())
    .bind(event.progress_id.to_string())
    .bind(&event.event_type)
    .bind(event.position as i64)
    .bind(&event.user_id)
    .bind(event.timestamp)
    .execute(&state.db)
    .await
    .map_err(|e| {
        println!("DATABASE ERROR: Failed to insert progress event: {}", e);
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;
    
    // Update progress
    sqlx::query("
        UPDATE reading_progress 
        SET position = ?, last_accessed = ? 
        WHERE id = ?"
    )
    .bind(progress.position as i64)
    .bind(Utc::now())
    .bind(progress.id.to_string())
    .execute(&state.db)
    .await
    .map_err(|e| {
        println!("DATABASE ERROR: Failed to update reading progress: {}", e);
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;
    
    println!("PROGRESS UPDATED: User {} successfully updated position to {}", progress.user_id, progress.position);
    Ok(Json(progress))
}

async fn get_progress(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ReadingProgress>, (axum::http::StatusCode, String)> {
    println!("PROGRESS GET: Retrieving progress with ID {}", id);
    
    // Query the database for the progress with the given ID
    let progress = sqlx::query("
        SELECT id, file_path, position, user_id, last_accessed, lock_holder, lock_expiry 
        FROM reading_progress 
        WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        println!("DATABASE ERROR: Failed to query reading progress: {}", e);
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;
    
    // Convert the database row to a ReadingProgress struct
    match progress {
        Some(row) => {
            // Convert ID to UUID
            let id_str: String = row.get(0);
            let id = match Uuid::parse_str(&id_str) {
                Ok(uuid) => uuid,
                Err(e) => return Err((StatusCode::BAD_REQUEST, format!("Invalid UUID: {}", e))),
            };
            
            // Extract other fields
            let file_path: String = row.get(1);
            let position: i64 = row.get(2);
            let user_id: String = row.get(3);
            let last_accessed: DateTime<Utc> = row.get(4);
            let lock_holder: Option<String> = row.get(5);
            let lock_expiry: Option<DateTime<Utc>> = row.get(6);
            
            let progress = ReadingProgress {
                id,
                file_path,
                position: position as usize,
                user_id,
                last_accessed,
                lock_holder,
                lock_expiry,
            };
            
            println!("PROGRESS FOUND: ID {} at position {}", id, progress.position);
            Ok(Json(progress))
        },
        None => {
            println!("PROGRESS NOT FOUND: ID {}", id);
            Err((StatusCode::NOT_FOUND, format!("Progress with ID {} not found", id)))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    let db = SqlitePoolOptions::new()
        .max_connections(10)
        .connect("data/hygg.db")
        .await?
    ;
    
    init_db(&db).await?;
    
    let app_state = AppState {
        db,
        locks: Arc::new(Mutex::new(Vec::new())),
    };
    
    let app = Router::new()
        .route("/file/:path", get(get_file_content))
        .route("/progress/lock", post(acquire_lock))
        .route("/progress/release", post(release_lock))
        .route("/progress/update", post(update_progress))
        .route("/progress/:id", get(get_progress))
        .layer(CorsLayer::permissive())
        .with_state(app_state);
    
    println!("Server running on http://localhost:3001");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
