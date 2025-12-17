use std::{convert::Infallible, fmt::Display, ops::BitOr, str::FromStr, sync::Arc};

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::MethodRouter,
};
use error_stack::{Report, ResultExt};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use topics_core::TopicEngine;
use tracing::{debug, error, info, instrument, warn};
use utoipa_axum::router::OpenApiRouter;

use crate::state::TopicAppState;

pub type OAuthResult<T> = Result<T, Report<MissingOAuthProperty>>;

#[derive(Debug, thiserror::Error)]
#[error("{0} oauth property not specified")]
pub struct MissingOAuthProperty(&'static str);

#[derive(Debug, Clone)]
pub struct OAuthConfig {
    /// Where to fetch public keys
    pub jwks_url: String,
    /// Who issued the token (the URL of the issuer)
    pub issuer_url: String,
    /// Key in JWT for roles
    pub roles_claims_path: String,
    /// this api's identifier.
    // might all be the same, could maybe hard code this, but also maybe it's best to pass that info in via the env
    pub audience: String,
}

const OAUTH_JWKS_URL: &str = "OAUTH_JWKS_URL";
const OAUTH_ISSUER_URL: &str = "OAUTH_ISSUER_URL";
const OAUTH_ROLES_JWT_PATH: &str = "OAUTH_ROLES_JWT_PATH";
const OAUTH_AUDIENCE: &str = "OAUTH_AUDIENCE";

impl OAuthConfig {
    pub fn from_env() -> OAuthResult<Self> {
        Ok(Self {
            jwks_url: std::env::var(OAUTH_JWKS_URL)
                .change_context(MissingOAuthProperty(OAUTH_JWKS_URL))?,
            issuer_url: std::env::var(OAUTH_ISSUER_URL)
                .change_context(MissingOAuthProperty(OAUTH_ISSUER_URL))?,
            roles_claims_path: std::env::var(OAUTH_ROLES_JWT_PATH)
                .change_context(MissingOAuthProperty(OAUTH_ROLES_JWT_PATH))?,
            audience: std::env::var(OAUTH_AUDIENCE).unwrap_or_else(|_| {
                info!("OAUTH_AUDIENCE not specified, going with default");
                String::from("topics-api")
            }),
        })
    }
}

/*
 * How does this work?
 * Start up an authorization service.
 * Use that authorization service to get a JWT token (acting as a client)
 * Send that token along with a request to this service (resource server)
 * This service uses the authorization service's public keys (retrieved at startup,
 *  when the app state is created), along with the issue and audience (passed in as env variables)
 *  to decode and validate the JWT token.
 * In the token, lives roles that can be parsed. These roles are used to guard endpoints.
 *
 */

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    email: Option<String>,
    // apparently this can be a string or an array
    #[serde(default)]
    aud: Value,
    /// expiration
    exp: usize,
    /// issued?
    iss: String,
    #[serde(flatten)]
    extra: serde_json::Map<String, Value>,
}

impl Claims {
    pub fn into_authed_user(self, roles_path: &str) -> AuthedUser {
        let parts = roles_path.split('.');

        let mut current = Value::Object(self.extra);

        for part in parts {
            current = match current.get(part) {
                Some(val) => val.clone(),
                None => {
                    return AuthedUser {
                        id: self.sub.into(),
                        email: self.email.map(|e| e.into()),
                        roles: Roles::NONE,
                    };
                }
            }
        }

        let roles = match current {
            Value::Array(roles) => roles.into_iter().fold(Roles::NONE, |r, next| {
                r | next
                    .to_string()
                    .parse()
                    .expect("roles flag parse is infallible")
            }),
            _ => Roles::NONE,
        };

        AuthedUser {
            id: self.sub.into(),
            email: self.email.map(|e| e.into()),
            roles,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Roles(u8);
impl Roles {
    const MAX: u8 = 4;
    pub const NONE: Roles = Roles(0);
    pub const TOPIC_READ: Roles = Roles(1);
    pub const TOPIC_WRITE: Roles = Roles(2);
    pub const TOPIC_ADMIN: Roles = Roles(Self::MAX);

    fn contains(&self, other: Roles) -> bool {
        self.0 & other.0 != Roles::NONE.0
    }

    fn iter(&self) -> RolesIter {
        RolesIter::new(*self)
    }
}

impl Default for Roles {
    fn default() -> Self {
        Self::NONE
    }
}

/// An iterator over the individual roles stored in the `Roles` bitflag.
/// ```rust
/// let roles = Roles::TOPIC_WRITE | Roles::TOPIC_READ;
/// let mut itr = roles.iter();
///
/// assert_eq!(Some(Roles::TOPIC_READ), itr.next());
/// assert_eq!(Some(Roles::TOPIC_WRITE), itr.next());
/// assert_eq!(None, itr.next());
/// ```
struct RolesIter {
    roles: Roles,
    idx: u8,
}

impl RolesIter {
    fn new(roles: Roles) -> Self {
        Self { roles, idx: 0 }
    }
}

impl Iterator for RolesIter {
    type Item = Roles;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let roles = self.roles.0 >> self.idx;

            if roles == 0 {
                return None;
            }

            let role = roles % 2;

            if role == 1 {
                let result = Some(Roles(2u8.pow(self.idx as u32)));
                self.idx += 1;
                return result;
            } else {
                self.idx += 1;
            }
        }
    }
}

impl Display for Roles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if *self == Roles::NONE {
            write!(f, "[]")
        } else {
            write!(f, "[")?;
            for role in self.iter() {
                match role.0 {
                    1 => write!(f, "TOPIC_READ,")?,
                    2 => write!(f, "TOPIC_WRITE,")?,
                    4 => write!(f, "ADMIN,")?,
                    _ => unreachable!("unless new topic added"),
                }
            }
            write!(f, "]")
        }
    }
}

impl BitOr for Roles {
    type Output = Roles;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl FromStr for Roles {
    type Err = Infallible; // unknown roles are ignored

    #[instrument]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim_matches('"') {
            "TOPIC_ADMIN" => Ok(Roles::TOPIC_ADMIN),
            "TOPIC_READ" => Ok(Roles::TOPIC_READ),
            "TOPIC_WRITE" => Ok(Roles::TOPIC_WRITE),
            other => {
                warn!("Unknown role: {other}. Ignoring");
                Ok(Roles::NONE)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuthedUser {
    pub id: Arc<str>,
    pub email: Option<Arc<str>>,
    pub roles: Roles,
}

impl AuthedUser {
    pub fn has_roles(&self, role: Roles) -> bool {
        self.roles.contains(role)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Jwks {
    pub keys: Vec<Jwk>,
}

// I understood this that night when I was using claude to figure out how this all works.
// felt like a fever dream.
// figure this out again
#[derive(Debug, Deserialize, Clone)]
pub struct Jwk {
    pub kid: String,
    pub kty: String,
    #[serde(rename = "use")]
    pub key_use: Option<String>,
    pub n: String,
    pub e: String,
    pub alg: String,
}

// TODO better error logging
#[instrument(skip_all)]
pub async fn validate_token<T: TopicEngine>(
    State(state): State<TopicAppState<T>>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
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

        let jwk = state.find_jwk_key(&kid).await.ok_or_else(|| {
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

        let token_data = decode::<Claims>(token, &decoding_key, &validation).map_err(|e| {
            error!("Token validation error: {e}");
            StatusCode::UNAUTHORIZED
        })?;

        let authed_user = token_data
            .claims
            .into_authed_user(&state.oauth_config.roles_claims_path);

        info!("Token validated for user '{}'", authed_user.id);
        info!("User roles: {}", authed_user.roles,);

        request.extensions_mut().insert(authed_user);
    }
    Ok(next.run(request).await)
}

pub trait ProtectedRouter<S> {
    fn protected_route(
        self,
        path: &str,
        method_router: MethodRouter<S>,
        required_roles: Roles,
    ) -> Self;
}

impl<S> ProtectedRouter<S> for OpenApiRouter<S>
where
    S: Send + Sync + Clone + 'static,
{
    #[instrument(skip_all)]
    fn protected_route(
        self,
        path: &str,
        method_router: MethodRouter<S>,
        required_roles: Roles,
    ) -> Self {
        debug!(
            "creating route '{path}' protected by roles {required_roles} at address {:p}",
            &required_roles
        );
        self.route(
            path,
            method_router.layer(middleware::from_fn_with_state(
                required_roles,
                require_roles,
            )),
        )
    }
}

#[instrument(skip_all)]
async fn require_roles(
    State(required_roles): State<Roles>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let user = req.extensions().get::<AuthedUser>().ok_or_else(|| {
        error!("endpoint requires authorized user, none was found");
        StatusCode::UNAUTHORIZED
    })?;

    debug!("required roles: {required_roles}");
    if required_roles == Roles::NONE || user.has_roles(required_roles) {
        Ok(next.run(req).await)
    } else {
        warn!("User {} does not have the authority! (🧙‍♂️🚫➡️)", user.id);
        Err(StatusCode::FORBIDDEN)
    }
}

#[cfg(test)]
mod tests {
    use crate::auth::Roles;

    #[test]
    fn roles_contains() {
        let roles = Roles::TOPIC_READ;

        assert_eq!(Roles::TOPIC_READ, roles);

        let roles = Roles::TOPIC_READ | Roles::TOPIC_WRITE;

        assert!(roles.contains(Roles::TOPIC_READ));
        assert!(roles.contains(Roles::TOPIC_WRITE));
        assert!(roles.contains(Roles::TOPIC_WRITE | Roles::TOPIC_READ));
        assert!(
            !roles.contains(Roles::TOPIC_ADMIN),
            "!{:b}.contains({:b})",
            roles.0,
            Roles::TOPIC_ADMIN.0
        );
    }

    #[test]
    fn roles_iter() {
        let roles = Roles::TOPIC_READ | Roles::TOPIC_WRITE;
        let mut iter = roles.iter();

        assert_eq!(Some(Roles::TOPIC_READ), iter.next(), "expecting TOPIC_READ");
        assert_eq!(
            Some(Roles::TOPIC_WRITE),
            iter.next(),
            "expecting TOPIC_WRITE"
        );
        assert_eq!(None, iter.next(), "expecting None");
    }

    #[test]
    fn roles_display() {
        let roles = Roles::TOPIC_READ;

        assert_eq!("[TOPIC_READ,]", &roles.to_string());

        let roles = Roles::TOPIC_READ | Roles::TOPIC_ADMIN;

        assert_eq!("[TOPIC_READ,ADMIN,]", &roles.to_string());

        let roles = Roles::NONE;
        assert_eq!("[]", &roles.to_string());
    }
}
