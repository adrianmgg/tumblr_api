use crate::npf;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

// https://www.tumblr.com/docs/en/api/v2#postspost-id---fetching-a-post-neue-post-format
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
// #[serde(deny_unknown_fields)]
pub struct NPFPost {
    /// The short name used to uniquely identify a blog
    pub blog_name: String,
    /// The post's unique ID
    #[serde(flatten, with = "post_id_serde")]
    pub id: i64,
    /// "The post's unique "genesis" ID as a String. Only available to the post owner in certain circumstances."
    /// (longer explanation [here](https://www.tumblr.com/docs/en/api/v2#posts--retrieve-published-posts), in the footnote at the bottom of the "Response" section)
    pub genesis_post_id: Option<String>,
    /// "The location of the post"
    pub post_url: String,
    /// "The type of post"
    ///
    /// **currently not actually checked -- since we're only supporting NPF so far anyways this should only ever be "blocks"**
    #[serde(rename = "type")]
    pub post_type: String,
    pub timestamp: i64, // TODO "The time of the post, in seconds since the epoch"
    pub date: String,   // TODO "The GMT date and time of the post, as a string"
    // /// "The post format"
    // (only present on old-style posts)
    // pub format: PostFormat,
    /// "The key used to reblog this post, see the `/post/reblog` method"
    pub reblog_key: String,
    /// "Tags applied to the post"
    pub tags: Vec<String>,
    // TODO "bookmarklet", "mobile" old-style only?
    /// information about the source of the content.
    /// "Exists only if there's a content source."
    #[serde(flatten)]
    pub source: Option<SourceInfo>,
    /// "Indicates if a user has already liked a post or not.
    ///  Exists only if the request is fully authenticated with OAuth."
    pub liked: bool,
    /// "Indicates the current state of the post"
    pub state: PostState,
    /// "Indicates whether the post is stored in the Neue Post Format"
    pub is_blocks_post_format: bool,
    /// (undocumented) the post's original type? only present on npf posts.
    pub original_type: String,
    /// (undocumented?)
    // wait is this one actually not mentioned in the docs anywhere?
    pub blog: Blog,
    #[serde(flatten)]
    pub blaze_info: BlazeInfo,
    /// "Short text summary to the end of the post URL"
    pub slug: String,
    /// "Short url for the post"
    pub short_url: String,
    pub summary: String,
    pub should_open_in_legacy: bool,
    // TODO type?
    pub recommended_source: serde_json::Value,
    // TODO type?
    pub recommended_color: serde_json::Value,
    pub followed: bool,
    // TODO - should this be nullable? (check what a no-notes post gives)
    pub note_count: i32,
    pub content: Vec<super::npf::ContentBlock>,
    // TODO
    pub layout: Vec<serde_json::Value>,
    // TODO
    pub trail: Vec<serde_json::Value>,
    #[serde(flatten)]
    interactability: InteractabilityInfo,
    pub display_avatar: bool,
    // TODO specifically when does this one show up? most posts didnt have it
    pub is_pinned: Option<bool>,
    #[serde(flatten)]
    pub ask_info: Option<AskInfo>,
    #[serde(flatten, with = "post_submission_info_serde")]
    pub submission_info: Option<SubmissionInfo>,
    /// fields not captured by anything else
    #[serde(flatten)]
    pub other_fields: serde_json::Map<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PostFormat {
    HTML,
    Markdown,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PostState {
    Published,
    Queued,
    Draft,
    Private,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Blog {
    name: String,
    title: String,
    description: String,
    url: String,
    // TODO parse tumblr uuids?
    uuid: String,
    // TODO parse date
    updated: i64,
    // TODO ?
    tumblrmart_accessories: serde_json::Map<String, serde_json::Value>,
    can_show_badges: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ReblogInteractability {
    // TODO is this all the variants?
    Everyone,
    Noone,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct AskInfo {
    pub asking_name: String,
    pub asking_url: String,
    pub asking_avatar: Vec<npf::MediaObject>,
}

// TODO make this an enum on anon / not anon ?
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct SubmissionInfo {
    /// "Author of post, only available when submission is not anonymous"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_author_is_adult: Option<bool>,
    /// "Name on an anonymous submission"
    // TODO do these two always occur together?
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anonymous_name: Option<String>,
    /// "Email on an anonymous submission"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anonymous_email: Option<String>,
}

mod post_submission_info_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::SubmissionInfo;

    #[derive(Serialize, Deserialize)]
    struct Shim {
        is_submission: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        post_author: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        post_author_is_adult: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        anonymous_name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        anonymous_email: Option<String>,
    }

    #[derive(Serialize)]
    struct ShimEmpty {}

    pub(super) fn serialize<S>(
        opt: &Option<SubmissionInfo>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match opt {
            None => ShimEmpty {}.serialize(serializer),
            Some(SubmissionInfo {
                post_author,
                post_author_is_adult,
                anonymous_name,
                anonymous_email,
            }) => Shim {
                is_submission: true,
                // TODO make a serialize-specific version of the struct with `&str`s to avoid these?
                post_author: post_author.clone(),
                post_author_is_adult: *post_author_is_adult,
                anonymous_name: anonymous_name.clone(),
                anonymous_email: anonymous_email.clone(),
            }
            .serialize(serializer),
        }
    }

    pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<Option<SubmissionInfo>, D::Error>
    where
        D: Deserializer<'de>,
    {
        match Option::<Shim>::deserialize(deserializer)? {
            Some(Shim {
                is_submission: _,
                post_author,
                post_author_is_adult,
                anonymous_name,
                anonymous_email,
            }) => Ok(Some(SubmissionInfo {
                post_author,
                post_author_is_adult,
                anonymous_name,
                anonymous_email,
            })),
            None => Ok(None),
        }
    }
}

mod post_id_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Serialize, Deserialize)]
    struct IdShim {
        /// (see [`super::NPFPost::id`])
        id: i64,
        /// "The post's unique ID as a String, for clients that don't support 64-bit integers"
        id_string: String,
    }

    pub(super) fn serialize<S>(id: &i64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        IdShim {
            id: id.to_owned(),
            id_string: id.to_string(),
        }
        .serialize(serializer)
    }

    pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<i64, D::Error>
    where
        D: Deserializer<'de>,
    {
        IdShim::deserialize(deserializer).map(|shim| shim.id)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct SourceInfo {
    /// "The URL for the source of the content (for quotes, reblogs, etc.)"
    pub source_url: String,
    /// "The title of the source site"
    pub source_title: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct BlazeInfo {
    pub is_blazed: bool,
    pub is_blaze_pending: bool,
    pub can_ignite: bool,
    pub can_blaze: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct InteractabilityInfo {
    pub can_like: bool,
    pub interactability_reblog: ReblogInteractability,
    pub can_reblog: bool,
    pub can_send_in_message: bool,
    pub can_reply: bool,
}

// https://www.tumblr.com/docs/en/api/v2#userinfo--get-a-users-information#errors-and-error-subcodes
#[derive(Debug, Deserialize, Serialize)]
pub struct ApiError {
    // TODO should title/code be `Option`al?
    title: String,
    code: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiResponseMeta {
    /// "The 3-digit HTTP Status-Code (e.g., 200)"
    pub status: i32,
    /// "The HTTP Reason-Phrase (e.g., OK)"
    pub msg: String,
    /// unknown/unhandled fields
    #[serde(flatten)]
    pub other_fields: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserInfoResponse {
    pub user: UserInfo,
    /// unknown/unhandled fields
    #[serde(flatten)]
    pub other_fields: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserInfo {
    /// "The number of blogs the user is following"
    pub following: i64,
    /// "The default posting format - html, markdown, or raw"
    pub default_post_format: String, // TODO enum
    /// "The user's tumblr short name"
    pub name: String,
    /// "The total count of the user's likes"
    pub likes: i64,
    /// "Each item is a blog the user has permissions to post to"
    pub blogs: Vec<UserInfoBlog>,
    /// unknown/unhandled fields
    #[serde(flatten)]
    pub other_fields: serde_json::Map<String, serde_json::Value>,
}

// TODO this can probably be merged with the other `Blog`
// TODO but if not give this a better name
#[derive(Debug, Deserialize, Serialize)]
pub struct UserInfoBlog {
    /// "the short name of the blog"
    pub name: String,
    /// "the URL of the blog"
    pub url: String,
    /// "the title of the blog"
    pub title: String,
    /// "indicates if this is the user's primary blog"
    pub primary: bool,
    /// "total count of followers for this blog"
    pub followers: i64,
    /// "indicate if posts are tweeted auto, Y, N"
    pub tweet: String, // TODO to bool
    /// "indicates whether a blog is public or private"
    #[serde(rename = "type")]
    pub blog_type: String, // TODO enum
    /// unknown/unhandled fields
    #[serde(flatten)]
    pub other_fields: serde_json::Map<String, serde_json::Value>,
}

// https://www.tumblr.com/docs/en/api/v2#posts---createreblog-a-post-neue-post-format
#[derive(Debug, Deserialize, Serialize, TypedBuilder)]
pub struct CreatePostRequest {
    /// "An array of NPF content blocks to be used to make the post; in a reblog, this is any content you want to add."
    pub content: Vec<crate::npf::ContentBlock>,
    // /// "An array of NPF layout objects to be used to lay out the post content."
    // pub layout: Option<Vec<tumblr_api::npf::LayoutObject>>, // TODO
    /// "The initial state of the new post, such as "published" or "queued"."
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<CreatePostState>,
    /// "The exact future date and time (ISO 8601 format) to publish the post, if desired. This parameter will be ignored unless the state parameter is "queue"."
    #[builder(default, setter(into, strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publish_on: Option<String>, // TODO some other type
    /// "The exact date and time (ISO 8601 format) in the past to backdate the post, if desired. This backdating does not apply to when the post shows up in the Dashboard."
    #[builder(default, setter(into, strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>, // TODO some other type
    /// "A comma-separated list of tags to associate with the post."
    #[builder(default, setter(into, strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,
    /// "A source attribution for the post content."
    #[builder(default, setter(into, strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    /// "Whether or not to share this via any connected Twitter account on post publish. Defaults to the blog's global setting."
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_to_twitter: Option<bool>,
    /// "Whether this should be a private answer, if this is an answer."
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_private: Option<bool>,
    /// "A custom URL slug to use in the post's permalink URL"
    #[builder(default, setter(into, strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    /// "Who can interact with this when reblogging"
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interactability_reblog: Option<ReblogInteractability>,
}
// TODO ^ currently just has the making a new post stuff, same endpoint is also the way to do reblogs.
//      maybe best to do it as an enum of new post / reblog? since which fields are required is different
//      between the two
// TODO should we add `other_fields`s to requests too? or just response stuff

/// <https://www.tumblr.com/docs/en/api/v2#note-about-post-states>
/// "Posts can be in the following 'states' as indicated in requests to the post creation/editing endpoints"
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CreatePostState {
    /// "the post should be publicly published immediately"
    Published,
    /// "the post should be added to the end of the blog's post queue"
    Queue,
    /// "the post should be saved as a draft"
    Draft,
    /// "the post should be privately published immediately"
    Private,
    /// "the post is a new submission"
    Unapproved,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreatePostResponse {
    // TODO - "intentionally a string instead of an integer for 32bit device compatibility" - should make it an int
    /// "the id of the created post"
    id: String,
    // TODO - field `state` - observed values: "published", "draft", "private", "queued"
    // TODO - field `display_text` - observed values: (a string)
    /// unknown/unhandled fields
    #[serde(flatten)]
    pub other_fields: serde_json::Map<String, serde_json::Value>,
}
