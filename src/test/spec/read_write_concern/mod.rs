mod connection_string;
mod document;

use crate::bson::{self, Bson, Document};
use crate::{error::Result, options::WriteConcern};

fn write_concern_to_document(write_concern: &WriteConcern) -> Result<Document> {
    match bson::to_bson(&write_concern)? {
        Bson::Document(doc) => Ok(doc),
        _ => unreachable!(),
    }
}
