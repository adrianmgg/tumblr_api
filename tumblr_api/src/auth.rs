//! tumblr api authorization
//!
//! # Examples
//!
//! ## manually authorizing
//! First, create a [`Credentials`] from your api keys.
//! (just doing this will **not** make any api calls)
//! ```
//! use tumblr_api::auth::Credentials;
//! let credentials = Credentials::new("your consumer key", "your consumer secret");
//! ```
//! Then, when you need a token for interacting with the api, get one via
//! [`Credentials::authorize`].
//!
//! ```no_run
//! # use tumblr_api::auth::Credentials;
//! # #[tokio::main]
//! # async fn main() -> anyhow::Result<()> {
//! # let credentials = Credentials::new("your consumer key", "your consumer secret");
//! let reqwest_client = reqwest::Client::new();
//! let token = credentials.authorize(&reqwest_client).await?;
//! # Ok(())
//! # }
//! ```
//! <div class="warning">
//!
//! You should [`authorize`][`Credentials::authorize`] once for each API request you send,
//! ideally immemdiately before sending the request.
//!
//! [`Credentials`] handles caching the most recent token and only actually re-authorizing when the
//! old token has expired, so there's no need to cache the token yourself, and more importantly, no
//! gurantee that the returned token will be valid for very long.
//! </div>
//!
//! ## reusing tokens across runs
//! this is not yet implemented but is a planned feature for `1.0`, and will probably involve
//! adding [`Deserialize`] and [`Serialize`] implementations to [`Credentials`], so that programs
//! that aren't long-running (e.g. a CLI tool for creating posts) can still benefit from
//! [`Credentials`] only re-authorizing when needed.

use std::{
    fmt,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use serde_enum_str::{Deserialize_enum_str, Serialize_enum_str};
use serde_with::{serde_as, DurationSeconds};
use veil::Redact;

/// API credentials, which can be used to acquire an access token
///
/// [`Credentials`]'s [`Debug`][`fmt::Debug`] implementation will be redacted (via [`veil`]), so
/// you can log it without worrying about revealing your keys.
#[derive(Redact)]
pub struct Credentials {
    #[redact]
    consumer_key: String,
    #[redact]
    consumer_secret: String,
    token: async_lock::Mutex<Option<TokenWithExpiry>>,
}

/// access token for the API
///
/// <div class="warning">
///
/// [`BearerToken`]'s [`Debug`][`fmt::Debug`] implementation will be redacted (via [`veil`]).
///
/// To get the actual content of the token, use either [`Display`][`fmt::Display`] or
/// [`String::from`] / [`Into<String>`].
/// </div>
#[derive(Redact, Clone, Serialize, Deserialize)]
pub struct BearerToken(#[redact] String);

impl From<BearerToken> for String {
    fn from(val: BearerToken) -> Self {
        val.0
    }
}

impl fmt::Display for BearerToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
struct TokenWithExpiry {
    token: BearerToken,
    /// when the token will expire
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
        access_token: BearerToken,
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
pub enum Error {
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

    async fn definitely_authorize(
        &self,
        http_client: &reqwest::Client,
    ) -> Result<TokenWithExpiry, Error> {
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
                    token: access_token,
                    expires_at,
                })
            }
            OAuth2AuthResponse::Error {
                error,
                error_description,
                error_uri,
            } => Err(Error::OAuth {
                error,
                error_description,
                error_uri,
            }),
        }
    }

    /// returns an active token, authorizing if we haven't already done so or if the currently
    /// stored token has expired
    pub async fn authorize(&self, http_client: &reqwest::Client) -> Result<BearerToken, Error> {
        let mut guard = self.token.lock().await;
        match &*guard {
            None => {
                let new_token = self.definitely_authorize(http_client).await?;
                let ret = new_token.token.clone();
                *guard = Some(new_token);
                Ok(ret)
            }
            Some(token) => {
                if token.is_expired() {
                    let new_token = self.definitely_authorize(http_client).await?;
                    let ret = new_token.token.clone();
                    *guard = Some(new_token);
                    Ok(ret)
                } else {
                    Ok(token.token.clone())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // `BearerToken`s must deserialize from a single json string, otherwise the oauth responses will
    // fail to deserialize.
    #[test]
    fn test_bearertoken_deserialize() {
        let token: BearerToken = serde_json::from_value(serde_json::json!("hello world"))
            .expect("BearerToken deserialize failed");
        let token_str: String = token.into();
        assert_eq!(token_str, "hello world".to_string());
    }
}
