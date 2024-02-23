use std::process::Command;

fn main() {
    let output = Command::new("uv").args(["pip", "freeze"]).output();
    let mut package_names = Vec::<&str>::new();
    if let Ok(o) = output {
        let requirements =
            String::from_utf8(o.stdout).expect("Failed to convert stdout from `uv pip freeze`");
        for requirement in requirements.split("\n") {
            let package_name = requirement.split("==").nth(0).unwrap();
            if package_name.is_empty() {
                continue;
            }
            package_names.push(package_name);
        }
        dbg!(package_names);
    } else {
        panic!("oops!");
    }
}
