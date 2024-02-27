"""Creates tables in database."""
import sqlite3


DB_PATH = "pypeep.db"

DROP_PROJECTS = "DROP TABLE IF EXISTS projects;"
CREATE_PROJECTS = """
    CREATE TABLE projects (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT UNIQUE NOT NULL,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP
        );
    """
DROP_REQUIREMENTS = "DROP TABLE IF EXISTS requirements;"
CREATE_REQUIREMENTS = """
    CREATE TABLE requirements (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT UNIQUE NOT NULL,
        current_version VARCHAR(10),
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP
        );
    """
DROP_PROJECT_REQUIREMENTS = "DROP TABLE IF EXISTS project_requirements;"
CREATE_PROJECT_REQUIREMENTS = """
    CREATE TABLE project_requirements (
        project_name TEXT NOT NULL,
        requirement TEXT NOT NULL,
        current_version VARCHAR(10) NOT NULL,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP,
        PRIMARY KEY (project_name, requirement),
        FOREIGN KEY(project_name) REFERENCES projects(name),
        FOREIGN KEY(requirement) REFERENCES requirements(name)
        );
    """
SCRIPTS = (
    DROP_PROJECTS,
    CREATE_PROJECTS,
    DROP_REQUIREMENTS,
    CREATE_REQUIREMENTS,
    DROP_PROJECT_REQUIREMENTS,
    CREATE_PROJECT_REQUIREMENTS,
)


con = sqlite3.connect(DB_PATH)
cur = con.cursor()

for i, script in enumerate(SCRIPTS):
    print(f"running script {i + 1} of {len(SCRIPTS)}")
    cur.execute(script)
    con.commit()

con.commit()
cur.close()
con.close()
