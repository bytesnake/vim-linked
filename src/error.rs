use std::{io, result};
use crate::parser::NodeId;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    InvalidLink(String, String),
    InvalidHeader(String),
    MissingNote(NodeId),
    Other(String),
}
