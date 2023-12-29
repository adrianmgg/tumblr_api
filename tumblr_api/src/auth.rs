use veil::Redact;

pub trait TokenProvider<Err> {
    // TODO give this a better name
    async fn get_token(&self) -> Result<AccessToken, Err>;
}

#[derive(Redact, Clone, Eq, PartialEq)]
pub struct AccessToken {
    #[redact]
    value: String,
}

pub mod simple_oauth2 {
    use std::time::{Duration, Instant};

    use serde::{Deserialize, Serialize};
    use serde_enum_str::{Deserialize_enum_str, Serialize_enum_str};
    use serde_with::{serde_as, DurationSeconds};
    use veil::Redact;

    use super::{AccessToken, TokenProvider};

    #[derive(Redact)]
    pub struct OAuth2AuthHandler {
        #[redact]
        pub consumer_key: String,
        #[redact]
        pub consumer_secret: String,
        // TODO should i be pub?
        pub http_client: reqwest::Client,
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

    impl OAuth2AuthHandler {
        pub async fn authorize(&self) -> Result<AccessToken, AuthError> {
            let request_sent_at = Instant::now();
            // TODO make a proper serde struct for this rather than doing it this way
            let form_data = [
                ("grant_type", "client_credentials"),
                ("scope", "basic offline_access write"),
                ("client_id", &self.consumer_key),
                ("client_secret", &self.consumer_secret),
            ];
            let resp: OAuth2AuthResponse = self
                .http_client
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
                    Ok(AccessToken {
                        value: access_token,
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
    }

    impl TokenProvider<AuthError> for OAuth2AuthHandler {
        async fn get_token(&self) -> Result<AccessToken, AuthError> {
            self.authorize().await
        }
    }
}
