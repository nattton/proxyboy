use crate::{establish_connection, models::InsertMock, schema::mocks};
use diesel::prelude::*;
use std::fs;

#[derive(serde::Deserialize)]
struct Config {
    router_list: Vec<RouterConfig>,
}

#[derive(serde::Deserialize)]
struct RouterConfig {
    #[serde(default = "default_name")]
    name: String,
    #[serde(default = "default_enable")]
    enable: bool,
    method: String,
    url: String,
    file: String,
    #[serde(default = "default_status_code")]
    status_code: i32,
    #[serde(default = "default_delay")]
    delay: i32,
    #[serde(default = "default_content_type")]
    content_type: String,
}

fn default_name() -> String {
    "".to_string()
}

fn default_enable() -> bool {
    true
}

fn default_status_code() -> i32 {
    200
}

fn default_delay() -> i32 {
    0
}

fn default_content_type() -> String {
    "application/json".to_string()
}

pub fn import_config() -> Result<(), Box<dyn std::error::Error>> {
    let config_content = fs::read_to_string("config.json")?;
    let config: Config = serde_json::from_str(&config_content)?;

    let mut conn = establish_connection();

    // Clear existing mocks
    diesel::delete(mocks::table).execute(&mut conn)?;
    println!("Cleared existing mocks");

    for router in config.router_list {
        // Handle multiple methods (comma-separated)
        let methods: Vec<&str> = router.method.split(',').map(|s| s.trim()).collect();

        for method in methods {
            let insert_mock = InsertMock {
                name: router.name.clone(),
                is_enable: router.enable,
                request_method: method.to_uppercase(),
                request_url: router.url.clone(),
                response_file_path: router.file.clone(),
                response_status_code: router.status_code,
                response_delay: router.delay,
                response_content_type: router.content_type.clone(),
            };

            diesel::insert_into(mocks::table)
                .values(&insert_mock)
                .execute(&mut conn)?;

            println!(
                "Imported mock: {} {} -> {}",
                method.to_uppercase(),
                router.url,
                router.file
            );
        }
    }

    println!("Successfully imported router configurations");
    Ok(())
}
