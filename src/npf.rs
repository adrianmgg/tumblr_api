use std::collections::HashMap;

use serde::{Deserialize, Serialize, Deserializer};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Blog {
    pub uuid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

// TODO give this a better name
fn attribution_deserialize_special_case_stuff<'de, D>(deserializer: D) -> Result<Option<Attribution>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Foo {
        EmptyTuple([i32; 0]),
        SingleAttributionValue(Attribution),
    }
    match Option::<Foo>::deserialize(deserializer)? {
        None | Some(Foo::EmptyTuple(_)) => Ok(None),
        Some(Foo::SingleAttributionValue(v)) => Ok(Some(v)),
    }
}

fn single_or_list_of_one<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum SingleOrListOfOne<T> {
        Single(T),
        ListOfOne([T; 1]),
    }
    match SingleOrListOfOne::<T>::deserialize(deserializer)? {
        SingleOrListOfOne::Single(a) => Ok(a),
        SingleOrListOfOne::ListOfOne([a]) => Ok(a),
    }
}

/// <https://www.tumblr.com/docs/npf#content-blocks>
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "lowercase", deny_unknown_fields)]
pub enum ContentBlock {
    /// <https://www.tumblr.com/docs/npf#content-block-type-text>
    Text {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        subtype: Option<TextSubtype>,
        #[serde(skip_serializing_if = "Option::is_none")]
        indent_level: Option<i32>,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        formatting: Option<Vec<InlineFormat>>,
    },
    /// <https://www.tumblr.com/docs/npf#content-block-type-image>
    Image {
        /// "An array of [MediaObject]s which represent different available sizes of this image asset."
        media: Vec<MediaObject>,
        /// "Colors used in the image."
        /// 
        /// (undocumented) note: colors may instead be listed under [`MediaObject::colors`] in individual entries of [`ContentBlock::Image::media`]
        #[serde(skip_serializing_if = "Option::is_none")]
        colors: Option<HashMap<String, String>>,
        /// "A feedback token to use when this image block is a GIF Search result."
        #[serde(skip_serializing_if = "Option::is_none")]
        feedback_token: Option<String>,
        // (one spot in the docs says that an image's `poster` goes here -- that's wrong afaict, it goes in the media object)
        // TODO doc ("See the Attributions section for details about these objects.")
        // TODO some posts sent with `"attribution": []` ???
        #[serde(skip_serializing_if = "Option::is_none", default, deserialize_with = "attribution_deserialize_special_case_stuff")]
        attribution: Option<Attribution>,
        /// "Text used to describe the image, for screen readers. 4096 character maximum."
        // TODO enforce that max length on serialize
        #[serde(skip_serializing_if = "Option::is_none")]
        alt_text: Option<String>,
        /// "A caption typically shown under the image. 4096 character maximum."
        // TODO enforce that max length on serialize
        #[serde(skip_serializing_if = "Option::is_none")]
        caption: Option<String>,
        /// (undocumented) exif tags associated with the image
        ///
        /// some sample values:
        /// ```json
        /// {"Time": 1590426081, "FocalLength": 3, "FocalLength35mmEquiv": 3, "Aperture": 1.8, "ExposureTime": 0.0011904761904761906, "ISO": 20, "CameraMake": "Apple", "CameraModel": "iPhone 7", "Lens": "3mm"}
        /// {"Time": 1647793571}
        /// {"Time": "1625497381"}
        /// ```
        /// (note how `"Time"` is sometimes a string!)
        #[serde(skip_serializing_if = "Option::is_none")]
        exif: Option<serde_json::Map<String, serde_json::Value>>,
        /// (undocumented)
        #[serde(skip_serializing_if = "Option::is_none")]
        clickthrough: Option<Clickthrough>,
    },
    /// <https://www.tumblr.com/docs/npf#content-block-type-link>
    Link {
        /// "The URL to use for the link block."
        url: String,
        /// "The title of where the link goes."
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        /// "The description of where the link goes."
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        /// "The author of the link's content."
        #[serde(skip_serializing_if = "Option::is_none")]
        author: Option<String>,
        /// "The name of the site being linked to."
        #[serde(skip_serializing_if = "Option::is_none")]
        site_name: Option<String>,
        /// "Supplied on NPF Post consumption, ignored during NPF Post creation."
        #[serde(skip_serializing_if = "Option::is_none")]
        display_url: Option<String>,
        /// "Supplied on NPF Post consumption, ignored during NPF Post creation."
        #[serde(skip_serializing_if = "Option::is_none")]
        poster: Option<Vec<MediaObject>>,
    },
    /// <https://www.tumblr.com/docs/npf#content-block-type-audio>
    Audio {
        // TODO - "either the media field or url field must be present" -- should the types of this represent the either/or-ness of that? (also applies to ::Video)
        /// "The URL to use for the audio block, if no media is present."
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        /// "The Media Object to use for the audio block, if no url is present."
        #[serde(skip_serializing_if = "Option::is_none")]
        media: Option<MediaObject>,
        // TODO should maybe have this as an enum with an 'other' variant
        /// "The provider of the audio source, whether it's tumblr for native audio or a trusted third party."
        #[serde(skip_serializing_if = "Option::is_none")]
        provider: Option<String>,
        /// "The title of the audio asset."
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        /// "The artist of the audio asset."
        #[serde(skip_serializing_if = "Option::is_none")]
        artist: Option<String>,
        /// "The album from which the audio asset originated."
        #[serde(skip_serializing_if = "Option::is_none")]
        album: Option<String>,
        /// "An image media object to use as a "poster" for the audio track, usually album art."
        #[serde(skip_serializing_if = "Option::is_none", default, deserialize_with = "single_or_list_of_one")]
        poster: Option<MediaObject>,
        /// "HTML code that could be used to embed this audio track into a webpage."
        #[serde(skip_serializing_if = "Option::is_none")]
        embed_html: Option<String>,
        /// "A URL to the embeddable content to use as an iframe."
        #[serde(skip_serializing_if = "Option::is_none")]
        embed_url: Option<String>,
        /// "Optional provider-specific metadata about the audio track."
        // TODO is Value the right thing to use here?
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<serde_json::Value>,
        /// "Optional attribution information about where the audio track came from."
        #[serde(skip_serializing_if = "Option::is_none", default, deserialize_with = "attribution_deserialize_special_case_stuff")]
        attribution: Option<Attribution>,
    },
    /// <https://www.tumblr.com/docs/npf#content-block-type-video>
    Video {
        /// "The URL to use for the video block, if no media is present."
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        /// "The Media Object to use for the video block, if no url is present."
        #[serde(skip_serializing_if = "Option::is_none")]
        media: Option<MediaObject>,
        /// "The provider of the video, whether it's tumblr for native video or a trusted third party."
        #[serde(skip_serializing_if = "Option::is_none")]
        provider: Option<String>,
        /// "HTML code that could be used to embed this video into a webpage."
        #[serde(skip_serializing_if = "Option::is_none")]
        embed_html: Option<String>,
        /// "An embed iframe object used for constructing video iframes."
        #[serde(skip_serializing_if = "Option::is_none")]
        embed_iframe: Option<EmbedIframe>,
        /// "A URL to the embeddable content to use as an iframe."
        #[serde(skip_serializing_if = "Option::is_none")]
        embed_url: Option<String>,
        /// "An image media object to use as a "poster" for the video, usually a single frame."
        // (table in the docs say this is a single MediaObject, but it's actually a list of them)
        #[serde(skip_serializing_if = "Option::is_none")]
        poster: Option<Vec<MediaObject>>,
        /// "Optional provider-specific metadata about the video."
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<serde_json::Value>,
        /// "Optional attribution information about where the video came from."
        #[serde(skip_serializing_if = "Option::is_none", default, deserialize_with = "attribution_deserialize_special_case_stuff")]
        attribution: Option<Attribution>,
        /// "Whether this video can be played on a cellular connection."
        #[serde(skip_serializing_if = "Option::is_none")]
        can_autoplay_on_cellular: Option<bool>,
        // kinda undocumented, it's in one of the examples in the docs but they never explain it
        // TODO sometimes just a single value rather than a list
        #[serde(skip_serializing_if = "Option::is_none")]
        filmstrip: Option<MediaObject>,
    },
    /// <https://www.tumblr.com/docs/npf#content-block-type-paywall>
    Paywall {
        // TODO
    },
    /// (undocumented)
    // TODO - some of these fields should probably be optiona
    Poll {
        client_id: String,
        question: String,
        answers: Vec<PollAnswer>,
        settings: PollSettings,
        // TODO - timestamp string, should probably parse it
        created_at: String,
        timestamp: i64,
    },
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
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
#[serde(tag = "type", rename_all = "lowercase", deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct InlineFormatRange {
    pub start: i32,
    pub end: i32,
}

/// <https://www.tumblr.com/docs/npf#media-objects>
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
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
    /// (undocumented)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_key: Option<String>,
    /// (undocumented) see [`ContentBlock::Image::colors`]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub colors: Option<HashMap<String, String>>,
    /// <https://www.tumblr.com/docs/npf#gif-posters>
    #[serde(skip_serializing_if = "Option::is_none")]
    poster: Option<Box<MediaObject>>,
    /// (undocumented) video alternative for animated gif image
    #[serde(skip_serializing_if = "Option::is_none")]
    video: Option<Vec<MediaObject>>,
}

/// <https://www.tumblr.com/docs/npf#attributions>
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "lowercase", deny_unknown_fields)]
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
        /// (undocumented) the href.li version of the url
        url_redirect: Option<String>,
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
#[serde(deny_unknown_fields)]
pub struct Post {
    pub id: String,
}

/// <https://www.tumblr.com/docs/npf#embed-iframe-objects>
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EmbedIframe {
    /// "A URL used for constructing and embeddable video iframe"
    url: String,
    /// "The width of the video iframe"
    #[serde(skip_serializing_if = "Option::is_none")]
    width: Option<i32>,
    /// "The height of the video iframe"
    /// #[serde(skip_serializing_if = "Option::is_none")]
    height: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PollAnswer {
    answer_text: String,
    client_id: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PollSettings {
    multiple_choice: bool,
    // TODO should probably be an enum - what are the possible values?
    close_status: String,
    expire_after: serde_json::Number,
    source: String,
}

/// (undocumented)
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Clickthrough {
    pub web_url: String,
    // TODO - not sure what type this one is
    pub deeplink_url: Option<()>,
}
