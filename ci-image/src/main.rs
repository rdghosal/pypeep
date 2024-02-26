use clap;
use sqlx::{Pool, Sqlite, SqlitePool};
use std::{
    env::VarError,
    io::{self, Error},
    process::{self, Command},
};
use tabled::{settings::Style, Table, Tabled};
use tokio;

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
        Some(p) => {
            let config = match Config::from_env() {
                Ok(c) => c,
                _ => {
                    tracing::error!("failed to load configuration");
                    process::exit(1);
                }
            };
            let pool = match get_db_pool(&config.db_uri).await {
                Ok(p) => p,
                _ => {
                    tracing::error!("failed to load configuration");
                    process::exit(2);
                }
            };
            install_package(p);
            let requirements = get_requirements();
            update_projects(p, &pool).await;
            for requirement in &requirements {
                update_requirements(&requirement.name, &pool).await;
                update_project_requirements(
                    p,
                    &requirement.name,
                    &requirement.current_version,
                    &pool,
                )
                .await;
            }
            println!(
                "Recorded the current version of {} requirements for {} as follows:",
                &requirements.len(),
                &p
            );
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

async fn get_db_pool(db_uri: &String) -> Result<Pool<Sqlite>, sqlx::Error> {
    let pool = SqlitePool::connect(db_uri).await?;
    Ok(pool)
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

async fn update_projects(project: &String, pool: &SqlitePool) {
    tracing::info!("inserting {} into [projects]", project);
    let _ = sqlx::query("INSERT OR IGNORE INTO projects(name) VALUES (?)")
        .bind(project)
        .execute(pool)
        .await
        .unwrap();
}

async fn update_requirements(requirement: &String, pool: &SqlitePool) {
    tracing::info!("inserting {} into [requirements]", requirement);
    let _ = sqlx::query("INSERT OR IGNORE INTO requirements(name) VALUES (?)")
        .bind(requirement)
        .execute(pool)
        .await
        .unwrap();
}

async fn update_project_requirements(
    project: &String,
    requirement: &String,
    requirement_version: &String,
    pool: &SqlitePool,
) {
    tracing::info!(
        "inserting {}@{} into [projects_requirements]",
        &requirement,
        &requirement_version
    );
    let _ = sqlx::query(
        "INSERT INTO project_requirements(project_name, requirement, current_version) \
            VALUES (?, ?, ?) ON CONFLICT(project_name, requirement) \
            DO UPDATE SET current_version = ?, updated_at = CURRENT_TIMESTAMP",
    )
    .bind(project)
    .bind(requirement)
    .bind(requirement_version)
    .bind(requirement_version)
    .execute(pool)
    .await
    .unwrap();
}
