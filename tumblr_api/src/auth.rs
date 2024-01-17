use std::time::{Duration, Instant};

use serde::{Serialize, Deserialize};
use serde_enum_str::{Deserialize_enum_str, Serialize_enum_str};
use serde_with::{serde_as, DurationSeconds};
use veil::Redact;

#[derive(Redact)]
pub struct Credentials {
    #[redact]
    consumer_key: String,
    #[redact]
    consumer_secret: String,
    token: async_lock::Mutex<Option<TokenWithExpiry>>,
}

#[derive(Redact, Clone)]
pub struct BearerToken(#[redact] pub String);

#[derive(Debug)]
struct TokenWithExpiry {
    token: BearerToken,
    expires_at: Instant,
}

impl TokenWithExpiry {
    fn is_expired(&self) -> bool {
        let now = Instant::now();
        // TODO this should probably be done with some extra tolerance, if we're within a
        //      couple seconds of expiring we should probably just count it
        now >= self.expires_at
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Scope {
    Basic,
    OfflineAccess,
    Write,
}

/// possible values of `error` in OAuth 2 error response, see <https://www.rfc-editor.org/rfc/rfc6749#section-5.2>
#[derive(Eq, PartialEq, Deserialize_enum_str, Serialize_enum_str, Debug)]
#[serde(rename_all = "snake_case")]
pub enum OAuth2AuthErrorCode {
    InvalidRequest,
    InvalidClient,
    InvalidGrant,
    UnauthorizedClient,
    UnsupportedGrantType,
    InvalidScope,
    /// some unknown error
    #[serde(other)]
    Unknown(String),
}

#[serde_as]
#[derive(Redact, Serialize, Deserialize)]
#[serde(untagged)]
enum OAuth2AuthResponse {
    Token {
        #[redact]
        access_token: String,
        #[serde_as(as = "DurationSeconds<u64>")]
        expires_in: Duration,
        // token_type: String,
        // scope: String,
    },
    // https://www.rfc-editor.org/rfc/rfc6749#section-5.2
    Error {
        error: OAuth2AuthErrorCode,
        #[serde(skip_serializing_if = "Option::is_none")]
        error_description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error_uri: Option<String>,
    },
}

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error(transparent)]
    Network(#[from] reqwest::Error),
    // TODO give this a better message format instead of just :?ing the `Option<String>`s
    #[error("oauth error! {error} - {error_description:?} - {error_uri:?}")]
    OAuth {
        error: OAuth2AuthErrorCode,
        error_description: Option<String>,
        error_uri: Option<String>,
    },
}

impl Credentials {
    pub fn new<S1, S2>(consumer_key: S1, consumer_secret: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self {
            consumer_key: consumer_key.into(),
            consumer_secret: consumer_secret.into(),
            token: None.into(),
        }
    }

    async fn definitely_authorize(&self, http_client: &reqwest::Client) -> Result<TokenWithExpiry, AuthError> {
        let request_sent_at = Instant::now();
        // TODO make a proper serde struct for this rather than doing it this way
        let form_data = [
            ("grant_type", "client_credentials"),
            ("scope", "basic offline_access write"),
            ("client_id", &self.consumer_key),
            ("client_secret", &self.consumer_secret),
        ];
        let resp: OAuth2AuthResponse = http_client
            .post("https://api.tumblr.com/v2/oauth2/token")
            .form(&form_data)
            .send()
            .await?
            .json()
            .await?;
        match resp {
            OAuth2AuthResponse::Token {
                access_token,
                expires_in,
            } => {
                let expires_at = request_sent_at + expires_in;
                Ok(TokenWithExpiry {
                    token: BearerToken(access_token),
                    expires_at,
                })
            }
            OAuth2AuthResponse::Error {
                error,
                error_description,
                error_uri,
            } => Err(AuthError::OAuth {
                error,
                error_description,
                error_uri,
            }),
        }
    }

    /// returns an active token, authorizing if we haven't already done so or if the currently
    /// stored token has expired
    pub async fn authorize(&self, http_client: &reqwest::Client) -> Result<BearerToken, AuthError> {
        let mut guard = self.token.lock().await;
        match &*guard {
            None => {
                let new_token = self.definitely_authorize(http_client).await?;
                let ret = new_token.token.clone();
                *guard = Some(new_token);
                Ok(ret)
            },
            Some(token) => {
                if token.is_expired() {
                    let new_token = self.definitely_authorize(http_client).await?;
                    let ret = new_token.token.clone();
                    *guard = Some(new_token);
                    Ok(ret)
                } else {
                    Ok(token.token.clone())
                }
            },
        }
    }
}
