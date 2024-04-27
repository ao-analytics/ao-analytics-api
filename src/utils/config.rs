use tracing::error;

pub struct Config {
    pub port: i32,

    pub db_url: String,
}

impl Config {
    pub fn from_env() -> Option<Self> {
        let db_url = get_var_from_env_or_dotenv("DATABASE_URL")?;
        let port = get_var_from_env_or_dotenv("PORT")?.parse::<i32>().ok()?;

        Some(Config { db_url, port })
    }
}

fn get_var_from_env_or_dotenv(name: &str) -> Option<String> {
    let var = std::env::var(name).or(dotenv::var(name));

    match var {
        Ok(var) => Some(var),
        Err(_) => {
            error!("{} is not set", name);
            None
        }
    }
}
