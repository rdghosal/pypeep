use std::process::Command;

use tabled::{settings::Style, Table, Tabled};

#[derive(Tabled)]
struct PyRequirement<'a> {
    id: usize,
    name: &'a str,
    current_version: &'a str,
}

fn main() {
    let output = Command::new("uv").args(["pip", "freeze"]).output();
    let mut requirements = Vec::<PyRequirement>::new();
    if let Ok(o) = output {
        let installed =
            String::from_utf8(o.stdout).expect("Failed to convert stdout from `uv pip freeze`");
        for (i, requirement) in installed.split("\n").enumerate() {
            if requirement.is_empty() {
                continue;
            }
            let mut split = requirement.split("==");
            let (name, current_version) = (split.next().unwrap(), split.next().unwrap());
            requirements.push(PyRequirement {
                id: i + 1,
                name,
                current_version,
            });
        }
        let mut table = Table::new(requirements);
        table.with(Style::psql());
        println!("{table}");
    } else {
        panic!("oops!");
    }
}
