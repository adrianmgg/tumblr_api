use std::collections::HashMap;

use serde::{Deserialize, Serialize, Deserializer};
use typed_builder::TypedBuilder;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MentionBlog {
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
    Text(ContentBlockText),
    Image(ContentBlockImage),
    Link(ContentBlockLink),
    Audio(ContentBlockAudio),
    Video(ContentBlockVideo),
    Paywall(ContentBlockPaywall),
    Poll(ContentBlockPoll),
}

impl From<ContentBlockText> for ContentBlock {
    fn from(val: ContentBlockText) -> Self {
        ContentBlock::Text(val)
    }
}
impl From<ContentBlockImage> for ContentBlock {
    fn from(val: ContentBlockImage) -> Self {
        ContentBlock::Image(val)
    }
}
impl From<ContentBlockLink> for ContentBlock {
    fn from(val: ContentBlockLink) -> Self {
        ContentBlock::Link(val)
    }
}
impl From<ContentBlockAudio> for ContentBlock {
    fn from(val: ContentBlockAudio) -> Self {
        ContentBlock::Audio(val)
    }
}
impl From<ContentBlockVideo> for ContentBlock {
    fn from(val: ContentBlockVideo) -> Self {
        ContentBlock::Video(val)
    }
}
impl From<ContentBlockPaywall> for ContentBlock {
    fn from(val: ContentBlockPaywall) -> Self {
        ContentBlock::Paywall(val)
    }
}
impl From<ContentBlockPoll> for ContentBlock {
    fn from(val: ContentBlockPoll) -> Self {
        ContentBlock::Poll(val)
    }
}

/// <https://www.tumblr.com/docs/npf#content-block-type-text>
#[derive(Serialize, Deserialize, TypedBuilder, Debug, PartialEq, Eq)]
#[builder(doc, build_method(into))]
pub struct ContentBlockText {
    #[builder(setter(into))]
    pub text: String,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtype: Option<TextSubtype>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indent_level: Option<i32>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub formatting: Option<Vec<InlineFormat>>,
}

/// <https://www.tumblr.com/docs/npf#content-block-type-image>
#[derive(Serialize, Deserialize, TypedBuilder, Debug, PartialEq, Eq)]
#[builder(doc, build_method(into))]
pub struct ContentBlockImage {
    /// "An array of [MediaObject]s which represent different available sizes of this image asset."
    pub media: Vec<MediaObject>,
    /// "Colors used in the image."
    /// 
    /// (undocumented) note: colors may instead be listed under [`MediaObject::colors`] in individual entries of [`ContentBlockImage::media`]
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub colors: Option<HashMap<String, String>>,
    /// "A feedback token to use when this image block is a GIF Search result."
    #[builder(default, setter(into, strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback_token: Option<String>,
    // (one spot in the docs says that an image's `poster` goes here -- that's wrong afaict, it goes in the media object)
    // TODO doc ("See the Attributions section for details about these objects.")
    // TODO some posts sent with `"attribution": []` ???
    #[builder(default, setter(into, strip_option))]
    #[serde(skip_serializing_if = "Option::is_none", default, deserialize_with = "attribution_deserialize_special_case_stuff")]
    pub attribution: Option<Attribution>,
    /// "Text used to describe the image, for screen readers. 4096 character maximum."
    // TODO enforce that max length on serialize
    #[builder(default, setter(into, strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt_text: Option<String>,
    /// "A caption typically shown under the image. 4096 character maximum."
    // TODO enforce that max length on serialize
    #[builder(default, setter(into, strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    /// (undocumented) exif tags associated with the image
    ///
    /// some sample values:
    /// ```json
    /// {"Time": 1590426081, "FocalLength": 3, "FocalLength35mmEquiv": 3, "Aperture": 1.8, "ExposureTime": 0.0011904761904761906, "ISO": 20, "CameraMake": "Apple", "CameraModel": "iPhone 7", "Lens": "3mm"}
    /// {"Time": 1647793571}
    /// {"Time": "1625497381"}
    /// ```
    /// (note how `"Time"` is sometimes a string!)
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exif: Option<serde_json::Map<String, serde_json::Value>>,
    /// (undocumented)
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clickthrough: Option<Clickthrough>,
}

/// <https://www.tumblr.com/docs/npf#content-block-type-link>
#[derive(Serialize, Deserialize, TypedBuilder, Debug, PartialEq, Eq)]
#[builder(doc, build_method(into))]
pub struct ContentBlockLink {
    /// "The URL to use for the link block."
    pub url: String,
    /// "The title of where the link goes."
    #[builder(default, setter(into, strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// "The description of where the link goes."
    #[builder(default, setter(into, strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// "The author of the link's content."
    #[builder(default, setter(into, strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// "The name of the site being linked to."
    #[builder(default, setter(into, strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site_name: Option<String>,
    /// "Supplied on NPF Post consumption, ignored during NPF Post creation."
    #[builder(default, setter(into, strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_url: Option<String>,
    /// "Supplied on NPF Post consumption, ignored during NPF Post creation."
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poster: Option<Vec<MediaObject>>,
}

/// <https://www.tumblr.com/docs/npf#content-block-type-audio>
#[derive(Serialize, Deserialize, TypedBuilder, Debug, PartialEq, Eq)]
#[builder(doc, build_method(into))]
pub struct ContentBlockAudio {
    // TODO - "either the media field or url field must be present" -- should the types of this represent the either/or-ness of that? (also applies to ::Video)
    /// "The URL to use for the audio block, if no media is present."
    #[builder(default, setter(into, strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// "The Media Object to use for the audio block, if no url is present."
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media: Option<MediaObject>,
    // TODO should maybe have this as an enum with an 'other' variant
    /// "The provider of the audio source, whether it's tumblr for native audio or a trusted third party."
    #[builder(default, setter(into, strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// "The title of the audio asset."
    #[builder(default, setter(into, strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// "The artist of the audio asset."
    #[builder(default, setter(into, strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist: Option<String>,
    /// "The album from which the audio asset originated."
    #[builder(default, setter(into, strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album: Option<String>,
    /// "An image media object to use as a "poster" for the audio track, usually album art."
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none", default, deserialize_with = "single_or_list_of_one")]
    pub poster: Option<MediaObject>,
    /// "HTML code that could be used to embed this audio track into a webpage."
    #[builder(default, setter(into, strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embed_html: Option<String>,
    /// "A URL to the embeddable content to use as an iframe."
    #[builder(default, setter(into, strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embed_url: Option<String>,
    /// "Optional provider-specific metadata about the audio track."
    // TODO is Value the right thing to use here?
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    /// "Optional attribution information about where the audio track came from."
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none", default, deserialize_with = "attribution_deserialize_special_case_stuff")]
    pub attribution: Option<Attribution>,
}

/// <https://www.tumblr.com/docs/npf#content-block-type-video>
#[derive(Serialize, Deserialize, TypedBuilder, Debug, PartialEq, Eq)]
#[builder(doc, build_method(into))]
pub struct ContentBlockVideo {
    /// "The URL to use for the video block, if no media is present."
    #[builder(default, setter(into, strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// "The Media Object to use for the video block, if no url is present."
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media: Option<MediaObject>,
    /// "The provider of the video, whether it's tumblr for native video or a trusted third party."
    #[builder(default, setter(into, strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// "HTML code that could be used to embed this video into a webpage."
    #[builder(default, setter(into, strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embed_html: Option<String>,
    /// "An embed iframe object used for constructing video iframes."
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embed_iframe: Option<EmbedIframe>,
    /// "A URL to the embeddable content to use as an iframe."
    #[builder(default, setter(into, strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embed_url: Option<String>,
    /// "An image media object to use as a "poster" for the video, usually a single frame."
    // (table in the docs say this is a single MediaObject, but it's actually a list of them)
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poster: Option<Vec<MediaObject>>,
    /// "Optional provider-specific metadata about the video."
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    /// "Optional attribution information about where the video came from."
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none", default, deserialize_with = "attribution_deserialize_special_case_stuff")]
    pub attribution: Option<Attribution>,
    /// "Whether this video can be played on a cellular connection."
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_autoplay_on_cellular: Option<bool>,
    // kinda undocumented, it's in one of the examples in the docs but they never explain it
    // TODO sometimes just a single value rather than a list
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filmstrip: Option<MediaObject>,
}

/// <https://www.tumblr.com/docs/npf#content-block-type-paywall>
#[derive(Serialize, Deserialize, TypedBuilder, Debug, PartialEq, Eq)]
#[builder(doc, build_method(into))]
pub struct ContentBlockPaywall {
    // TODO
}

/// (undocumented)
// TODO - some of these fields should probably be optiona
#[derive(Serialize, Deserialize, TypedBuilder, Debug, PartialEq, Eq)]
#[builder(doc, build_method(into))]
pub struct ContentBlockPoll {
    pub client_id: String,
    pub question: String,
    pub answers: Vec<PollAnswer>,
    pub settings: PollSettings,
    // TODO - timestamp string, should probably parse it
    pub created_at: String,
    pub timestamp: i64,
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
pub struct InlineFormat {
    pub start: i32,
    pub end: i32,
    #[serde(flatten)]
    pub format: InlineFormatType,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "lowercase", deny_unknown_fields)]
pub enum InlineFormatType {
    /// <https://www.tumblr.com/docs/npf#inline-format-types-bold-italic-strikethrough-small>
    Bold,
    /// <https://www.tumblr.com/docs/npf#inline-format-types-bold-italic-strikethrough-small>
    Italic,
    /// <https://www.tumblr.com/docs/npf#inline-format-types-bold-italic-strikethrough-small>
    Strikethrough,
    /// <https://www.tumblr.com/docs/npf#inline-format-types-bold-italic-strikethrough-small>
    Small,
    /// <https://www.tumblr.com/docs/npf#inline-format-type-link>
    Link {
        url: String,
    },
    /// <https://www.tumblr.com/docs/npf#inline-format-type-mention>
    Mention {
        blog: MentionBlog,
    },
    /// <https://www.tumblr.com/docs/npf#inline-format-type-color>
    Color {
        /// "The color to use, in standard hex format, with leading #."
        // TODO - should actually parse these rather than leave them as strings
        hex: String,
    },
}

/// <https://www.tumblr.com/docs/npf#media-objects>
#[derive(Serialize, Deserialize, TypedBuilder, Debug, PartialEq, Eq)]
#[builder(doc)]
#[serde(deny_unknown_fields)]
pub struct MediaObject {
    /// "The canonical URL of the media asset"
    #[builder(setter(into))]
    pub url: String,
    /// "The MIME type of the media asset, or a best approximation will be made based on the given URL"
    #[builder(default, setter(into, strip_option))]
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// "The width of the media asset, if that makes sense (for images and videos, but not for audio)"
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i32>,
    /// "The height of the media asset, if that makes sense (for images and videos, but not for audio)"
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
    /// "For display purposes, this indicates whether the dimensions are defaults"
    /// > If the original dimensions of the media are not known, a boolean flag [MediaObject.original_dimensions_missing] with a value of true will also be included in the media object. In this scenario, width and height will be assigned default dimensional values of 540 and 405 respectively. Please note that this field should only be available when consuming an NPF Post, it is not allowed during Post creation."
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_dimensions_missing: Option<bool>,
    /// "This indicates whether this media object is a cropped version of the original media"
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cropped: Option<bool>,
    /// "This indicates whether this media object has the same dimensions as the original media"
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_original_dimensions: Option<bool>,
    /// (undocumented)
    #[builder(default, setter(into, strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_key: Option<String>,
    /// (undocumented) see [`ContentBlockImage::colors`]
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub colors: Option<HashMap<String, String>>,
    /// <https://www.tumblr.com/docs/npf#gif-posters>
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poster: Option<Box<MediaObject>>,
    /// (undocumented) video alternative for animated gif image
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video: Option<Vec<MediaObject>>,
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
        blog: MentionBlog,
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
        blog: MentionBlog,
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
