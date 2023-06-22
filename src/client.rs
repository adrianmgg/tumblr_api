// use oauth2::{
//     basic::{BasicClient, BasicTokenType},
//     AuthUrl, ClientId, ClientSecret, Scope, TokenResponse, TokenUrl,
// };
use thiserror::Error;

use std::fmt::Debug;
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

#[derive(Redact, Serialize, Deserialize)]
#[serde(untagged)]
enum OAuth2AuthResponse {
    Token {
        #[redact]
        access_token: String,
        expires_in: i32,
        // token_type: String,
        // scope: String,
    },
    Error {
        error: String,
        error_description: String,
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
    // TODO add expires_in/expires_at/whatever
}

#[derive(Error, Debug)]
pub enum AuthError {
    #[error(transparent)]
    NetworkError(#[from] reqwest::Error),
    #[error("oauth error! {error} - {error_description}")]
    OAuthError {
        error: String,
        error_description: String,
    },
}

impl Client {
    pub fn new(credentials: Credentials) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            credentials,
        }
    }

    pub async fn authorize(&mut self) -> Result<(), AuthError> {
        match &self.credentials {
            Credentials::OAuth2(creds) => {
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
                        self.
                    },
                    OAuth2AuthResponse::Error { error, error_description } => todo!(),
                }
            },
        }
    }
}
