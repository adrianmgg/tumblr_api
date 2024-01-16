use std::time::{Duration, Instant};

use serde::{Serialize, Deserialize};
use serde_enum_str::{Deserialize_enum_str, Serialize_enum_str};
use serde_with::{serde_as, DurationSeconds};
use veil::Redact;


#[derive(Debug)]
pub enum Credentials {
    // OAuth1(OAuth1Credentials),
    OAuth2(OAuth2Credentials),
}

#[derive(Redact)]
pub struct OAuth2Credentials {
    #[redact]
    pub consumer_key: String,
    #[redact]
    pub consumer_secret: String,
}

impl From<OAuth2Credentials> for Credentials {
    fn from(value: OAuth2Credentials) -> Self {
        Self::OAuth2(value)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
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
        access_token: Box<str>,
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

#[derive(Debug, Clone)]
pub enum Token {
    OAuth2(OAuth2Token),
}

#[derive(Redact, Clone)]
pub struct OAuth2Token {
    #[redact]
    access_token: Box<str>,
    /// when the token will expire
    expires_at: Instant,
}

impl Token {
    // TODO ok yeah this should maybe be a Token trait rather than an enum
    pub(crate) fn is_expired(&self) -> bool {
        match self {
            Token::OAuth2(token) => {
                let now = Instant::now();
                // TODO this should probably be done with some extra tolerance, if we're within a
                //      couple seconds of expiring we should probably just count it
                now >= token.expires_at
            },
        }
    }
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
    pub fn new_oauth2<S1, S2>(consumer_key: S1, consumer_secret: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self::OAuth2(OAuth2Credentials {
            consumer_key: consumer_key.into(),
            consumer_secret: consumer_secret.into(),
        })
    }

    pub(crate) async fn authorize(&self, http_client: &reqwest::Client) -> Result<Token, AuthError> {
        match self {
            Self::OAuth2(creds) => {
                let request_sent_at = Instant::now();
                // TODO make a proper serde struct for this rather than doing it this way
                let form_data = [
                    ("grant_type", "client_credentials"),
                    ("scope", "basic offline_access write"),
                    ("client_id", &creds.consumer_key),
                    ("client_secret", &creds.consumer_secret),
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
                        Ok(Token::OAuth2(OAuth2Token {
                            access_token,
                            expires_at,
                        }))
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
        }
    }
}
