use sqlx::{Pool, Sqlite, SqlitePool};

pub async fn get_db_pool(db_uri: &String) -> Result<Pool<Sqlite>, sqlx::Error> {
    let pool = SqlitePool::connect(db_uri).await?;
    Ok(pool)
}

pub async fn update_projects(project: &String, pool: &SqlitePool) -> Result<(), sqlx::Error> {
    tracing::info!("inserting {} into [projects]", project);
    let _ = sqlx::query("INSERT OR IGNORE INTO projects(name) VALUES (?)")
        .bind(project)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_requirements(
    requirement: &String,
    pool: &SqlitePool,
) -> Result<(), sqlx::Error> {
    tracing::info!("inserting {} into [requirements]", requirement);
    let _ = sqlx::query("INSERT OR IGNORE INTO requirements(name) VALUES (?)")
        .bind(requirement)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_project_requirements(
    project: &String,
    requirement: &String,
    requirement_version: &String,
    pool: &SqlitePool,
) -> Result<(), sqlx::Error> {
    tracing::info!(
        "inserting {}@{} into [projects_requirements]",
        &requirement,
        &requirement_version
    );
    let _ = sqlx::query(
        r#"INSERT INTO project_requirements(project_name, requirement, current_version)
            VALUES (?, ?, ?) ON CONFLICT(project_name, requirement)
            DO UPDATE SET current_version = ?, updated_at = CURRENT_TIMESTAMP"#,
    )
    .bind(project)
    .bind(requirement)
    .bind(requirement_version)
    .bind(requirement_version)
    .execute(pool)
    .await?;
    Ok(())
}
