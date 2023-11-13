
#[allow(dead_code)]
use std::io::Cursor;
use prost::Message;

pub mod proto_parser {
    pub mod common {
        include!(concat!(env!("OUT_DIR"), "/proto_parser.common.rs"));
    }
}

use proto_parser::common;

pub fn load_site(buf: &Vec<u8>) -> Result<common::GeoSiteList, prost::DecodeError> {
    common::GeoSiteList::decode(&mut Cursor::new(buf))
}