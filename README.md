[![Crates.io](https://img.shields.io/crates/v/tumblr_api)](https://crates.io/crates/tumblr_api)


# tumblr_api

A rust implementation of the Tumblr API.

This is still very much in beta! see [Major Planned/Unimplemented Features](#major-plannedunimplemented-features)

## Examples

### Creating a simple post with the client
```rust
use tumblr_api::{npf, client::Client, auth::Credentials};
let client = Client::new(Credentials::new(
    "your consumer key",
    "your consumer secret",
));
client
    .create_post(
        "blog-name",
        vec![npf::ContentBlockText::builder("hello world").build()],
    )
    .send()
    .await?;
```

### Creating a more complex post
```rust
use tumblr_api::client::CreatePostState;
// load the image that we'll be attaching to the post.
let my_image = std::fs::read("path/to/my_image.jpg")?;
// (currently, you need to manually create the reqwest::Body to pass in. that'll probably
//  change in a future version.)
let my_image = reqwest::Body::from(my_image);
client
    .create_post(
        "blog-name",
        vec![
            npf::ContentBlockText::builder("hello world").build(),
            npf::ContentBlockImage::builder(vec![npf::MediaObject::builder(
                npf::MediaObjectContent::Identifier("my-image-identifier".into()),
            )
            .build()])
            .build(),
            npf::ContentBlockText::builder("some bold text in a heading")
                .subtype(npf::TextSubtype::Heading1)
                .formatting(vec![npf::InlineFormat {
                    start: 5,
                    end: 9,
                    format: npf::InlineFormatType::Bold,
                }])
                .build(),
        ],
    )
    .add_attachment(my_image, "image/jpeg", "my-image-identifier")
    // add tags to your post
    // (this is currently a string since that's what the underlying api takes.
    //  Being able to pass a Vec<String> instead is a planned feature but hasn't been
    //  implemented quite yet.)
    .tags("tag_1,tag_2,tag_3")
    // add the post to your queue instead of immediately posting it
    .initial_state(CreatePostState::Queue)
    .send()
    .await?;
```

## Modules & Feature Flags
This library is split into 3 modules - `client`, `api`, and `npf` - and each has a feature flag of the same name that controls whether it's enabled.

## Major Planned/Unimplemented Features
- implement remaining api endpoints (currently it's just post creation plus a couple others)

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

<!-- to generate README: cargo readme --project-root ./tumblr_api/ --template ../README.tpl --output ../README.md -->
