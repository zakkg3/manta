use std::{fs, io::Write, path::PathBuf};

use directories::ProjectDirs;
use toml_edit::Document;

use crate::common::jwt_ops;

pub async fn exec(shasta_token: &str, shasta_base_url: &str, shasta_root_cert: &[u8]) {
    // Read configuration file

    // XDG Base Directory Specification
    let project_dirs = ProjectDirs::from(
        "local", /*qualifier*/
        "cscs",  /*organization*/
        "manta", /*application*/
    );

    let mut path_to_manta_configuration_file = PathBuf::from(project_dirs.unwrap().config_dir());

    path_to_manta_configuration_file.push("config.toml"); // ~/.config/manta/config is the file

    log::debug!(
        "Reading manta configuration from {}",
        &path_to_manta_configuration_file.to_string_lossy()
    );

    let config_file_content = fs::read_to_string(path_to_manta_configuration_file.clone())
        .expect("Error reading configuration file");

    let mut doc = config_file_content
        .parse::<Document>()
        .expect("ERROR: could not parse configuration file to TOML");

    let mut settings_hsm_available_vec = jwt_ops::get_claims_from_jwt_token(&shasta_token)
        .unwrap()
        .pointer("/realm_access/roles")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .map(|role_value| role_value.as_str().unwrap().to_string())
        .collect::<Vec<String>>();

    settings_hsm_available_vec
        .retain(|role| !role.eq("offline_access") && !role.eq("uma_authorization"));

    // VALIDATION
    if settings_hsm_available_vec.is_empty() {
        doc.remove("hsm_group");
        println!("hsm group unset");
    } else {
        eprintln!("Can't unset hsm when running in tenant mode.Exit");
        std::process::exit(1);
    }

    // Update configuration file content
    let mut manta_configuration_file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(path_to_manta_configuration_file)
        .unwrap();

    /* let mut output = File::create(path_to_manta_configuration_file).unwrap();
    write!(output, "{}", doc.to_string()); */

    manta_configuration_file
        .write_all(doc.to_string().as_bytes())
        .unwrap();
    manta_configuration_file.flush().unwrap();
}
