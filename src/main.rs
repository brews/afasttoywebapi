mod core;
mod models;
mod schema;
mod storage;

use crate::storage::{create_database_connection_pool, FacilitiesFilter};
use axum::extract::{Path, Query};
use axum::{
    extract::State, http::StatusCode, routing::delete, routing::get, routing::post, Json, Router,
};
use deadpool_diesel::postgres::Pool;
use dotenvy::dotenv;
use std::env;
use tower_http::trace::TraceLayer;
use tracing::{debug, error, info};

#[derive(Clone)]
struct AppState {
    conn_pool: Pool,
}

#[tokio::main]
async fn main() {
    // Init tracing
    tracing_subscriber::fmt::init();
    info!("starting server");

    // Read in settings.
    dotenv().ok();
    let database_url =
        env::var("DATABASE_URL").expect("required DATABASE_URL environment variable is not set");
    let host = env::var("HOST").expect("required HOST environment variable is not set");
    let port = env::var("PORT").expect("required PORT environment variable is not set");
    let server_url = format!("{host}:{port}");

    let conn_pool =
        create_database_connection_pool(database_url, 5).expect("unable to connect to database");
    // Here is where we'd do automated migrations if we're doing that.
    debug!("setup database connection pool");

    let state = AppState { conn_pool };

    let app = Router::new()
        .route("/facilities", post(post_facility))
        .route("/facilities/{uid}", get(get_facility))
        .route("/facilities/", get(get_facilities))
        .route("/facilities/{uid}", delete(delete_facility))
        .layer(TraceLayer::new_for_http())
        .with_state(state);
    debug!("setup app routes");

    info!("listening on {server_url:?}");
    let listener = tokio::net::TcpListener::bind(server_url).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// Handle request to create a new facility.
async fn post_facility(
    State(state): State<AppState>,
    Json(payload): Json<core::Facility>,
) -> Result<Json<core::Facility>, StatusCode> {
    debug!("received request to post {payload:?}");

    let client_result = state.conn_pool.get().await;
    let client = match client_result {
        Ok(r) => r,
        Err(e) => {
            error!("error collecting client from connection pool {e:?}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let interaction_result = client
        .interact(|conn| storage::write_facility(conn, payload))
        .await;
    let new_facility_result = match interaction_result {
        Ok(r) => r,
        Err(e) => {
            error!("error interacting through connection pool {e:?}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match new_facility_result {
        Ok(new_facility) => Ok(Json(new_facility)), // TODO: Should be StatusCode::CREATED. Check it.
        Err(diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::UniqueViolation,
            _,
        )) => Err(StatusCode::CONFLICT), // TODO: Should we log this?
        Err(e) => {
            error!("error writing new facility to database {e:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Handle request for to get an existing facility.
async fn get_facility(
    Path(uid): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<core::Facility>, StatusCode> {
    debug!("received request to get facility {uid:?}");

    let client_result = state.conn_pool.get().await;
    let client = match client_result {
        Ok(r) => r,
        Err(e) => {
            error!("error collecting client from connection pool {e:?}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let interaction_result = client
        .interact(|conn| storage::read_facility(conn, uid))
        .await;
    let read_facility_result = match interaction_result {
        Ok(r) => r,
        Err(e) => {
            error!("error interacting through connection pool {e:?}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match read_facility_result {
        Ok(matching_facility) => Ok(Json(matching_facility)),
        Err(diesel::result::Error::NotFound) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("error getting facility from database {e:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Handle request to list facilities.
async fn get_facilities(
    State(state): State<AppState>,
    Query(params): Query<FacilitiesFilter>,
) -> Result<Json<Vec<core::Facility>>, StatusCode> {
    debug!("Received request to get facilities with filter {params:?}");

    let client_result = state.conn_pool.get().await;
    let client = match client_result {
        Ok(r) => r,
        Err(e) => {
            return {
                error!("error collecting client from connection pool {e:?}");
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    };

    let interaction_result = client
        .interact(|conn| storage::list_facilities(conn, params))
        .await;
    let list_facilities_result = match interaction_result {
        Ok(r) => r,
        Err(e) => {
            error!("error interacting through connection pool {e:?}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match list_facilities_result {
        Ok(facilities) => Ok(Json(facilities)),
        Err(e) => {
            error!("error listing facilities from database {e:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Handle request to delete a new facility.
async fn delete_facility(
    State(state): State<AppState>,
    Path(uid): Path<String>,
) -> Result<StatusCode, StatusCode> {
    debug!("Received request to delete facility {uid:?}");

    let client_result = state.conn_pool.get().await;
    let client = match client_result {
        Ok(r) => r,
        Err(e) => {
            error!("error collecting client from connection pool {e:?}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let interaction_result = client
        .interact(|conn| storage::delete_facility(conn, uid))
        .await;
    let delete_result = match interaction_result {
        Ok(r) => r,
        Err(e) => {
            error!("error interacting through connection pool {e:?}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match delete_result {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(diesel::result::Error::NotFound) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("error deleting facility from database {e:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// TODO: Format error messages properly.
