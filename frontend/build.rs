extern crate failure;
extern crate sass_rs;
extern crate toml;
extern crate webapp;

use failure::Error;
use sass_rs::{compile_file, Options, OutputStyle};
use std::{
    env,
    fs::{copy, read_to_string, write},
    path::{Path, PathBuf},
    process::Command,
};
use webapp::{config::Config, CONFIG_FILENAME};

const REPOSITORY: &str = "https://github.com/uikit/uikit.git";
const TAG: &str = "v3.0.0-rc.10";
const CSS_FILE: &str = "style.css";
const SCSS_FILE: &str = "style.scss";

pub fn main() -> Result<(), Error> {
    // Prepeare UIKit and build the complete style
    prepare_style()?;

    // Prepare the API URL paths
    prepare_api()?;

    Ok(())
}

fn run<F>(name: &str, mut configure: F) -> Result<(), Error>
where
    F: FnMut(&mut Command) -> &mut Command,
{
    let mut command = Command::new(name);
    let configured = configure(&mut command);
    if !configured.status()?.success() {
        panic!("failed to execute {:?}", configured);
    }
    Ok(())
}

fn prepare_style() -> Result<(), Error> {
    // Prepare the directory
    let out_dir = env::var("OUT_DIR")?;
    let mut target = PathBuf::from(out_dir);
    target.push("uikit");

    // Clone the repo if needed
    if !Path::new(&target.join(".git")).exists() {
        run("git", |command| {
            command
                .arg("clone")
                .arg(format!("--branch={}", TAG))
                .arg("--recursive")
                .arg(REPOSITORY)
                .arg(&target)
        })?;
    }

    // Copy the scss file into the output directory
    target.pop();
    target.push(SCSS_FILE);
    copy(format!("src/{}", SCSS_FILE), &target)?;

    // Build the file
    let mut options = Options::default();
    options.output_style = OutputStyle::Compressed;
    match compile_file(&target, options) {
        Err(error) => panic!(error),
        Ok(content) => {
            // Copy the file into the static directory
            target.pop();
            target.push(CSS_FILE);
            write(&target, content)?;
            copy(&target, format!("static/css/{}", CSS_FILE))?;
        }
    }

    Ok(())
}

fn prepare_api() -> Result<(), Error> {
    let config: Config = toml::from_str(&read_to_string(format!("../{}", CONFIG_FILENAME))?)?;

    let secure_protocol = if config.server.tls { "s" } else { "" };
    let api_url = format!("http{}://{}:{}", secure_protocol, config.server.ip, config.server.port);

    println!("cargo:rustc-env=API_URL={}", api_url);
    println!(
        "cargo:rustc-env=API_URL_LOGIN_CREDENTIALS={}{}",
        api_url, config.api.login_credentials
    );
    println!(
        "cargo:rustc-env=API_URL_LOGIN_SESSION={}{}",
        api_url, config.api.login_session
    );
    println!("cargo:rustc-env=API_URL_LOGOUT={}{}", api_url, config.api.logout);

    Ok(())
}
