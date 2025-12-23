use error_stack::{Report, ResultExt};
use serde::Deserialize;
use tracing::info;

use crate::ArwLock;

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

#[derive(Debug, Clone)]
pub struct JwksState {
    pub keys: ArwLock<Vec<Jwk>>,
}

impl JwksState {
    pub async fn find_key(&self, kid: &str) -> Option<Jwk> {
        let keys = self.keys.read().await;
        keys.iter().find(|k| k.kid == kid).cloned()
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
    // pub kty: String,
    // #[serde(rename = "use")]
    // pub key_use: Option<String>,
    // modulus
    pub n: String,
    // exponent
    pub e: String,
    // pub alg: String,
}
