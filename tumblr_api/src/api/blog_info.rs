//! "Retrieve Blog Info" method - [`/v2/blog/<blog identifier>/info`](https://www.tumblr.com/docs/en/api/v2#info---retrieve-blog-info)

use serde::{Serialize, Deserialize};

use crate::npf;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct BlogInfoResponse {
    /// "The display title of the blog"
    pub title: String,
    /// "The total number of posts to this blog"
    pub posts: i64,
    /// "The short blog name that appears before tumblr.com in a standard blog hostname"
    pub name: String,
    // TODO time type correctly
    /// "The time of the most recent post, in seconds since the epoch"
    pub updated: i64,
    /// "You guessed it! The blog's description"
    pub description: String,
    /// "Indicates whether the blog allows questions"
    pub ask: bool,
    // TODO "returned only if ask is true"
    /// "Indicates whether the blog allows anonymous questions"
    pub ask_anon: bool,
    // TODO
    /// "Whether you're following the blog, returned only if this request has an authenticated user"
    pub followed: bool,
    /// "Number of likes for this user, returned only if this is the user's primary blog and sharing of likes is enabled"
    pub likes: Option<i64>,
    // TODO
    /// "Indicates whether this blog has been blocked by the calling user's primary blog; returned only if there is an authenticated user making this call"
    pub is_blocked_from_primary: bool,
    // TODO
    // /// "An array of avatar objects, each a different size, which should each have a width, height, and URL."
    // avatar: Vec<_>,
    /// "The blog's canonical URL"
    pub url: String,
    /// "The blog's general theme options, which may not be useful if the blog uses a custom theme."
    pub theme: BlogTheme,
    // TODO this one definitely needs unknown fields
}

// TODO finish copying over the doc descriptions for these
// TODO use a better type than String for the colors
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct BlogTheme {
    /// "the shape of the mask over the user's avatar"
    pub avatar_shape: AvatarShape,
    pub background_color: String,
    pub body_font: String,
    // TODO
    pub header_bounds: serde_json::Value,
    pub header_image: String,
    pub header_image_npf: npf::ContentBlockImage,
    pub header_image_focused: String,
    pub header_image_poster: String,
    pub header_image_scaled: String,
    pub header_stretch: bool,
    pub link_color: String,
    pub show_avatar: bool,
    pub show_description: bool,
    pub show_header_image: bool,
    pub show_title: bool,
    pub title_color: String,
    pub title_font: String,
    pub title_font_weight: String,
    // TODO probably add unknown fields here
}

// TODO should we add an unknown/other case?
// TODO this probably goes with shared stuff
/// "the shape of the mask over the user's avatar"
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AvatarShape {
    Circle,
    Square,
}
