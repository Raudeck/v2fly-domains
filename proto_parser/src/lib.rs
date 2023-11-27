
#[allow(dead_code)]
use std::io::Cursor;
use prost::Message;
use proto_parser::common::{self, GeoSiteList};

pub mod proto_parser {
    pub mod common {
        include!(concat!(env!("OUT_DIR"), "/proto_parser.common.rs"));
    }
}

pub fn create() -> GeoSiteList {
    let buf = GeoSiteList::default();
    buf
}

pub fn geosite_serialization(sites: &GeoSiteList) -> Vec<u8> {
    let mut buf = Vec::<u8>::new();
    buf.reserve(sites.encoded_len());
    sites.encode(&mut buf).unwrap();
    buf
}

pub fn load_site(buf: &Vec<u8>) -> Result<common::GeoSiteList, prost::DecodeError> {
    common::GeoSiteList::decode(&mut Cursor::new(buf))
}