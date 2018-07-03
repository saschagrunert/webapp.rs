extern crate capnpc;
extern crate failure;
extern crate sass_rs;
extern crate toml;

#[macro_use]
extern crate serde_derive;

use capnpc::CompilerCommand;
use failure::Error;
use sass_rs::{compile_file, Options, OutputStyle};
use std::{
    env,
    fs::{copy, read_to_string, write},
    path::{Path, PathBuf},
    process::Command,
};

const REPOSITORY: &str = "https://github.com/uikit/uikit.git";
const TAG: &str = "v3.0.0-rc.6";
const CSS_FILE: &str = "style.css";
const SCSS_FILE: &str = "style.scss";
const CAPNP_FILE: &str = "protocol.capnp";

pub fn main() -> Result<(), Error> {
    // Prepeare UIKit and build the complete style
    prepare_style()?;

    // Compile capnp protocol definition
    prepare_capnp()?;

    // Prepare the global project configuration
    prepare_config()?;

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
    copy(format!("src/frontend/{}", SCSS_FILE), &target)?;

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

fn prepare_capnp() -> Result<(), Error> {
    CompilerCommand::new()
        .file(PathBuf::from("src").join(CAPNP_FILE))
        .run()?;

    Ok(())
}

#[derive(Deserialize)]
struct Config {
    server: Server,
}

#[derive(Deserialize)]
struct Server {
    ip: String,
    port: String,
    tls: bool,
}

fn prepare_config() -> Result<(), Error> {
    let config: Config = toml::from_str(&read_to_string("Config.toml")?)?;

    // Set the websocket path directly within the build target
    let ws_prot = if config.server.tls { "wss" } else { "ws" };
    println!(
        "cargo:rustc-env=WS_URL={}://{}:{}/ws",
        ws_prot, config.server.ip, config.server.port
    );

    Ok(())
}
