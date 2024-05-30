use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::Path;

fn main() {
    // Get build output dir
    let out_dir = env::var_os("OUT_DIR").unwrap();

    // Codegen
    default_config_codegen(&out_dir);

    // Rebuild if build.rs changes
    println!("cargo::rerun-if-changed=build.rs");
}

/// Generates the function default_config in /config/default.rs
fn default_config_codegen(out_dir: &OsString) {
    let dest_dir = Path::new(out_dir).join("config/");

    // Get default config as String
    let default_config_path = Path::new("./config.toml");
    let default_config = fs::read_to_string(default_config_path)
        .expect("config.toml should exist")
        .replace('\"', "\\\"");

    // Create output dir for codegen
    if !dest_dir.exists() {
        fs::create_dir(&dest_dir).expect("should be able to create config dir");
    }

    // Write default_config function to file
    fs::write(
        dest_dir.join("default.rs"),
        format!(
            "pub fn default_config() -> String {{
            String::from(\"{default_config}\")
        }}
        "
        ),
    )
    .expect("should be able to write to file");

    // Rebuild if config.toml changes
    println!("cargo::rerun-if-changed=config.toml");
}
