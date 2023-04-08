use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Blog {
    pub uuid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// <https://www.tumblr.com/docs/npf#content-blocks>
#[derive(Serialize, Deserialize)]
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
        media: Vec<MediaObject>,
    }
}

#[derive(Serialize, Deserialize)]
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
#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
pub struct InlineFormatRange {
    pub start: i32,
    pub end: i32,
}

/// <https://www.tumblr.com/docs/npf#media-objects>
#[derive(Serialize, Deserialize)]
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    macro_rules! json_serde_test {
        ($testname:ident, $type:ty, $thing:expr, $json:expr) => {
            #[test]
            fn $testname() {
                let block = $thing;
                let serialized = serde_json::to_value(block).unwrap();
                assert_eq!(serialized, $json,)
            }
        };
    }

    json_serde_test!(
        contentblock_text_simple,
        ContentBlock,
        ContentBlock::Text {
            text: "Hello world!".to_string(),
            subtype: None,
            indent_level: None,
            formatting: vec![]
        },
        json!({"type": "text", "text": "Hello world!"})
    );

    json_serde_test!(
        contentblock_text_indented_no_indent_level,
        ContentBlock,
        ContentBlock::Text {
            text: "Hello world!".to_string(),
            subtype: Some(TextSubtype::Indented),
            indent_level: None,
            formatting: vec![]
        },
        json!({"type": "text", "text": "Hello world!", "subtype": "indented"})
    );

    json_serde_test!(
        contentblock_text_indented_with_indent_level,
        ContentBlock,
        ContentBlock::Text {
            text: "Hello world!".to_string(),
            subtype: Some(TextSubtype::Indented),
            indent_level: Some(1),
            formatting: vec![]
        },
        json!({"type": "text", "text": "Hello world!", "subtype": "indented", "indent_level": 1})
    );

    json_serde_test!(
        contentblock_text_inline_format_bold_and_italic,
        ContentBlock,
        ContentBlock::Text {
            text: "some bold and italic text".to_string(),
            subtype: None,
            indent_level: None,
            formatting: vec![
                InlineFormat::Bold {
                    range: InlineFormatRange { start: 5, end: 9 }
                },
                InlineFormat::Italic {
                    range: InlineFormatRange { start: 14, end: 20 }
                },
            ]
        },
        json!({"type":"text","text":"some bold and italic text","formatting":[{"start":5,"end":9,"type":"bold"},{"start":14,"end":20,"type":"italic"}]})
    );

    json_serde_test!(
        contentblock_text_inline_format_link,
        ContentBlock,
        ContentBlock::Text {
            text: "Found this link for you".to_string(),
            subtype: None,
            indent_level: None,
            formatting: vec![InlineFormat::Link {
                range: InlineFormatRange { start: 6, end: 10 },
                url: "https://www.nasa.gov".to_string()
            }]
        },
        json!({"type":"text","text":"Found this link for you","formatting":[{"start":6,"end":10,"type":"link","url":"https://www.nasa.gov"}]})
    );

    json_serde_test!(
        contentblock_text_inline_format_mention,
        ContentBlock,
        ContentBlock::Text {
            text: "Shout out to @david".to_string(),
            subtype: None,
            indent_level: None,
            formatting: vec![InlineFormat::Mention {
                range: InlineFormatRange { start: 13, end: 19 },
                blog: Blog {
                    uuid: "t:123456abcdf".to_string(),
                    name: Some("david".to_string()),
                    url: Some("https://davidslog.com/".to_string())
                }
            }]
        },
        json!({"type":"text","text":"Shout out to @david","formatting":[{"start":13,"end":19,"type":"mention","blog":{"uuid":"t:123456abcdf","name":"david","url":"https://davidslog.com/"}}]})
    );

    json_serde_test!(
        contentblock_text_inline_format_color,
        ContentBlock,
        ContentBlock::Text {
            text: "Celebrate Pride Month".to_string(),
            subtype: None,
            indent_level: None,
            formatting: vec![InlineFormat::Color {
                range: InlineFormatRange { start: 10, end: 15 },
                hex: "#ff492f".to_string()
            }]
        },
        json!({"type":"text","text":"Celebrate Pride Month","formatting":[{"start":10,"end":15,"type":"color","hex":"#ff492f"}]})
    );
}
