use serde_with::DurationSeconds;
use serde_enum_str::{Deserialize_enum_str, Serialize_enum_str};
use serde_with::serde_as;
// use oauth2::{
//     basic::{BasicClient, BasicTokenType},
//     AuthUrl, ClientId, ClientSecret, Scope, TokenResponse, TokenUrl,
// };
use thiserror::Error;

use std::{fmt::Debug, time::{Instant, Duration}};
use veil::Redact;

// use reqwest::header::{AUTHORIZATION, ACCEPT, CONTENT_TYPE};
use serde::{Serialize, Deserialize};
use typed_builder::TypedBuilder;

#[derive(Debug)]
pub struct Client {
    credentials: Credentials,
    http_client: reqwest::Client,
    token: Option<Token>,
}

#[derive(Debug)]
pub enum Credentials {
    // OAuth1(OAuth1Credentials),
    OAuth2(OAuth2Credentials),
}

// TODO redact this
#[derive(Redact, TypedBuilder)]
#[builder(
    build_method(into),
    field_defaults(setter(into)),
)]
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

/// possible values of `error` in OAuth2 error response, see <https://www.rfc-editor.org/rfc/rfc6749#section-5.2>
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
    }
}

#[derive(Debug)]
enum Token {
    OAuth2(OAuth2Token),
}

#[derive(Redact)]
struct OAuth2Token {
    #[redact]
    access_token: String,
    /// when the token will expire
    expires_at: Instant,
}

#[derive(Error, Debug)]
pub enum AuthError {
    #[error(transparent)]
    NetworkError(#[from] reqwest::Error),
    // TODO give this a better message format instead of just :?ing the `Option<String>`s
    #[error("oauth error! {error} - {error_description:?} - {error_uri:?}")]
    OAuthError {
        error: OAuth2AuthErrorCode,
        error_description: Option<String>,
        error_uri: Option<String>,
    },
}

impl Client {
    /// ```no_run
    /// # use tumblr_api::client::{Client, OAuth2Credentials};
    /// /// create a client with oauth2 credentials
    /// let client = Client::new(
    ///     OAuth2Credentials::builder()
    ///         .consumer_key("<consumer key here>")
    ///         .consumer_secret("<consumer secret here>")
    ///         .build()
    /// );
    /// ```
    pub fn new(credentials: Credentials) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            credentials,
            token: None,
        }
    }

    async fn get_token_or_authorize(&mut self) -> Result<&Token, AuthError> {
        match &self.credentials {
            Credentials::OAuth2(creds) => {
                let request_sent_at = Instant::now();
                // TODO make a proper serde struct for this rather than doing it this way
                let foo = [
                    ("grant_type", "client_credentials"),
                    ("scope", "basic offline_access write"),
                    ("client_id", &creds.consumer_key),
                    ("client_secret", &creds.consumer_secret),
                ];
                let resp: OAuth2AuthResponse = self.http_client.post("https://api.tumblr.com/v2/oauth2/token")
                    .form(&foo)
                    .send()
                    .await?
                    .json()
                    .await?;
                match resp {
                    OAuth2AuthResponse::Token { access_token, expires_in } => {
                        let expires_at = request_sent_at + expires_in;
                        let token = Token::OAuth2(OAuth2Token { access_token, expires_at });
                        self.token = Some(token);
                        Ok(&token)
                    },
                    OAuth2AuthResponse::Error { error, error_description, error_uri } => {
                        Err(AuthError::OAuthError { error, error_description, error_uri })
                    },
                }
            },
        }
    }

    pub async fn authorize(&mut self) -> Result<(), AuthError> {
        self.get_token_or_authorize()
            .await
            .map(|_| ())
    }
}
