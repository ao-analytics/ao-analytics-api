use config::Config;

#[derive(Debug, Clone, Config)]
pub struct Config {
    pub port: i32,
    pub database_url: String,
}
