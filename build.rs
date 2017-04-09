use std::env;
use std::path::Path;
// use std::process::Command;

fn main() {
    let project_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let project_dir = Path::new(&project_dir);
    let js_dir_path = project_dir.join("target/libs");
    println!("cargo:rustc-flags=-L {}", js_dir_path.to_str().unwrap());
    //
    // Command::new("npm")
    //     .args(&["install", "--silent"])
    //     .status()
    //     .ok()
    //     .expect(r#"failed to run "npm install""#);
    //
    // Command::new("npm")
    //     .args(&["run", "build", "--silent"])
    //     .status()
    //     .ok()
    //     .expect(r#"failed to run "npm run build""#);
}
