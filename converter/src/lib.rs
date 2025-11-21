#![allow(unused)]
pub mod mihomo;
pub mod singbox;

pub enum DomainType {
    Domain,
    DomainSuffix,
    DomainKeyword,
    DomainRegex,
}

pub trait GnerateDomainSet {
    fn push_domain(&mut self, dat: String);
    fn push_domain_suffix(&mut self, dat: String);
    fn push_domain_keyworkd(&mut self, dat: String);
    fn push_domain_regex(&mut self, dat: String);
}

#[cfg(test)]
mod tests {
    use std::io::{BufRead, BufReader, Read};

    use super::*;

    #[test]
    fn compile() {
        let mut sing_box = Singbox::new();
        sing_box.push(DomainType::Domain, "example.com".to_string());
        let res = sing_box.compile("example.srs");
        assert!(res.is_ok());
        let file = std::fs::OpenOptions::new()
            .read(true)
            .open("example.srs")
            .unwrap();
        let mut reader = BufReader::new(file);
        let mut magic_bytes = [0u8; 3];
        reader.read(&mut magic_bytes);
        assert_eq!(magic_bytes, [83, 82, 83]);
        std::fs::remove_file("example.srs").unwrap();
    }

    #[test]
    fn push() {
        let mut sing_box = Singbox::new();
        sing_box.push(DomainType::Domain, "example.com".to_string());
        assert!(sing_box.rules[0].domain.iter().len() == 1)
    }
}
