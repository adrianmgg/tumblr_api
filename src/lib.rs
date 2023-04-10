use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Blog {
    pub uuid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// <https://www.tumblr.com/docs/npf#content-blocks>
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ContentBlock {
    /// <https://www.tumblr.com/docs/npf#content-block-type-text>
    Text {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        subtype: Option<TextSubtype>,
        #[serde(skip_serializing_if = "Option::is_none")]
        indent_level: Option<i32>,
        #[serde(skip_serializing_if = "Vec::is_empty", default)]
        formatting: Vec<InlineFormat>,
    },
    /// <https://www.tumblr.com/docs/npf#content-block-type-image>
    Image {
        /// "An array of [MediaObject]s which represent different available sizes of this image asset."
        media: Vec<MediaObject>,
        /// "Colors used in the image."
        #[serde(skip_serializing_if = "Option::is_none")]
        colors: Option<HashMap<String, String>>,
        /// "A feedback token to use when this image block is a GIF Search result."
        #[serde(skip_serializing_if = "Option::is_none")]
        feedback_token: Option<String>,
        /// "For GIFs, this is a single-frame "poster""
        #[serde(skip_serializing_if = "Option::is_none")]
        poster: Option<String>,
        // TODO doc ("See the Attributions section for details about these objects.")
        #[serde(skip_serializing_if = "Option::is_none")]
        attribution: Option<Attribution>,
        /// "Text used to describe the image, for screen readers. 4096 character maximum."
        // TODO enforce that max length on serialize
        #[serde(skip_serializing_if = "Option::is_none")]
        alt_text: Option<String>,
        /// "A caption typically shown under the image. 4096 character maximum."
        // TODO enforce that max length on serialize
        #[serde(skip_serializing_if = "Option::is_none")]
        caption: Option<String>,
    },
    /// <https://www.tumblr.com/docs/npf#content-block-type-link>
    Link {
        // TODO
    },
    /// <https://www.tumblr.com/docs/npf#content-block-type-audio>
    Audio {
        // TODO
    },
    /// <https://www.tumblr.com/docs/npf#content-block-type-video>
    Video {
        // TODO
    },
    /// <https://www.tumblr.com/docs/npf#content-block-type-paywall>
    Paywall {
        // TODO
    },
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TextSubtype {
    Heading1,
    Heading2,
    Quirky,
    Quote,
    Indented,
    Chat,
    OrderedListItem,
    UnorderedListItem,
}

/// <https://www.tumblr.com/docs/npf#inline-formatting-within-a-text-block>
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum InlineFormat {
    /// <https://www.tumblr.com/docs/npf#inline-format-types-bold-italic-strikethrough-small>
    Bold {
        #[serde(flatten)]
        range: InlineFormatRange,
    },
    /// <https://www.tumblr.com/docs/npf#inline-format-types-bold-italic-strikethrough-small>
    Italic {
        #[serde(flatten)]
        range: InlineFormatRange,
    },
    /// <https://www.tumblr.com/docs/npf#inline-format-types-bold-italic-strikethrough-small>
    Strikethrough {
        #[serde(flatten)]
        range: InlineFormatRange,
    },
    /// <https://www.tumblr.com/docs/npf#inline-format-types-bold-italic-strikethrough-small>
    Small {
        #[serde(flatten)]
        range: InlineFormatRange,
    },
    /// <https://www.tumblr.com/docs/npf#inline-format-type-link>
    Link {
        #[serde(flatten)]
        range: InlineFormatRange,
        url: String,
    },
    /// <https://www.tumblr.com/docs/npf#inline-format-type-mention>
    Mention {
        #[serde(flatten)]
        range: InlineFormatRange,
        blog: Blog,
    },
    /// <https://www.tumblr.com/docs/npf#inline-format-type-color>
    Color {
        #[serde(flatten)]
        range: InlineFormatRange,
        /// "The color to use, in standard hex format, with leading #."
        // TODO - should actually parse these rather than leave them as strings
        hex: String,
    },
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct InlineFormatRange {
    pub start: i32,
    pub end: i32,
}

/// <https://www.tumblr.com/docs/npf#media-objects>
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct MediaObject {
    /// "The canonical URL of the media asset"
    pub url: String,
    /// "The MIME type of the media asset, or a best approximation will be made based on the given URL"
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// "The width of the media asset, if that makes sense (for images and videos, but not for audio)"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i32>,
    /// "The height of the media asset, if that makes sense (for images and videos, but not for audio)"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
    /// "For display purposes, this indicates whether the dimensions are defaults"
    /// > If the original dimensions of the media are not known, a boolean flag [MediaObject.original_dimensions_missing] with a value of true will also be included in the media object. In this scenario, width and height will be assigned default dimensional values of 540 and 405 respectively. Please note that this field should only be available when consuming an NPF Post, it is not allowed during Post creation."
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_dimensions_missing: Option<bool>,
    /// "This indicates whether this media object is a cropped version of the original media"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cropped: Option<bool>,
    /// "This indicates whether this media object has the same dimensions as the original media"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_original_dimensions: Option<bool>,
}

/// <https://www.tumblr.com/docs/npf#attributions>
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Attribution {
    /// <https://www.tumblr.com/docs/npf#attribution-type-post>
    Post {
        /// "The URL of the Post to be attributed."
        url: String,
        /// "A [`Post`] object with at least an [`Post::id`] field."
        post: Post,
        blog: Blog,
    },
    /// <https://www.tumblr.com/docs/npf#attribution-type-link>
    Link {
        /// "The URL to be attributed for the content."
        url: String,
    },
    /// <https://www.tumblr.com/docs/npf#attribution-type-blog>
    Blog {
        blog: Blog,
    },
    /// <https://www.tumblr.com/docs/npf#attribution-type-app>
    App {
        /// "The canonical URL to the source content in the third-party app."
        url: String,
        /// "The name of the application to be attributed."
        #[serde(skip_serializing_if = "Option::is_none")]
        app_name: Option<String>,
        /// "Any display text that the client should use with the attribution."
        #[serde(skip_serializing_if = "Option::is_none")]
        display_text: Option<String>,
        /// "A specific logo [`MediaObject`] that the client should use with the third-party app attribution."
        #[serde(skip_serializing_if = "Option::is_none")]
        logo: Option<MediaObject>,
    },
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Post {
    // TODO
}
