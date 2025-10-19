use std::env::var;

use dotenvy::dotenv;

pub struct Config {
    pub port: u16,
    pub scheme: String,
    pub host: String,
}

impl Config {
    pub fn try_parse() -> Result<Config, &'static str> {
        let _ = dotenv();

        Ok(Config {
            port: var("PORT")
                .map_err(|_| "An error occured while getting PORT env param")?
                .parse::<u16>()
                .map_err(|_| "An error occured while parsing PORT env param")?,
            scheme: var("SCHEME").map_err(|_| "An error occured while getting SCHEME env param")?,
            host: var("HOST").map_err(|_| "An error occured while getting HOST env param")?,
        })
    }
}
