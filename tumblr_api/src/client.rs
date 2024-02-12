// TODO (see below)
//! client (TODO: one-line description here)
//!
//! # Examples
//!
//! creating a client
//! ```no_run
//! use tumblr_api::{client::Client, auth::Credentials};
//! let client = Client::new(Credentials::new("your consumer key", "your consumer secret"));
//! ```
//!
// TODO (see below)
//! TODO example name here
//! ```no_run
//! # use tumblr_api::{client::Client, auth::Credentials};
//! # #[tokio::main]
//! # async fn main() -> Result<(), tumblr_api::client::RequestError> {
//! # let client = Client::new(Credentials::new("your consumer key", "your consumer secret"));
//! let limits = client.api_limits().send().await?;
//! println!("you are {} posts away from hitting post limit", limits.user.posts.remaining);
//! # Ok(())
//! # }
//! ```
//!
//! creating a post
//! ```no_run
//! # use tumblr_api::{client::Client, auth::Credentials};
//! use tumblr_api::npf;
//! # #[tokio::main]
//! # async fn main() -> Result<(), tumblr_api::client::RequestError> {
//! # let client = Client::new(Credentials::new("your consumer key", "your consumer secret"));
//! client
//!     .create_post(
//!         "blog-name",
//!         vec![npf::ContentBlockText::builder("hello world").build()],
//!     )
//!     .send()
//!     .await?;
//! # Ok(())
//! # }
//! ```

use tumblr_api_derive::Builder;

use std::borrow::Cow;
use std::{fmt::Debug, sync::Arc};

use serde::{de::DeserializeOwned, Serialize};

use crate::{
    api::{Response, SuccessResponse},
    auth::{Credentials, Error as AuthError},
};

#[derive(Clone)]
pub struct Client {
    inner: Arc<ClientInner>,
}

struct ClientInner {
    credentials: Credentials,
    http_client: reqwest::Client,
}

#[derive(thiserror::Error, Debug)]
pub enum RequestError {
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[error(transparent)]
    Network(#[from] reqwest::Error),
    #[error(transparent)]
    Deserializing(#[from] serde_json::Error),
    #[error(transparent)]
    Api(#[from] crate::api::ResponseError),
}

impl ClientInner {
    async fn do_request<RT, U, B>(
        &self,
        method: reqwest::Method,
        url: U,
        json: Option<B>,
        parts: Option<Vec<(Cow<'static, str>, reqwest::multipart::Part)>>,
    ) -> Result<SuccessResponse<RT>, RequestError>
    where
        RT: DeserializeOwned,
        U: reqwest::IntoUrl,
        B: Serialize + Sized,
    {
        let mut request_builder = self.http_client.request(method, url);
        let token = self.credentials.authorize(&self.http_client).await?;
        request_builder = request_builder.bearer_auth(token);
        if let Some(parts) = parts {
            let mut form = reqwest::multipart::Form::new();
            if let Some(json) = json {
                let body_part =
                    reqwest::multipart::Part::text(serde_json::to_string(&json).unwrap()) // TODO handle instead of unwrapping
                        .mime_str("application/json")
                        .unwrap(); // TODO handle instead of unwrapping?
                form = form.part("json", body_part);
                for (part_id, part) in parts {
                    form = form.part(part_id, part);
                }
            }
            request_builder = request_builder.multipart(form);
        } else if let Some(json) = json {
            request_builder = request_builder.json(&json);
        }

        let bytes = request_builder.send().await?.bytes().await?;
        let resp: Response<RT> = serde_json::from_slice(&bytes)?;
        let resp: crate::api::ResponseResult<RT> = resp.into();
        resp.map_err(RequestError::from)
    }
}

impl Client {
    #[must_use]
    pub fn new(credentials: Credentials) -> Self {
        Self {
            inner: Arc::new(ClientInner {
                http_client: reqwest::Client::new(),
                credentials,
            }),
        }
    }

    #[must_use]
    pub fn user_info(&self) -> UserInfoRequestBuilder {
        UserInfoRequestBuilder::new(self.clone())
    }

    #[must_use]
    pub fn create_post<B, C>(&self, blog_identifier: B, content: C) -> CreatePostRequestBuilder
    where
        B: Into<Box<str>>,
        C: Into<Vec<crate::npf::ContentBlock>>,
    {
        CreatePostRequestBuilder::new(self.clone(), blog_identifier.into(), content.into())
    }

    #[must_use]
    pub fn api_limits(&self) -> ApiLimitsRequestBuilder {
        ApiLimitsRequestBuilder::new(self.clone())
    }
}

#[derive(Builder)]
#[builder(ctor(vis = ""))]
pub struct UserInfoRequestBuilder {
    #[builder(set(ctor))]
    client: Client,
}

impl UserInfoRequestBuilder {
    pub async fn send(self) -> Result<crate::api::UserInfoResponse, RequestError> {
        self.client
            .inner
            .do_request(
                reqwest::Method::GET,
                "https://api.tumblr.com/v2/user/info",
                Option::<String>::None,
                None,
            )
            .await
            .map(|r| r.response)
    }
}

// TODO move over the doc stuff from
#[derive(Debug, PartialEq, Eq)]
pub enum CreatePostState {
    Published,
    // TODO should we do these two as `Queue, Schedule { publish_on: ... }` or as `Queue { publish_on: Option<...> }`?
    Queue,
    Schedule { publish_on: time::OffsetDateTime },
    Draft,
    Private,
    Unapproved,
}

// TODO figure out we want to expose the `date` field (and also like. what it even does lmao)
#[derive(Builder)]
#[builder(ctor(vis = ""))]
pub struct CreatePostRequestBuilder {
    #[builder(set(ctor))]
    client: Client,
    #[builder(set(ctor))]
    blog_identifier: Box<str>,
    #[builder(set(ctor))]
    content: Vec<crate::npf::ContentBlock>,
    #[builder(set(setter(
        into,
        strip_option,
        doc = "set the tags the created post will have. corresponds to [`api::CreatePostRequest::tags`][crate::api::CreatePostRequest::tags]"
    )))]
    tags: Option<Box<str>>,
    // TODO should we skip the Option<> and just have this be set to Published by default?
    #[builder(set(setter(into, strip_option)))]
    initial_state: Option<CreatePostState>,
    #[builder(set(setter(into, strip_option)))]
    source_url: Option<Box<str>>,
    // TODO need to add 'call method on it' set mode (push in this case), and add a way to set the default used explicitly
    #[builder(set = "no")]
    attachments: Vec<CreatePostAttachment>,
}

struct CreatePostAttachment {
    stream: reqwest::Body,
    mime_type: Box<str>,
    identifier: Cow<'static, str>,
}

impl CreatePostRequestBuilder {
    #[must_use]
    pub fn add_attachment<S1, S2>(
        mut self,
        stream: reqwest::Body,
        mime_type: S1,
        identifier: S2,
    ) -> Self
    where
        S1: Into<Box<str>>,
        S2: Into<Cow<'static, str>>,
    {
        self.attachments.push(CreatePostAttachment {
            stream,
            mime_type: mime_type.into(),
            identifier: identifier.into(),
        });
        self
    }

    pub async fn send(self) -> Result<crate::api::CreatePostResponse, RequestError> {
        // the api takes state & publish_on as two different properties,
        //  where publish_on is only valid when the state is queue & that represents a scheduled post.
        //  we instead expose it as a single enum where queue & schedule are different variants,
        //  so we need to map that back to the two separate fields that the api wants.
        let (state, publish_on) = match self.initial_state {
            None => (None, None),
            Some(CreatePostState::Draft) => (Some(crate::api::CreatePostState::Draft), None),
            Some(CreatePostState::Private) => (Some(crate::api::CreatePostState::Private), None),
            Some(CreatePostState::Published) => {
                (Some(crate::api::CreatePostState::Published), None)
            }
            Some(CreatePostState::Unapproved) => {
                (Some(crate::api::CreatePostState::Unapproved), None)
            }
            Some(CreatePostState::Queue) => (Some(crate::api::CreatePostState::Queue), None),
            Some(CreatePostState::Schedule { publish_on }) => (
                Some(crate::api::CreatePostState::Queue),
                Some(
                    // TODO handle properly instead of unwrapping
                    // TODO also the format isn't right i think b/c these were 400.8001ing last time i checked
                    publish_on
                        .format(&time::format_description::well_known::Iso8601::DEFAULT)
                        .unwrap(),
                ),
            ),
        };
        self.client
            .inner
            .do_request(
                reqwest::Method::POST,
                format!(
                    "https://api.tumblr.com/v2/blog/{}/posts",
                    self.blog_identifier
                ),
                Some(crate::api::CreatePostRequest {
                    content: self.content,
                    state,
                    publish_on,
                    date: None,
                    tags: self.tags.map(std::convert::Into::into), // TODO
                    source_url: self.source_url.map(std::convert::Into::into), // TODO
                    send_to_twitter: None,
                    is_private: None,
                    slug: None,
                    interactability_reblog: None,
                }),
                Some(
                    self.attachments
                        .into_iter()
                        .map(|attachment| {
                            let part = reqwest::multipart::Part::stream(attachment.stream)
                                // tumblr requires a filename but doesn't actually check it so we just put something there
                                .file_name("a")
                                .mime_str(&attachment.mime_type)
                                .unwrap(); // TODO handle instead of just unwrapping
                            (attachment.identifier, part)
                        })
                        .collect(),
                ),
            )
            .await
            .map(|r| r.response)
    }
}

#[derive(Builder)]
#[builder(ctor(vis = ""))]
pub struct ApiLimitsRequestBuilder {
    #[builder(set(ctor))]
    client: Client,
}

impl ApiLimitsRequestBuilder {
    pub async fn send(self) -> Result<crate::api::LimitsResponse, RequestError> {
        self.client
            .inner
            .do_request(
                reqwest::Method::GET,
                "https://api.tumblr.com/v2/user/limits",
                Option::<String>::None,
                None,
            )
            .await
            .map(|r| r.response)
    }
}
