use crate::errors::AppError;
use crate::services::jwt::verify_access_token;
use crate::state::AppState;
use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};

pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|val| val.to_str().ok())
        .and_then(|str| str.strip_prefix("Bearer "))
        .ok_or_else(|| AppError::Unauthorized("Missing or malformed Bearer token".into()))?;

    let claims = verify_access_token(auth_header, &state.config.jwt_secret)?;

    req.extensions_mut().insert(claims.sub);

    Ok(next.run(req).await)
}
