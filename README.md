A rust implementation of the Tumblr API.

This is still very much in beta! see [Major Planned/Unimplemented Features](#major-plannedunimplemented-features)

## Examples

### Example: Creating a simple post with the client
```rust
use tumblr_api::client::{Client, Credentials};
use tumblr_api::npf;
let client = Client::new(Credentials::new_oauth2(
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

## Major Planned/Unimplemented Features
- refreshing access tokens (currently, the client will just start failing after the token expires)
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
