macro_rules! json_serde_eq {
    ($type:ty, $json:tt, $thing:expr) => {{
        let thing: $type = $thing;
        let json: serde_json::Value = serde_json::from_str($json).unwrap();
        let serialized = serde_json::to_value(&thing).unwrap();
        assert_eq!(serialized, json);
        let deserialized: $type = serde_json::from_value(json).unwrap();
        assert_eq!(thing, deserialized);
    }};
}

macro_rules! json_de_eq {
    ($type:ty, $json:tt, $thing:expr) => {{
        let thing: $type = $thing;
        let json: serde_json::Value = serde_json::from_str($json).unwrap();
        let deserialized: $type = serde_json::from_value(json).unwrap();
        assert_eq!(thing, deserialized);
    }};
}

use tumblr_api::npf::*;

#[test]
fn content_block_text() {
    json_serde_eq!(
        ContentBlock,
        r#"{"type": "text", "text": "Hello world!"}"#,
        ContentBlock::Text(ContentBlockText::builder().text("Hello world!").build())
    );
    json_serde_eq!(
        ContentBlock,
        r#"{"type":"text", "text":"some bold indented text", "subtype": "indented", "indent_level": 1, "formatting":[{"start":5,"end":9,"type":"bold"}]}"#,
        ContentBlockText::builder()
            .text("some bold indented text")
            .subtype(TextSubtype::Indented)
            .indent_level(1)
            .formatting(vec![InlineFormat {
                start: 5,
                end: 9,
                format: InlineFormatType::Bold
            }])
            .build()
    )
}

#[test]
fn content_block_attribution_empty_list() {
    json_de_eq!(
        ContentBlock,
        r#"{"type": "image", "media": [], "attribution": []}"#,
        ContentBlockImage::builder().media(vec![]).build()
    );
}

#[test]
fn text_subtype() {
    json_serde_eq!(TextSubtype, r#""heading1""#, TextSubtype::Heading1);
    json_serde_eq!(TextSubtype, r#""heading2""#, TextSubtype::Heading2);
    json_serde_eq!(TextSubtype, r#""quirky""#, TextSubtype::Quirky);
    json_serde_eq!(TextSubtype, r#""quote""#, TextSubtype::Quote);
    json_serde_eq!(TextSubtype, r#""indented""#, TextSubtype::Indented);
    json_serde_eq!(TextSubtype, r#""chat""#, TextSubtype::Chat);
    json_serde_eq!(
        TextSubtype,
        r#""ordered-list-item""#,
        TextSubtype::OrderedListItem
    );
    json_serde_eq!(
        TextSubtype,
        r#""unordered-list-item""#,
        TextSubtype::UnorderedListItem
    );
}

#[test]
fn inline_format() {
    json_serde_eq!(
        InlineFormat,
        r#"{"type": "bold", "start": 5, "end": 9}"#,
        InlineFormat {
            start: 5,
            end: 9,
            format: InlineFormatType::Bold
        }
    );
    json_serde_eq!(
        InlineFormat,
        r#"{"type": "italic", "start": 14, "end": 20}"#,
        InlineFormat {
            start: 14,
            end: 20,
            format: InlineFormatType::Italic
        }
    );
    json_serde_eq!(
        InlineFormat,
        r#"{"type": "strikethrough", "start": 0, "end": 1}"#,
        InlineFormat {
            start: 0,
            end: 1,
            format: InlineFormatType::Strikethrough
        }
    );
    json_serde_eq!(
        InlineFormat,
        r#"{"type": "small", "start": 5, "end": 10}"#,
        InlineFormat {
            start: 5,
            end: 10,
            format: InlineFormatType::Small
        }
    );
    json_serde_eq!(
        InlineFormat,
        r#"{"type": "link", "start": 6, "end": 10, "url": "https://www.nasa.gov"}"#,
        InlineFormat {
            start: 6,
            end: 10,
            format: InlineFormatType::Link {
                url: "https://www.nasa.gov".into()
            }
        }
    );
    json_serde_eq!(
        InlineFormat,
        r#"{"start":13,"end":19,"type":"mention","blog":{"uuid":"t:123456abcdf","name":"david","url":"https://davidslog.com/"}}"#,
        InlineFormat {
            start: 13,
            end: 19,
            format: InlineFormatType::Mention {
                blog: MentionBlog {
                    uuid: "t:123456abcdf".to_string(),
                    name: Some("david".to_string()),
                    url: Some("https://davidslog.com/".into()),
                }
            }
        }
    );
    json_serde_eq!(
        InlineFormat,
        r##"{"start":10,"end":15,"type":"color","hex":"#ff492f"}"##,
        InlineFormat {
            start: 10,
            end: 15,
            format: InlineFormatType::Color {
                hex: "#ff492f".into(),
            }
        }
    );
}
