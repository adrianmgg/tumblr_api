use std::fmt;

use serde::{Deserialize, Deserializer, de::{Visitor, self}, Serialize};

// // https://www.tumblr.com/docs/en/api/v2#postspost-id---fetching-a-post-neue-post-format
#[derive(Debug, PartialEq, Eq)]
// // #[serde(deny_unknown_fields)]
pub struct NPFPost {
    /// The short name used to uniquely identify a blog
    pub blog_name: String,
    /// The post's unique ID
    pub id: i64,
//     /// "The post's unique "genesis" ID as a String. Only available to the post owner in certain circumstances."
//     /// (longer explanation [here](https://www.tumblr.com/docs/en/api/v2#posts--retrieve-published-posts), in the footnote at the bottom of the "Response" section)
//     pub genesis_post_id: Option<String>,
//     /// "The location of the post"
//     pub post_url: String,
//     /// "The type of post"
//     /// 
//     /// **currently not actually checked -- since we're only supporting NPF so far anyways this should only ever be "blocks"**
//     #[serde(rename = "type")]
//     pub post_type: String,
//     pub timestamp: i64, // TODO "The time of the post, in seconds since the epoch"
//     pub date: String,   // TODO "The GMT date and time of the post, as a string"
//     // /// "The post format"
//     // (only present on old-style posts)
//     // pub format: PostFormat,
//     /// "The key used to reblog this post, see the `/post/reblog` method"
//     pub reblog_key: String,
//     /// "Tags applied to the post"
//     pub tags: Vec<String>,
//     // TODO "bookmarklet", "mobile" old-style only?
//     /// information about the source of the content.
//     /// "Exists only if there's a content source."
//     #[serde(flatten)]
//     pub source: Option<SourceInfo>,
//     /// "Indicates if a user has already liked a post or not.
//     ///  Exists only if the request is fully authenticated with OAuth."
//     pub liked: bool,
//     /// "Indicates the current state of the post"
//     pub state: PostState,
//     /// "Indicates whether the post is stored in the Neue Post Format"
//     pub is_blocks_post_format: bool,
//     /// (undocumented) the post's original type? only present on npf posts.
//     pub original_type: String,
//     /// (undocumented?)
//     // wait is this one actually not mentioned in the docs anywhere?
//     pub blog: Blog,
//     #[serde(flatten)]
//     pub blaze_info: BlazeInfo,
//     /// "Short text summary to the end of the post URL"
//     pub slug: String,
//     /// "Short text summary to the end of the post URL"
//     pub short_url: String,
//     pub summary: String,
//     pub should_open_in_legacy: bool,
//     // TODO type?
//     pub recommended_source: serde_json::Value,
//     // TODO type?
//     pub recommended_color: serde_json::Value,
//     pub followed: bool,
//     // TODO - should this be nullable? (check what a no-notes post gives)
//     pub note_count: i32,
//     pub content: Vec<super::npf::ContentBlock>,
//     // TODO
//     pub layout: Vec<serde_json::Value>,
//     // TODO
//     pub trail: Vec<serde_json::Value>,
//     #[serde(flatten)]
//     interactability: InteractabilityInfo,
//     pub display_avatar: bool,
//     // TODO specifically when does this one show up? most posts didnt have it
//     pub is_pinned: Option<bool>,
//     #[serde(flatten)]
//     pub ask_info: Option<AskInfo>,
//     #[serde(flatten, with = "post_submission_info_serde")]
//     pub submission_info: Option<SubmissionInfo>,
//     /// fields not captured by anything else
//     #[serde(flatten)]
//     pub other_fields: serde_json::Map<String, serde_json::Value>,
}

impl<'de> Deserialize<'de> for NPFPost {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        #[derive(serde_enum_str::Deserialize_enum_str)]
        #[serde(rename_all = "snake_case")]
        enum Field {
            Id, IdString,
            BlogName,
            #[serde(other)]
            Unknown(String),
        }

        struct NPFPostVisitor;
        impl<'de> Visitor<'de> for NPFPostVisitor {
            type Value = NPFPost;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> fmt::Result {
                formatter.write_str("a tumblr post")  // TODO could phrase this better
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut id = None;
                // (we just track whether we were given this value since we don't actually use it)
                let mut got_id_string = false;
                let mut blog_name = None;

                macro_rules! duplicate_if_some {
                    ($var:ident, $field_name:expr) => {
                        if $var.is_some() { return Err(de::Error::duplicate_field($field_name)); }
                    };
                    ($var:ident) => { duplicate_if_some!($var, stringify!($var)); }
                }

                macro_rules! simple_field {
                    ($var:ident, $field_name:expr) => {
                        {
                            duplicate_if_some!($var, $field_name);
                            $var = Some(map.next_value()?);
                        }
                    };
                    ($var:ident) => { { simple_field!($var, stringify!($var)); } };
                }

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Id => simple_field!(id),
                        Field::IdString => {
                            if got_id_string {
                                return Err(de::Error::duplicate_field("id_string"));
                            }
                            map.next_value::<String>()?;
                            got_id_string = true;
                        }
                        Field::BlogName => simple_field!(blog_name),
                        Field::Unknown(_) => {},
                    }
                }
                let id = id.ok_or_else(|| de::Error::missing_field("id"))?;
                let blog_name = blog_name.ok_or_else(|| de::Error::missing_field("blog_name"))?;
                Ok(NPFPost { blog_name, id })
            }
        }

        const FIELDS: &[&str] = &["id", "id_string", "blog_name"];
        deserializer.deserialize_struct("Post", FIELDS, NPFPostVisitor)
    }
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
    pub asking_avatar: Vec<crate::npf::MediaObject>,
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
                post_author: post_author.to_owned(),
                post_author_is_adult: post_author_is_adult.to_owned(),
                anonymous_name: anonymous_name.to_owned(),
                anonymous_email: anonymous_email.to_owned(),
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
