//! Based off of [zvariant::as_value](https://docs.rs/zvariant/5.5.1/zvariant/as_value/index.html)
//!
//! If the target value doesn't have the same signature as the requested signature, try to parse
//! the target value with the requested signature.
//!
//! This is useful for when you are deserializing something is similar in type, but changes to some
//! other similar type somehow. For example, dict entry "mpris:length" has value type `int64` for
//! Firefox, but `uint64` for Spotify, so we try parsing the `uint64` into `int64` instead.

mod deserialize;
pub use deserialize::{Deserialize, deserialize};

pub mod try_as_optional {
    use super::*;

    pub use deserialize::deserialize_optional as deserialize;
}
