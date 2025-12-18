use std::{fmt::Debug, str::FromStr};

use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use error_stack::{Report, ResultExt};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use tracing::{error, info, instrument};

use crate::{
    ArwLock,
    auth::{
        claims::Claims,
        oauth::{Jwk, Jwks, JwksState, OAuthConfig},
        roles::Roles,
    },
};

#[derive(Debug, Clone)]
pub struct AuthState {
    jwks: JwksState,
    oauth_config: OAuthConfig,
}

#[derive(Debug, thiserror::Error)]
#[error("failed to create validate token state")]
pub struct AuthStateCreationErr;

impl AuthState {
    pub async fn create() -> Result<Self, Report<AuthStateCreationErr>> {
        let oauth_config = OAuthConfig::from_env().change_context(AuthStateCreationErr)?;
        let jwks = refresh_jwks_from_url(&oauth_config.jwks_url)
            .await
            .change_context(AuthStateCreationErr)?;
        Ok(Self {
            jwks: JwksState {
                keys: ArwLock::new(jwks),
            },
            oauth_config,
        })
    }

    #[instrument]
    pub async fn refresh_jwks(&mut self) -> Result<(), Report<RefreshJwksErr>> {
        let jwks = refresh_jwks_from_url(&self.oauth_config.jwks_url).await?;
        let mut keys = self.jwks.keys.write().await;
        *keys = jwks;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
#[error("failed to retrieve jwks data")]
pub struct RefreshJwksErr;

#[instrument]
async fn refresh_jwks_from_url(jwks_uri: &str) -> Result<Vec<Jwk>, Report<RefreshJwksErr>> {
    info!("fetching JWKS");

    let jwks: Jwks = reqwest::get(jwks_uri)
        .await
        .change_context(RefreshJwksErr)?
        .json()
        .await
        .change_context(RefreshJwksErr)?;

    if jwks.keys.is_empty() {
        error!("no jwks were found");
    } else {
        info!("found {} jwks", jwks.keys.len());
    }
    Ok(jwks.keys)
}

#[instrument(skip_all)]
pub async fn validate_token<R>(
    State(state): State<AuthState>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode>
where
    R: Roles,
    <R as FromStr>::Err: Debug,
{
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    // not all endpoints will require quthorization, this should allow those to go through.
    // those that require authorization will expect an `AuthedUser` to exist in extensions
    if let Some(auth_header) = auth_header {
        if !auth_header.starts_with("Bearer ") {
            error!("invalid authorization type");
            return Err(StatusCode::UNAUTHORIZED);
        }

        let token = &auth_header["Bearer ".len()..];

        let header = jsonwebtoken::decode_header(token).map_err(|_| {
            error!("JWT token decoding (without verification) failed");
            StatusCode::UNAUTHORIZED
        })?;
        let kid = header.kid.ok_or_else(|| {
            error!("invalid token: kid missing");
            StatusCode::UNAUTHORIZED
        })?;

        let jwk = state.jwks.find_key(&kid).await.ok_or_else(|| {
            error!("kid key not found");
            StatusCode::UNAUTHORIZED
        })?;

        let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e).map_err(|_| {
            error!("failed to create decoding key");
            StatusCode::UNAUTHORIZED
        })?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[&state.oauth_config.audience]);
        validation.set_issuer(&[&state.oauth_config.issuer_url]);

        let token_data = jsonwebtoken::decode::<Claims>(token, &decoding_key, &validation)
            .map_err(|e| {
                error!("Token validation error: {e}");
                StatusCode::UNAUTHORIZED
            })?;

        let authed_user = token_data
            .claims
            .into_authed_user::<R>(&state.oauth_config.roles_claims_path);

        info!("Token validated for user '{}'", authed_user.id);
        info!("User roles: {}", authed_user.roles,);

        request.extensions_mut().insert(authed_user);
    }
    Ok(next.run(request).await)
}
