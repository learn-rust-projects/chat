use axum::{
    extract::{FromRequestParts, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use tracing::warn;

use crate::middlewares::TokenVerify;

pub async fn verify_token<T>(State(state): State<T>, request: Request, next: Next) -> Response
where
    T: TokenVerify + Clone + Send + Sync + 'static,
{
    let (mut parts, body) = request.into_parts();
    let req =
        match TypedHeader::<Authorization<Bearer>>::from_request_parts(&mut parts, &state).await {
            Ok(TypedHeader(Authorization(bearer))) => {
                let token = bearer.token();
                match state.verify(token) {
                    Ok(claims) => {
                        let mut req = Request::from_parts(parts, body);
                        req.extensions_mut().insert(claims);
                        req
                    }
                    Err(err) => {
                        let msg = format!("verify token failed:{:?}", err);
                        warn!(msg);
                        return (StatusCode::FORBIDDEN, msg).into_response();
                    }
                }
            }
            Err(err) => {
                let msg = format!("parse Authorization header failed:{}", err);
                warn!(msg);
                return (StatusCode::UNAUTHORIZED, msg).into_response();
            }
        };

    next.run(req).await
}
