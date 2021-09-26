use clap::{Arg, App};

#[derive(Debug, PartialEq)]
pub struct Config {
    pub level_file: String,
}

impl Config {
    pub fn new() -> Self {
        let matches = App::new("cg_mars_lander_ga")
            .version("0.1.0")
            .author("Simon Galasso <simon.galasso@hotmail.fr>")
            .about("Find best mars lander trajectory")
            .arg(Arg::with_name("file")
                .required(true)
                .help("level file"))
            .get_matches();
        return Self {
            level_file: matches.value_of("file").unwrap_or("").to_string(),
        }
    }
}