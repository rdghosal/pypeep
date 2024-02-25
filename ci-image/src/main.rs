use clap;
// use futures::TryStreamExt;
use sqlx::{Row, SqliteConnection, SqlitePool};
use std::process::{self, Command};
use tokio;

use tabled::{settings::Style, Table, Tabled};

#[derive(Tabled)]
struct PyRequirement {
    id: usize,
    name: String,
    current_version: String,
}

struct Config {
    db_uri: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let args = make_cli().get_matches();
    let package = args.get_one::<String>("package");
    match package {
        Some(p) => {
            install_package(&p);
            let config = get_config();
            let requirements = get_requirements();
            let pool = SqlitePool::connect(&config.db_uri).await;
            if pool.is_err() {
                process::exit(1);
            }
            // let rows = sqlx::query("SELECT * FROM requirements")
            //     .fetch_all(&pool.unwrap())
            //     .await;
            // if let Ok(ref r) = rows {
            //     for rr in r {
            //         let v: &str = rr.try_get("name").unwrap();
            //         println!("{v}");
            //     }
            //     // map the row into a user-defined domain type
            // }
            let mut table = Table::new(requirements);
            table.with(Style::psql());
            println!("{table}");
        }
        None => {
            tracing::error!("failed to parse argument --package");
            process::exit(1);
        }
    }
}

fn get_config() -> Config {
    match std::env::var("PYPEEP_DB_PATH") {
        Ok(db_uri) => {
            tracing::info!("setting config");
            Config { db_uri }
        }
        _ => {
            tracing::error!("failed to retrieve configuration");
            process::exit(1);
        }
    }
}

fn make_cli() -> clap::Command {
    tracing::debug!("initializing command");
    clap::Command::new(env!("CARGO_CRATE_NAME"))
        .about("Downloads a specified Python package and stores information about the current state of its dependencies.")
        .arg(
            clap::Arg::new("package")
                .long("package")
                .help("The Python package whose requirements (i.e., dependencies) we are checking.")
                .required(true),
        )
}

fn install_package(package: &String) {
    tracing::info!("installing {}", package);
    let status = Command::new("uv")
        .args(["pip", "install", package])
        .status();
    if let Err(_) = status {
        tracing::error!("failed to install package {package}");
        process::exit(2);
    }
}

fn get_requirements() -> Vec<PyRequirement> {
    let uv_freeze = Command::new("uv").args(["pip", "freeze"]).output();
    let mut requirements = Vec::<PyRequirement>::new();
    if let Ok(o) = uv_freeze {
        let installed = String::from_utf8(o.stdout);
        if let Err(_) = installed {
            tracing::error!("failed to convert stdout from `uv pip freeze`");
            process::exit(4);
        }
        for (i, requirement) in installed.unwrap().split("\n").enumerate() {
            if requirement.is_empty() {
                continue;
            }
            let mut split = requirement.split("==");
            let (name, current_version) = (split.next().unwrap(), split.next().unwrap());
            requirements.push(PyRequirement {
                id: i + 1,
                name: name.to_string(),
                current_version: current_version.to_string(),
            });
        }
    } else {
        tracing::error!("failed to parse package requirements");
        process::exit(3);
    }
    requirements
}
