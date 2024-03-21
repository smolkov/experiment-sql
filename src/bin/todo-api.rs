use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use axum::{
    response::{IntoResponse, Response},
    routing::{delete, get, patch, post},
    Router,
};
use sqlx::SqlitePool;
use tokio::io::Join;
use tokio::signal;
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use todo::pagination::Pagination;
use todo::todo::{CreateTodo, Todo, TodoRepository, UpdateTodo};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // axum logs rejections from built-in extractors with the `axum::rejection`
                // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
                "example_tracing_aka_logging=debug,tower_http=debug,axum::rejection=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let repo = TodoRepository::new(SqlitePool::connect(&std::env::var("DATABASE_URL")?).await?);

    let router = Router::new()
        .route("/todos", get(get_todos).post(add_todo))
        .route(
            "/todos/:id",
            get(get_todo).patch(update_todo).delete(delete_todo),
        )
        .route("/todos/persist", post(persist))
        .with_state(repo)
        .layer(ServiceBuilder::new())
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

async fn get_todos(
    pagination: Option<Query<Pagination>>,
    State(mut repo): State<TodoRepository>,
) -> Result<impl IntoResponse, StatusCode> {
    let Query(pagination) = pagination.unwrap_or_default();
    Ok(Json(repo.list(pagination).await.unwrap()))
}

async fn get_todo(
    Path(id): Path<i64>,
    State(mut repo): State<TodoRepository>,
) -> impl IntoResponse {
    let todo = repo.get(id).await.unwrap();
    Json(todo).into_response()
    // }else {
    // (StatusCode::NOT_FOUND,"Not found").into_response()
    // }
}

async fn add_todo(
    State(mut todos): State<TodoRepository>,
    Json(todo): Json<CreateTodo>,
) -> impl IntoResponse {
    let todo = todos.create(todo).await.unwrap();
    (StatusCode::CREATED, Json(todo)).into_response()
}

async fn delete_todo(
    Path(id): Path<i64>,
    State(mut repo): State<TodoRepository>,
) -> impl IntoResponse {
    repo.delete(id).await.unwrap();
    StatusCode::NO_CONTENT
}

async fn update_todo(
    Path(id): Path<i64>,
    State(mut repo): State<TodoRepository>,
    Json(todo): Json<UpdateTodo>,
) -> Result<impl IntoResponse, StatusCode> {
    repo.update(id, todo).await.unwrap();
    Ok(StatusCode::OK)
    // match  todos.update_item(id, todo) {
    // Some(todo) => Ok(Json(todo.clone())),
    // None => Err(StatusCode::NOT_FOUND),
    // }
}

async fn persist() -> impl IntoResponse {
    "Call method persist"
}
