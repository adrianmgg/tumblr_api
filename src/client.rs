use serde_with::DurationSeconds;
use serde_enum_str::{Deserialize_enum_str, Serialize_enum_str};
use serde_with::serde_as;
use thiserror::Error;

use std::{fmt::Debug, time::{Instant, Duration}, sync::{Arc, Mutex}};
use veil::Redact;

// use reqwest::header::{AUTHORIZATION, ACCEPT, CONTENT_TYPE};
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use typed_builder::TypedBuilder;

use crate::{api::{ApiError, ApiResponseMeta}, npf};

#[derive(Clone)]
pub struct Client {
    inner: Arc<Mutex<ClientInner>>,
}

struct ClientInner {
    credentials: Credentials,
    http_client: reqwest::Client,
    token: Option<Arc<Token>>,
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
    }
}

#[derive(Debug)]
pub enum Token {
    OAuth2(OAuth2Token),
}

#[derive(Redact)]
pub struct OAuth2Token {
    #[redact]
    access_token: String,
    /// when the token will expire
    expires_at: Instant,
}

#[derive(Error, Debug)]
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

#[derive(Error, Debug)]
pub enum RequestError {
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[error(transparent)]
    Network(#[from] reqwest::Error),
    #[error("api error! status: {status} message: {message} errors: {errors:#?}")] // TODO better message format
    Api {
        status: i32,
        message: String,
        errors: Vec<ApiError>,
        // TODO we're capturing other_fields on the response meta, should that be included here? 
    },
}

#[derive(Debug, Deserialize)]
struct ApiResponse<RT> {
    meta: ApiResponseMeta,
    #[serde(flatten)]
    thing: ApiResponseThing<RT>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ApiResponseThing<RT> {
    Failure {
        errors: Vec<ApiError>,
    },
    Success {
        response: RT,
    }
}

#[derive(Debug)]
pub struct ApiSuccessResponse<RT> {
    pub meta: ApiResponseMeta,
    pub response: RT,
}

impl Credentials {
    async fn authorize(&self, http_client: &reqwest::Client) -> Result<Token, AuthError> {
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
                let resp: OAuth2AuthResponse = http_client.post("https://api.tumblr.com/v2/oauth2/token")
                    .form(&form_data)
                    .send()
                    .await?
                    .json()
                    .await?;
                match resp {
                    OAuth2AuthResponse::Token { access_token, expires_in } => {
                        let expires_at = request_sent_at + expires_in;
                        Ok(Token::OAuth2(OAuth2Token { access_token, expires_at }))
                    },
                    OAuth2AuthResponse::Error { error, error_description, error_uri } => {
                        Err(AuthError::OAuth { error, error_description, error_uri })
                    },
                }
            },
        }
    }
}

impl ClientInner {
    async fn get_token_and_maybe_authorize(&mut self) -> Result<Arc<Token>, AuthError> {
        match &self.token {
            Some(token) => Ok(token.clone()),
            None => {
                let token: Arc<Token> = self.credentials
                    .authorize(&self.http_client)
                    .await?
                    .into();
                self.token = Some(token.clone());
                Ok(token)
            },
        }
    }

    async fn do_request<RT, U, B>(&mut self, method: reqwest::Method, url: U, json: Option<B>) -> Result<ApiSuccessResponse<RT>, RequestError>
    where
        RT: DeserializeOwned,
        U: reqwest::IntoUrl,
        B: Serialize + Sized,
    {
        let mut request_builder = self.http_client.request(method, url);
        match self.get_token_and_maybe_authorize().await?.as_ref() {
            Token::OAuth2(token) => {
                request_builder = request_builder.bearer_auth(&token.access_token);
            },
        }
        if let Some(json) = json {
            request_builder = request_builder.json(&json);
        }

        // TODO json() wraps the serde error in a reqwest error, so maybe we should either do the decode ourself or map the error back so we can have a top level decode error type
        let resp: ApiResponse<RT> = request_builder
            .send()
            .await?
            .json()
            .await?;

        // let foobar = request_builder
        //     .send()
        //     .await?;
        // let resp_str: String = foobar.text().await?;
        // dbg!(&resp_str);
        // let resp: ApiResponse<RT> = serde_json::from_str(&resp_str).unwrap();

        match resp.thing {
            ApiResponseThing::Failure { errors } => Err(RequestError::Api {
                errors,
                status: resp.meta.status,
                message: resp.meta.msg,
            }),
            ApiResponseThing::Success { response } => Ok(ApiSuccessResponse {
                response,
                meta: resp.meta,
            }),
        }
    }
}

impl Client {
    #[must_use]
    pub fn new(credentials: Credentials) -> Self {
        Self {
            inner: Arc::new(Mutex::new(ClientInner {
                http_client: reqwest::Client::new(),
                credentials,
                token: None,
            })),
        }
    }

    // async fn setup_authorized_request<U>(&mut self, method: reqwest::Method, url: U) -> Result<reqwest::RequestBuilder, AuthError>
    // where
    //     U: reqwest::IntoUrl,
    // {
    //     let mut request_builder = self.inner.http_client.request(method, url);
    //     let x = *self.inner;
    //     // match &*self.inner.borrow_mut().get_token_and_maybe_authorize().await? {
    //     //     Token::OAuth2(token) => {
    //     //         request_builder = request_builder.bearer_auth(&token.access_token);
    //     //     },
    //     // }
    //     Ok(request_builder)
    // }

    // async fn send_api_request<RT>(/*&self,*/ request_builder: reqwest::RequestBuilder) -> Result<ApiSuccessResponse<RT>, RequestError>
    // where
    //     RT: DeserializeOwned,
    // {
    //     // TODO json() wraps the serde error in a reqwest error, so maybe we should either do the decode ourself or map the error back so we can have a top level decode error type
    //     let resp: ApiResponse<RT> = request_builder
    //         .send()
    //         .await?
    //         .json()
    //         .await?;
    //     match resp.thing {
    //         ApiResponseThing::Failure { errors } => Err(RequestError::Api {
    //             errors,
    //             status: resp.meta.status,
    //             message: resp.meta.msg,
    //         }),
    //         ApiResponseThing::Success { response } => Ok(ApiSuccessResponse {
    //             response,
    //             meta: resp.meta,
    //         }),
    //     }
    // }

    // pub async fn user_info(&mut self) -> Result<ApiSuccessResponse<crate::api::UserInfoResponse>, RequestError> {
    //     Self::send_api_request(self.setup_authorized_request(reqwest::Method::GET, "https://api.tumblr.com/v2/user/info").await?).await
    // }

    pub fn user_info(&self) -> UserInfoRequestBuilder {
        UserInfoRequestBuilder::new(self.clone())
    }

    // pub async fn create_post(&mut self, blog_identifier: &str, request: crate::api::CreatePostRequest) -> Result<ApiSuccessResponse<crate::api::CreatePostResponse>, RequestError> {
    //     Self::send_api_request(
    //         self.setup_authorized_request(reqwest::Method::POST, format!("https://api.tumblr.com/v2/blog/{blog_identifier}/posts"))
    //             .await?
    //             .json(&request)
    //     ).await
    // }

    pub fn create_post<B, C>(&self, blog_identifier: B, content: C) -> CreatePostRequestBuilder
    where
        B: Into<Box<str>>,
        C: Into<Vec<crate::npf::ContentBlock>>,
    {
        CreatePostRequestBuilder::new(self.clone(), blog_identifier.into(), content.into())
    }
}

macro_rules! builder_setter {
    ($name:ident, $type:ty) => {
        #[allow(clippy::missing_const_for_fn)]
        #[must_use]
        pub fn $name(mut self, $name: $type) -> Self {
            self.$name = Some($name);
            self
        }
    };
    ($name:ident, into $type:ty) => {
        #[allow(clippy::missing_const_for_fn)]
        #[must_use]
        pub fn $name<T>(mut self, $name: T) -> Self
        where
            T: Into<$type>,
        {
            self.$name = Some($name.into());
            self
        }
    };
}

pub struct UserInfoRequestBuilder {
    client: Client,
}

impl UserInfoRequestBuilder {
    fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn send(self) -> Result<ApiSuccessResponse<crate::api::UserInfoResponse>, RequestError>  {
        self.client
            .inner
            .lock()
            .unwrap()
            .do_request(reqwest::Method::GET, "https://api.tumblr.com/v2/user/info", Option::<String>::None)
            .await
    }

    // async fn send(mut self) -> Result</*ApiSuccessResponse<crate::api::UserInfoResponse>*/ (), RequestError>  {
    //     let mut inner = self.client.inner.lock().unwrap();
    //     let mut request_builder = inner.http_client.request(reqwest::Method::GET, "https://api.tumblr.com/v2/user/info");
    //     match &*inner.get_token_and_maybe_authorize().await? {
    //         Token::OAuth2(token) => {
    //             request_builder = request_builder.bearer_auth(&token.access_token);
    //         },
    //     }
        
    // }
}

// TODO move over the doc stuff from 
pub enum CreatePostState {
    Published,
    // TODO should we do these two as `Queue, Schedule { publish_on: ... }` or as `Queue { publish_on: Option<...> }`?
    Queue,
    Schedule {
        publish_on: time::OffsetDateTime,
    },
    Draft,
    Private,
    Unapproved,
}

// TODO figure out we want to expose the `date` field (and also like. what it even does lmao)
pub struct CreatePostRequestBuilder {
    client: Client,
    blog_identifier: Box<str>,
    content: Vec<crate::npf::ContentBlock>,
    tags: Option<Box<str>>,
    // TODO should we skip the Option<> and just have this be set to Published by default?
    initial_state: Option<CreatePostState>,
    source_url: Option<Box<str>>,
}

impl CreatePostRequestBuilder {
    fn new(client: Client, blog_identifier: Box<str>, content: Vec<crate::npf::ContentBlock>) -> Self {
        Self {
            client,
            blog_identifier,
            content,
            tags: None,
            initial_state: None,
            source_url: None,
        }
    }

    builder_setter!(tags, into Box<str>);
    builder_setter!(initial_state, CreatePostState);
    builder_setter!(source_url, into Box<str>);

    pub async fn send(self) -> Result<ApiSuccessResponse<crate::api::CreatePostResponse>, RequestError>  {
        self.client
            .inner
            .lock()
            .unwrap()
            .do_request(reqwest::Method::POST, format!("https://api.tumblr.com/v2/blog/{}/posts", self.blog_identifier), Some(crate::api::CreatePostRequest {
                content: self.content,
                state: match self.initial_state {
                    None => None,
                    Some(CreatePostState::Draft) => Some(crate::api::CreatePostState::Draft),
                    Some(CreatePostState::Private) => Some(crate::api::CreatePostState::Private),
                    Some(CreatePostState::Published) => Some(crate::api::CreatePostState::Published),
                    Some(CreatePostState::Unapproved) => Some(crate::api::CreatePostState::Unapproved),
                    Some(CreatePostState::Queue | CreatePostState::Schedule { publish_on: _ }) => Some(crate::api::CreatePostState::Queue),
                },
                publish_on: match self.initial_state {
                    Some(CreatePostState::Schedule { publish_on }) => Some(publish_on.format(&time::format_description::well_known::Iso8601::DEFAULT).unwrap()),  // TOOD handle properly instead of unwrapping // TODO also the format isn't right i think b/c these are 400.8001ing
                    _ => None,
                },
                date: None,
                tags: self.tags.map(std::convert::Into::into), // TODO
                source_url: self.source_url.map(std::convert::Into::into), // TODO
                send_to_twitter: None,
                is_private: None,
                slug: None,
                interactability_reblog: None,
            }))
            .await
    }
}
