use std::fs::{remove_dir_all, OpenOptions};
use std::io::BufRead;
use std::{fs::File, io::{Read, Write}};
use proto_parser::proto_parser::common::{GeoSite, Domain};
use proto_parser::{load_site, geosite_serialization};
use proto_parser::proto_parser::common::domain::{Type, Attribute};
use geosite_converter::{v2site_to_sing, generate};
use fancy_regex::Regex;
use std::ffi::CString;

#[allow(non_snake_case)]
fn main() -> Result<(), std::io::Error> {
    let mut v2fly_dlc = File::open("dlc.dat")?;
    let mut v2fly_domain = Vec::<u8>::new();
    let mut country_code = Vec::<String>::new();
    v2fly_dlc.read_to_end(&mut v2fly_domain)?;
    let paths = std::fs::read_dir("resources")?;

    for file in paths {
        country_code.push(file.unwrap().file_name().to_string_lossy().to_string().to_lowercase().replace(".list", ""));
    }

    let mut sites = load_site(&v2fly_domain)?;

    for i in country_code {
        let index = sites.entry.iter().position(|domain|
            domain.country_code.eq(&i.to_uppercase())
        );
        if let Some(idx) = index {
            let _ = sites.entry.remove(idx);
        }
    }

    // Regex
    let domain_regex = Regex::new(r"(?<=DOMAIN,).*").unwrap();
    let domain_suffix_regex = Regex::new(r"(?<=DOMAIN-SUFFIX,).*").unwrap();
    let domain_keyword_regex = Regex::new(r"(?<=DOMAIN-KEYWORD,).*").unwrap();
    let v2fly_regex = Regex::new(r"(?<=URL-REGEX,).*").unwrap();

    let paths = std::fs::read_dir("resources")?;
    for file in paths {
        let path = file.unwrap();
        let file = std::io::BufReader::new(std::fs::File::open(path.path())?);
        let mut domains: GeoSite = GeoSite {
            country_code: path.file_name().to_str().unwrap().to_string().replace(".list", "").to_uppercase(),
            domain: Vec::<Domain>::new()
        };
        for i in file.lines() {
            let str = i.unwrap();
            if let Some(full) = domain_regex.find(str.as_str()).unwrap() {
                let domain_info = Domain {
                    r#type: Type::Full.into(),
                    value: full.as_str().chars().collect(),
                    attribute: Vec::<Attribute>::new()
                };
                domains.domain.append(&mut vec![domain_info]);
            } else if let Some(domain) = domain_suffix_regex.find(str.as_str()).unwrap() {
                let domain_info = Domain {
                    r#type: Type::Domain.into(),
                    value: domain.as_str().chars().collect(),
                    attribute: Vec::<Attribute>::new()
                };
                domains.domain.append(&mut vec![domain_info]);
            } else if let Some(keyword) = domain_keyword_regex.find(str.as_str()).unwrap() {
                let domain_info = Domain {
                    r#type: Type::Plain.into(),
                    value: keyword.as_str().chars().collect(),
                    attribute: Vec::<Attribute>::new()
                };
                domains.domain.append(&mut vec![domain_info]);
            } else if let Some(regex) = v2fly_regex.find(str.as_str()).unwrap() {
                let domain_info = Domain {
                    r#type: Type::Regex.into(),
                    value: regex.as_str().chars().collect(),
                    attribute: Vec::<Attribute>::new()
                };
                domains.domain.append(&mut vec![domain_info]);
            }
        }
        sites.entry.append(&mut vec![domains]);
    }
    
    if std::path::Path::new("ruleset").is_dir() {
        remove_dir_all("ruleset/").unwrap();
    }
    std::fs::create_dir("ruleset").unwrap();

    for i in sites.entry.iter() {
        if std::path::Path::new(format!("ruleset/{}", i.country_code.to_lowercase()).as_str()).exists() {
            std::fs::remove_file(format!("ruleset/{}", i.country_code.to_lowercase()).as_str()).unwrap();
        }
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .append(true)
            .open(format!("ruleset/{}.list", i.country_code.to_lowercase()).as_str()).unwrap();
        for sites in i.domain.iter() {
            match sites.r#type.try_into().unwrap() {
                Type::Full => {
                    writeln!(file, "{}", sites.value).unwrap();
                }
                Type::Domain => {
                    writeln!(file, ".{}", sites.value).unwrap();
                }
                Type::Plain => {
                    if !std::path::Path::new(format!("ruleset/{}-classical.yaml", i.country_code.to_lowercase()).as_str()).exists() {
                        let mut file = std::fs::OpenOptions::new()
                            .create_new(true)
                            .write(true)
                            .append(true)
                            .open(format!("ruleset/{}-classical.yaml", i.country_code.to_lowercase().as_str())).unwrap();
                        writeln!(file, "payload:").unwrap();
                        writeln!(file, "    - DOMAIN-KEYWORD, {}", sites.value).unwrap();
                    } else {
                        let mut file = std::fs::OpenOptions::new()
                            .create_new(false)
                            .write(true)
                            .append(true)
                            .open(format!("ruleset/{}-classical.yaml", i.country_code.to_lowercase().as_str())).unwrap();
                        writeln!(file, "    - DOMAIN-KEYWORD, {}", sites.value).unwrap();
                    }
                }
                Type::Regex => {}
            }
        }
    }

    if std::path::Path::new("geosite.dat").exists() {
        std::fs::remove_file("geosite.dat")?;
    }

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("geosite.dat")?;

    file.write(&geosite_serialization(&sites))?;

    unsafe {v2site_to_sing();}
    let geositeInputFile = CString::new("geosite.dat").unwrap();
    let ruleSetOutputDir = CString::new("sing-box_ruleset").unwrap();
    unsafe {
        generate(geositeInputFile.as_ptr(), ruleSetOutputDir.as_ptr());
    }
    Ok(())
}
