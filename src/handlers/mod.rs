pub mod audit;
pub mod auth;
pub mod memberships;
pub mod organizations;
pub mod permissions;
pub mod roles;
pub mod sessions;
pub mod users;

use crate::middleware::auth::require_auth;
use crate::state::AppState;
use axum::{
    Router, middleware,
    routing::{delete, get, patch, post},
};

pub fn app_router(state: AppState) -> Router {
    let public_routes = Router::new()
        .route("/health", get(|| async { "Frenzy IAM is Online!" }))
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/refresh", post(auth::refresh));

    let protected_routes = Router::new()
        .route("/auth/logout", post(auth::logout))
        .route("/users/me", get(users::get_me).patch(users::update_profile))
        .route(
            "/organizations",
            get(organizations::list_orgs).post(organizations::create_organization),
        )
        .route(
            "/organizations/{id}",
            get(organizations::get_organization).patch(organizations::update_organization),
        )
        .route(
            "/organizations/{id}/members",
            get(memberships::list_members).post(memberships::add_member),
        )
        .route(
            "/organizations/{id}/members/{user_id}",
            delete(memberships::remove_member),
        )
        .route("/sessions", get(sessions::list_sessions))
        .route("/sessions/{id}", delete(sessions::revoke_session))
        .route("/roles", get(roles::list_roles).post(roles::create_role))
        .route(
            "/roles/{id}",
            patch(roles::update_role).delete(roles::delete_role),
        )
        .route(
            "/permissions",
            get(permissions::list_permissions).post(permissions::assign_permission),
        )
        .route("/permissions/{id}", delete(permissions::remove_permission))
        .route_layer(middleware::from_fn_with_state(state.clone(), require_auth));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(state)
}
