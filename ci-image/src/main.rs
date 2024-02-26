use std::{
    env::VarError,
    process::{self, Command},
};
use tabled::{settings::Style, Table, Tabled};
use tokio;

use pypeep::cli::*;
use pypeep::db::*;

#[derive(Tabled)]
struct PyRequirement {
    id: usize,
    name: String,
    current_version: String,
}

struct Config {
    db_uri: String,
}

impl Config {
    fn from_env() -> Result<Self, VarError> {
        let db_uri = std::env::var("PYPEEP_DB_PATH")?;
        Ok(Self { db_uri })
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let args = make_cli().get_matches();
    let package = args.get_one::<String>("package");
    match package {
        None => {
            tracing::error!("failed to parse argument --package");
            process::exit(1);
        }
        Some(p) => {
            let Ok(config) = Config::from_env() else {
                tracing::error!("failed to load configuration");
                process::exit(2);
            };
            let Ok(pool) = get_db_pool(&config.db_uri).await else {
                tracing::error!("failed to connect to database");
                process::exit(3);
            };
            if install_package(p).is_err() {
                tracing::error!("failed to install package {}", &p);
                process::exit(4);
            }
            let Ok(requirements) = get_requirements() else {
                tracing::error!("failed to parse requirements for package {}", &p);
                process::exit(5);
            };
            if update_projects(p, &pool).await.is_err() {
                tracing::error!("failed to update [projects] {}", &p);
                process::exit(6);
            }
            for requirement in &requirements {
                if update_requirements(&requirement.name, &pool).await.is_err() {
                    tracing::error!(
                        "failed to update [requirements] for {}.{}",
                        &p,
                        &requirement.name
                    );
                    process::exit(6);
                }
                let res = update_project_requirements(
                    p,
                    &requirement.name,
                    &requirement.current_version,
                    &pool,
                )
                .await;
                if res.is_err() {
                    tracing::error!(
                        "failed to update [project_requirements] for {}.{}",
                        &p,
                        &requirement.name
                    );
                    process::exit(6);
                }
            }
            println!();
            println!(
                "Recorded the current version of {} requirements for {} as follows:",
                &requirements.len(),
                &p
            );
            let mut table = Table::new(requirements);
            table.with(Style::psql());
            println!("{table}");
        }
    }
}

fn install_package(package: &String) -> Result<(), std::io::Error> {
    tracing::info!("installing {}", package);
    let _ = Command::new("uv")
        .args(["pip", "install", package])
        .status()?;
    Ok(())
}

fn get_requirements() -> Result<Vec<PyRequirement>, Box<dyn std::error::Error>> {
    let uv_freeze = Command::new("uv").args(["pip", "freeze"]).output()?;
    let mut requirements = Vec::<PyRequirement>::new();
    let installed = String::from_utf8(uv_freeze.stdout)?;
    for (i, requirement) in installed.split("\n").enumerate() {
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
    Ok(requirements)
}
