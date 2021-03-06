//! This crate is a pure Rust MongoDB driver. It follows the
//! [MongoDB driver API and feature specifications](https://github.com/mongodb/specifications).
//!
//! To connect to a MongoDB database, pass a MongoDB connection string to
//! [`Client::with_uri_str`](struct.Client.html#method.with_uri_str):
//!
//! ```rust
//! # use mongors::{Client, error::Result};
//! #
//! # fn make_client() -> Result<Client> {
//! let client = Client::with_uri_str("mongodb://localhost:27017/")?;
//! # Ok(client)
//! # }
//! ```
//! Alternately, create an instance of [`ClientOptions`](options/struct.ClientOptions.html) and pass
//! it to [`Client::with_options`](struct.Client.html#method.with_options):
//!
//! ```rust
//! # use mongors::{
//! #     error::Result,
//! #     options::{StreamAddress, ClientOptions},
//! #     Client,
//! # };
//! #
//! # fn make_client() -> Result<Client> {
//! let options = ClientOptions::builder()
//!                   .hosts(vec![
//!                       StreamAddress {
//!                           hostname: "localhost".into(),
//!                           port: Some(27017),
//!                       }
//!                   ])
//!                   .build();
//!
//! let client = Client::with_options(options)?;
//! # Ok(client)
//! # }
//! ```
//!
//! Operations can be performed by obtaining a [`Database`](struct.Database.html) or
//! [`Collection`](struct.Collection.html) from the [`Client`](struct.Client.html):
//!
//! ```rust
//! # use mongors::{Client, error::Result};
//! # use mongors::doc;
//! #
//! # fn do_stuff() -> Result<()> {
//! # let client = Client::with_uri_str("mongodb://localhost:27017")?;
//! #
//! let db = client.database("some_db");
//! for coll_name in db.list_collection_names(None)? {
//!     println!("collection: {}", coll_name);
//! }
//!
//! let coll = db.collection("some-coll");
//! let result = coll.insert_one(doc! { "x": 1 }, None)?;
//! println!("{:#?}", result);
//!
//! # Ok(())
//! # }
//! ```

#![allow(unused_variables)]
#![cfg_attr(
    feature = "cargo-clippy",
    allow(
        clippy::unreadable_literal,
        clippy::cognitive_complexity,
        clippy::float_cmp
    )
)]

#[macro_use]
pub mod options;
// #[macro_use]
pub mod bson;
pub mod util;
mod bson_util;
mod client;
mod cmap;
mod coll;
mod collation;
mod concern;
mod cursor;
mod db;
pub mod error;
pub mod event;
mod is_master;
mod operation;
pub mod results;
mod sdam;
mod selection_criteria;
mod srv;
#[cfg(test)]
mod test;

#[cfg(test)]
#[macro_use]
extern crate derive_more;

pub use self::bson::{Bson, Document};

pub use crate::{
    client::Client,
    coll::{Collection, Namespace},
    cursor::Cursor,
    db::Database,
};
