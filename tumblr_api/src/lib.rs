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
//! # Modules & Feature Flags
//! This library is split into 3 modules - `client`, `api`, and `npf` - and each has a feature flag of the same name that controls whether it's enabled.
//! 
//! # Major Planned/Unimplemented Features
//! - refreshing access tokens (currently, the client will just start failing after the token expires)
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
