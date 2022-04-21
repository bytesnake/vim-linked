use std::result;
use crate::parser::NodeId;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    InvalidLink(usize, String, String),
    InvalidHeader(usize, String),
    MissingNote(NodeId),
    Other(String),
}
