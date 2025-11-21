use super::{DomainType, GnerateDomainSet};
use serde::Serialize;
use std::ffi::{CString, c_char, c_int, c_uchar};

#[link(name = "converter")]
unsafe extern "C" {
    fn singboxCompileRuleset(
        dat: *const c_char,
        len: c_int,
        version: c_uchar,
        output_path: *const c_char,
        output_path_len: c_int,
    ) -> *mut c_char;
}

#[derive(Serialize)]
struct Rule {
    domain: Option<Vec<String>>,
    domain_suffix: Option<Vec<String>>,
    domain_keyword: Option<Vec<String>>,
    domain_regex: Option<Vec<String>>,
}

#[derive(Serialize)]
pub struct Singbox {
    version: u8,
    rules: Vec<Rule>,
}

#[derive(Debug)]
pub struct Error(String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "error occurred while generating sing-box ruleset: {}",
            self.0
        )
    }
}

impl std::error::Error for Error {}

impl GnerateDomainSet for Singbox {
    #[inline]
    fn push_domain(&mut self, dat: String) {
        if self.rules[0].domain.is_none() {
            self.rules[0].domain = Some(Vec::<String>::with_capacity(100_000));
        }
        let Some(ref mut domain) = self.rules[0].domain else {
            panic!();
        };
        domain.push(dat);
    }

    #[inline]
    fn push_domain_suffix(&mut self, dat: String) {
        if self.rules[0].domain_suffix.is_none() {
            self.rules[0].domain_suffix = Some(Vec::<String>::with_capacity(100_000));
        }
        let Some(ref mut domain_suffix) = self.rules[0].domain_suffix else {
            panic!();
        };
        domain_suffix.push(dat);
    }

    #[inline]
    fn push_domain_keyworkd(&mut self, dat: String) {
        if self.rules[0].domain_keyword.is_none() {
            self.rules[0].domain_keyword = Some(Vec::<String>::with_capacity(100));
        }
        let Some(ref mut domain_keyword) = self.rules[0].domain_keyword else {
            panic!();
        };
        domain_keyword.push(dat);
    }

    #[inline]
    fn push_domain_regex(&mut self, dat: String) {
        if self.rules[0].domain_regex.is_none() {
            self.rules[0].domain_regex = Some(Vec::<String>::with_capacity(100));
        }
        let Some(ref mut domain_regex) = self.rules[0].domain_regex else {
            panic!();
        };
        domain_regex.push(dat);
    }
}

impl Singbox {
    pub fn new() -> Self {
        Self {
            version: 3,
            rules: vec![Rule {
                domain: None,
                domain_suffix: None,
                domain_keyword: None,
                domain_regex: None,
            }],
        }
    }

    #[inline]
    pub fn compile<'a>(&self, output_path: &'a str) -> Result<(), Error> {
        let data = serde_json::to_string(self).unwrap();

        unsafe {
            let err = singboxCompileRuleset(
                data.as_ptr() as *const c_char,
                data.len() as i32,
                3,
                output_path.as_ptr() as *const c_char,
                output_path.len() as i32,
            );
            if !err.is_null() {
                let dat = CString::from_raw(err).into_string().unwrap();
                return Err(Error(dat));
            }
        }
        Ok(())
    }

    pub fn push(&mut self, r#type: DomainType, dat: String) {
        match r#type {
            DomainType::Domain => self.push_domain(dat),
            DomainType::DomainKeyword => self.push_domain_keyworkd(dat),
            DomainType::DomainRegex => self.push_domain_regex(dat),
            DomainType::DomainSuffix => self.push_domain_suffix(dat),
        }
    }
}
