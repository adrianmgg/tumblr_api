//! A (currently incomplete) rust implementation of the Tumblr API.
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

#[cfg(feature = "api")]
pub mod api;
#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "npf")]
pub mod npf;
