pub mod auth;
pub mod organizations;
pub mod permissions;
pub mod roles;
pub mod sessions;
pub mod users;
use crate::middleware::auth::require_auth;
use crate::state::AppState;
use axum::{
    Router, middleware,
    routing::{delete, get, post},
};

pub fn app_router(state: AppState) -> Router {
    let public_routes = Router::new()
        .route("/health", get(|| async { "Frenzy IAM is Online!" }))
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/refresh", post(users::refresh));

    let protected_routes = Router::new()
        .route("/users/me", get(users::get_me))
        .route("/auth/logout", post(users::logout))
        .route(
            "/organizations",
            get(organizations::list_orgs).post(organizations::create_organization),
        )
        .route(
            "/organizations/:id",
            get(organizations::get_organization).patch(organizations::update_organization),
        )
        .route("/sessions", get(sessions::list_sessions))
        .route("/sessions/:id", delete(sessions::revoke_session))
        .route_layer(middleware::from_fn_with_state(state.clone(), require_auth));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(state)
}
