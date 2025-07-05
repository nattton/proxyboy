pub mod models;
pub mod schema;

use diesel::prelude::*;
use dotenvy::dotenv;
use std::env;

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL");
    match database_url {
        Ok(database_url) => SqliteConnection::establish(&database_url)
            .unwrap_or_else(|_| panic!("Error connecting to {}", database_url)),
        Err(_) => {
            let path = env::current_dir().expect("cannot get current dir");
            let database_url = format!("{}/database.db", path.display());
            SqliteConnection::establish(&database_url)
                .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
        }
    }
}
