use std::env;
use casdoor_rust_sdk::CasdoorConfig;
use once_cell::sync::Lazy;
use std::fs;

pub const IAM_URL: &str = "IAM_URL";
pub const IAM_CLIENT_ID: &str = "IAM_CLIENT_ID";
pub const IAM_CLIENT_SECRET: &str = "IAM_CLIENT_SECRET";
pub const IAM_PUB_CERT_FILE: &str = "IAM_PUB_CERT_FILE";
pub const IAM_ORG_NAME: &str = "IAM_ORG_NAME";
pub const IAM_APP_NAME: &str = "IAM_APP_NAME";


pub static CASDOOR_CONF: Lazy<CasdoorConfig> = Lazy::new(|| {
    let url = load_env_var_or_fail(IAM_URL);
    let client_id = load_env_var_or_fail(IAM_CLIENT_ID);
    let clietn_secret = load_env_var_or_fail(IAM_CLIENT_SECRET);
    let cert_file_path = load_env_var_or_fail(IAM_PUB_CERT_FILE);
    let cert = fs::read_to_string(&cert_file_path)
        .unwrap_or_else(|err| panic!("cannot read file {}: {}", cert_file_path, err));
    let org = load_env_var_or_fail(IAM_ORG_NAME);
    let app_name = load_env_var_or_fail(IAM_APP_NAME);

    CasdoorConfig::new(url, 
        client_id, clietn_secret, 
        cert, org, Some(app_name))
});

pub fn load_env_var(var: &str, default: &str) -> String {
    env::var(var).unwrap_or_else(|err| {
        tracing::warn!("cannot load {} env var: {}", var, err);
        default.to_owned()
    })
}

pub fn load_env_var_or_fail(var: &str) -> String {
    env::var(var).unwrap_or_else(|err| {
        panic!("cannot load {} env var: {}", var, err);
    })
}