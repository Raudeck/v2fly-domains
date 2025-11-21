use clap::{crate_version, Arg, Command};
use converter::mihomo::Mihomo;
use converter::singbox::Singbox;
use fancy_regex::Regex;
use log::info;
use proto_parser::proto_parser::common::domain::{Attribute, Type};
use proto_parser::proto_parser::common::{Domain, GeoSite};
use proto_parser::{geosite_serialization, load_site};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::fs::{remove_dir_all, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter};
use std::str::FromStr;
use std::{
    fs::File,
    io::{Read, Write},
};

#[allow(non_snake_case)]
fn main() -> Result<(), std::io::Error> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "INFO");
    }
    env_logger::init();

    let matches = Command::new("Ruleset Generator")
        .version(crate_version!())
        .author("Raudeck (github.com/Raudeck)")
        .arg(
            Arg::new("file")
                .short('f')
                .long("file")
                .required(true)
                .value_name("PATH")
                .help("Set the file path where clash format geosite.dat is located."),
        )
        .arg(
            Arg::new("text")
                .short('t')
                .long("text")
                .required(false)
                .value_name("DIR PATH")
                .help("Specify the ruleset format text folder for extra rules."),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .required(true)
                .value_name("DIR PATH")
                .help("Set the output directory."),
        )
        .get_matches();
    let mut geosite = File::open(matches.get_one::<String>("file").unwrap())?;
    let out_dir =
        std::path::PathBuf::from_str(matches.get_one::<String>("output").unwrap()).unwrap();
    let mut content = Vec::<u8>::new();
    geosite.read_to_end(&mut content)?;
    let mut sites = load_site(&content)?;

    let paths = if let Some(dir) = matches.get_one::<String>("text") {
        Some(std::fs::read_dir(dir)?)
    } else {
        None
    };

    // Clash format text files handler
    let type_matcher = Regex::new(".+(?=,)").unwrap();
    let text_matcher = Regex::new("(?<=,)(.*)").unwrap();
    if let Some(mut paths) = paths {
        while let Some(Ok(file)) = paths.next() {
            info!("Extract domain from {}.", file.path().to_str().unwrap());
            let country_code = file
                .file_name()
                .to_string_lossy()
                .to_string()
                .to_lowercase()
                .replace(".list", "");

            // remove ruleset if a similar entry exists
            let idx = sites
                .entry
                .iter()
                .position(|site| site.country_code.eq(&country_code.to_ascii_uppercase()));
            if let Some(idx) = idx {
                sites.entry.remove(idx);
            }

            // handle geosite from provided text files
            let mut domains = GeoSite {
                country_code: country_code.to_ascii_uppercase(),
                domain: Vec::<Domain>::new(),
            };
            let mut lines = BufReader::new(File::open(file.path())?).lines();
            while let Some(Ok(i)) = lines.next() {
                let Ok(Some(r#type)) = type_matcher.find(&i) else {
                    continue;
                };
                let Ok(Some(domain)) = text_matcher.find(&i) else {
                    continue;
                };
                match r#type.as_str() {
                    "DOMAIN" => {
                        domains.domain.push(Domain {
                            r#type: Type::Full.into(),
                            value: domain.as_str().to_string(),
                            attribute: Vec::<Attribute>::new(),
                        });
                    }
                    "DOMAIN-SUFFIX" => domains.domain.push(Domain {
                        r#type: Type::Domain.into(),
                        value: domain.as_str().to_string(),
                        attribute: Vec::<Attribute>::new(),
                    }),
                    "DOMAIN-KEYWORD" => domains.domain.push(Domain {
                        r#type: Type::Plain.into(),
                        value: domain.as_str().to_string(),
                        attribute: Vec::<Attribute>::new(),
                    }),
                    "DOMAIN-REGEX" => domains.domain.push(Domain {
                        r#type: Type::Plain.into(),
                        value: domain.as_str().to_string(),
                        attribute: Vec::<Attribute>::new(),
                    }),
                    _ => {}
                }
            }
            sites.entry.push(domains);
        }
    }

    // Remove files in the output directory
    [
        None,
        Some("classical"),
        Some("domain"),
        Some("singbox"),
        Some("mrs"),
    ]
    .iter()
    .for_each(|dir| {
        let mut out_dir = out_dir.clone();
        if let Some(dir) = dir {
            out_dir = out_dir.join(dir).clone();
        }
        if out_dir.is_dir() {
            remove_dir_all(&out_dir).unwrap();
        }
        std::fs::create_dir(out_dir).unwrap();
    });
    let classical_rules_dir = out_dir.join("classical");
    let domain_rules_dir = out_dir.join("domain");
    let singbox_rules_dir = out_dir.join("singbox");
    let mrs_rules_dir = out_dir.join("mrs");

    sites.entry.par_iter().for_each(|i| {
        info!("Handle {}", i.country_code.to_lowercase());
        let file_name = format!("{}.list", i.country_code.to_lowercase());
        let singbox_name = format!("{}.srs", i.country_code.to_lowercase());
        let mihomo_name = format!("{}.mrs", i.country_code.to_lowercase());
        let mut classical_file = BufWriter::new(
            OpenOptions::new()
                .create(true)
                .write(true)
                .read(true)
                .open(&classical_rules_dir.join(file_name.as_str()))
                .unwrap(),
        );
        let mut domain_file = BufWriter::new(
            OpenOptions::new()
                .create(true)
                .write(true)
                .open(&domain_rules_dir.join(file_name.as_str()))
                .unwrap(),
        );
        let mut singbox = Singbox::new();
        let mihomo = Mihomo::new();

        for sites in i.domain.iter() {
            let mut enable_domain = true;
            let mut enable_classical = true;
            let prefix: &str;
            match sites.r#type.try_into().unwrap() {
                Type::Full => {
                    prefix = "DOMAIN,";
                    singbox.push(converter::DomainType::Domain, sites.value.clone());
                }
                Type::Domain => {
                    prefix = "DOMAIN-SUFFIX,";
                    singbox.push(converter::DomainType::DomainSuffix, sites.value.clone());
                }
                Type::Plain => {
                    prefix = "DOMAIN-KEYWORD,";
                    enable_domain = false;
                    singbox.push(converter::DomainType::DomainKeyword, sites.value.clone());
                }
                Type::Regex => {
                    prefix = "DOMAIN-REGEX,";
                    enable_domain = false;
                    enable_classical = false;
                    singbox.push(converter::DomainType::DomainRegex, sites.value.clone());
                }
            }
            if enable_classical {
                writeln!(classical_file, "{}{}", prefix, sites.value).unwrap();
            }
            if enable_domain == true && prefix.eq("DOMAIN,") {
                writeln!(domain_file, "{}", sites.value).unwrap();
            }
            if enable_domain == true && prefix.eq("DOMAIN-SUFFIX,") {
                writeln!(domain_file, "+.{}", sites.value).unwrap();
            }
        }
        singbox
            .compile(singbox_rules_dir.join(&singbox_name).to_str().unwrap())
            .unwrap();
        classical_file.flush().unwrap();
        domain_file.flush().unwrap();
        let mut buf = Vec::new();
        BufReader::new(
            OpenOptions::new()
                .read(true)
                .open(&domain_rules_dir.join(&file_name))
                .unwrap(),
        )
        .read_to_end(&mut buf)
        .unwrap();
        mihomo
            .compile(&buf[..], mrs_rules_dir.join(&mihomo_name).to_str().unwrap())
            .unwrap();
    });
    if std::path::Path::new("geosite.dat").exists() {
        std::fs::remove_file("geosite.dat")?;
    }

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("geosite.dat")?;

    file.write(&geosite_serialization(&sites))?;
    Ok(())
}
