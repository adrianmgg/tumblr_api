use serde::{Deserialize, Serialize};

// <https://www.tumblr.com/docs/en/api/v2#posts--retrieve-published-posts> (the section under "Response" titled "Fields available for all Post types:")

// types listed at https://www.tumblr.com/docs/en/api/v2#postspost-id---fetching-a-post-neue-post-format, in the "type" row of the table under "Response"
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PostType {
    /// NPF format post
    // "If formatting as NPF, the type will be blocks"
    #[serde(rename = "blocks")]
    NPF,
    // "if formatting as legacy, the type will be one of the original legacy types (text, photo, quote, chat, link, video, audio)"
    Legacy(LegacyPostType),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LegacyPostType {
    Text,
    Photo,
    Quote,
    Chat,
    Link,
    Video,
    Audio,
}

// https://www.tumblr.com/docs/en/api/v2#postspost-id---fetching-a-post-neue-post-format
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NPFPost {
    /// The short name used to uniquely identify a blog
    pub blog_name: String,
    /// The post's unique ID
    pub id: i64,
    // (skip id_string)
    /// "The post's unique "genesis" ID as a String. Only available to the post owner in certain circumstances."
    /// (longer explanation [here](https://www.tumblr.com/docs/en/api/v2#posts--retrieve-published-posts), in the footnote at the bottom of the "Response" section)
    pub genesis_post_id: Option<String>,
    /// "The location of the post"
    pub post_url: String,
    /// "The type of post"
    #[serde(rename = "type")]
    pub post_type: String, // TODO handle this properly
    pub timestamp: i64, // TODO "The time of the post, in seconds since the epoch"
    pub date: String, // TODO "The GMT date and time of the post, as a string"
    // /// "The post format"
    // (only present on old-style posts)
    // pub format: PostFormat,
    /// "The key used to reblog this post, see the `/post/reblog` method"
    pub reblog_key: String,
    /// "Tags applied to the post"
    pub tags: Vec<String>,
    // TODO "bookmarklet", "mobile" old-style only?
    /// "The URL for the source of the content (for quotes, reblogs, etc.).
    ///  Exists only if there's a content source."
    pub source_url: Option<String>,
    /// "The title of the source site. Exists only if there's a content source."
    // TODO - do source_url and source_title *always* exist when the other does?
    pub source_title: Option<String>,
    /// "Indicates if a user has already liked a post or not.
    ///  Exists only if the request is fully authenticated with OAuth."
    pub liked: bool,
    /// "Indicates the current state of the post"
    pub state: PostState,
    /// "Indicates whether the post is stored in the Neue Post Format"
    pub is_blocks_post_format: bool,

    // // /// "The post type. If formatting as NPF, the type will be blocks; if formatting as legacy, the type will be one of the original legacy types (text, photo, quote, chat, link, video, audio)."
    // // #[serde(rename = "type", with = "postcommon_type_serde_bodge")]
    // // pub post_type: PostType,
    /// (undocumented) the post's original type? only present on npf posts.
    pub original_type: String,
    /// (undocumented?)
    // wait is this one actually not mentioned in the docs anywhere?
    pub blog: Blog,
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


mod postcommon_type_serde_bodge {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::{LegacyPostType, PostType};

    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    #[serde(rename_all = "lowercase")]
    enum PostTypeSerdeShimInner {
        Blocks,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    #[serde(untagged)]
    enum PostTypeSerdeShimOuter {
        NPF(PostTypeSerdeShimInner),
        Legacy(LegacyPostType),
    }

    pub(super) fn _serialize<S>(post_type: &PostType, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match post_type {
            PostType::NPF => PostTypeSerdeShimInner::Blocks.serialize(serializer),
            PostType::Legacy(t) => t.serialize(serializer),
        }
    }

    pub(super) fn _deserialize<'de, D>(deserializer: D) -> Result<PostType, D::Error>
    where
        D: Deserializer<'de>,
    {
        match PostTypeSerdeShimOuter::deserialize(deserializer)? {
            PostTypeSerdeShimOuter::NPF(_) => Ok(PostType::NPF),
            PostTypeSerdeShimOuter::Legacy(t) => Ok(PostType::Legacy(t)),
        }
    }
}
