use std::fs::remove_dir_all;
use std::{fs::File, io::{Read, Write}};
use proto_parser::load_site;
use proto_parser::proto_parser::common::domain::Type;
use geosite_converter::v2site_to_sing;

fn main() -> Result<(), std::io::Error> {
    // Open V2Fly geosite
    let mut file = File::open("dlc.dat")?;
    let mut buf = Vec::<u8>::new();
    file.read_to_end(&mut buf)?;

    // Load data
    let geosite_list = load_site(&buf).unwrap();

    if std::path::Path::new("ruleset").is_dir() {
        remove_dir_all("ruleset/").unwrap();
    }
    std::fs::create_dir("ruleset").unwrap();
    // Match domain
    for i in geosite_list.entry {
        if std::path::Path::new(format!("ruleset/{}", i.country_code.to_lowercase()).as_str()).exists() {
            std::fs::remove_file(format!("ruleset/{}", i.country_code.to_lowercase()).as_str()).unwrap();
        }
        
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .append(true)
            .open(format!("ruleset/{}.yaml", i.country_code.to_lowercase()).as_str()).unwrap();
        writeln!(file, "payload:").unwrap();
        for sites in i.domain {
            match sites.r#type.try_into().unwrap() {
                Type::Full => {
                    writeln!(file, "    -\'{}\'", sites.value).unwrap();
                }
                Type::Domain => {
                    writeln!(file, "    -\'+.{}\'", sites.value).unwrap();
                }
                Type::Plain => {
                    writeln!(file, "    -\'keyword:{}\'", sites.value).unwrap();
                }
                Type::Regex => {
                    writeln!(file, "    -\'regexp:{}\'", sites.value).unwrap();
                }
            }
        }
    }

    // Convert geosite.dat to sing-box format
    unsafe { v2site_to_sing(); }
    Ok(())
}
