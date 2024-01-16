//! A rust implementation of the Tumblr API.
//!
//! This is still very much in beta! see [Major Planned/Unimplemented Features](#major-plannedunimplemented-features)
//!
//! # Examples
//!
//! ## Creating a simple post with the client
//!```no_run
//! # #[tokio::main]
//! # async fn main() -> anyhow::Result<()> {
//! use tumblr_api::client::{Client, Credentials};
//! use tumblr_api::npf;
//! let client = Client::new(Credentials::new_oauth2(
//!     "your consumer key",
//!     "your consumer secret",
//! ));
//! client
//!     .create_post(
//!         "blog-name",
//!         vec![npf::ContentBlockText::builder("hello world").build()],
//!     )
//!     .send()
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Creating a more complex post
//! ```no_run
//! # #[tokio::main]
//! # async fn main() -> anyhow::Result<()> {
//! # use tumblr_api::client::{Client, Credentials};
//! # use tumblr_api::npf;
//! use tumblr_api::client::CreatePostState;
//! # let client = Client::new(Credentials::new_oauth2(
//! #     "your consumer key",
//! #     "your consumer secret",
//! # ));
//! // load the image that we'll be attaching to the post.
//! let my_image = std::fs::read("path/to/my_image.jpg")?;
//! // (currently, you need to manually create the reqwest::Body to pass in. that'll probably
//! //  change in a future version.)
//! let my_image = reqwest::Body::from(my_image);
//! client
//!     .create_post(
//!         "blog-name",
//!         vec![
//!             npf::ContentBlockText::builder("hello world").build(),
//!             npf::ContentBlockImage::builder(vec![npf::MediaObject::builder(
//!                 npf::MediaObjectContent::Identifier("my-image-identifier".into()),
//!             )
//!             .build()])
//!             .build(),
//!             npf::ContentBlockText::builder("some bold text in a heading")
//!                 .subtype(npf::TextSubtype::Heading1)
//!                 .formatting(vec![npf::InlineFormat {
//!                     start: 5,
//!                     end: 9,
//!                     format: npf::InlineFormatType::Bold,
//!                 }])
//!                 .build(),
//!         ],
//!     )
//!     .add_attachment(my_image, "image/jpeg", "my-image-identifier")
//!     // add tags to your post
//!     // (this is currently a string since that's what the underlying api takes.
//!     //  Being able to pass a Vec<String> instead is a planned feature but hasn't been
//!     //  implemented quite yet.)
//!     .tags("tag_1,tag_2,tag_3")
//!     // add the post to your queue instead of immediately posting it
//!     .initial_state(CreatePostState::Queue)
//!     .send()
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Modules & Feature Flags
//! This library is split into 3 modules - `client`, `api`, and `npf` - and each has a feature flag of the same name that controls whether it's enabled.
//!
//! # Major Planned/Unimplemented Features
//! - implement remaining api endpoints (currently it's just post creation plus a couple others)

// clippy::pedantic
#![warn(clippy::pedantic)]
#![allow(clippy::struct_excessive_bools)]
// clippy::restriction
#![warn(
    // clippy::alloc_instead_of_core,
    // clippy::std_instead_of_alloc,
    // clippy::std_instead_of_core,
    // clippy::exhaustive_structs,
    // clippy::exhaustive_enums,
    clippy::multiple_inherent_impl,
    clippy::partial_pub_fields,
    // clippy::unneeded_field_pattern,
    clippy::panic_in_result_fn,
    clippy::same_name_method,
    clippy::dbg_macro,
    clippy::todo,
    clippy::unimplemented,
    clippy::print_stderr,
    clippy::print_stdout,
    clippy::try_err,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::use_debug,
    clippy::question_mark,
    clippy::needless_question_mark,
)]
// clippy::cargo
#![warn(clippy::cargo)]
#![allow(clippy::cargo_common_metadata)]
#![allow(clippy::multiple_crate_versions)]

#[cfg(feature = "api")]
pub mod api;
#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "npf")]
pub mod npf;
#[cfg(feature = "auth")]
pub mod auth;
