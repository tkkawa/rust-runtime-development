use std::env;
use std::ffi::CString;

use anyhow::{bail, Ok, Result};
use nix::errno::Errno;
use nix::unistd;

pub fn do_exec(path: &str, args: &[String]) -> Result<()> {
    let p = CString::new(path.to_string()).unwrap();
    let a: Vec<CString> = args
        .iter()
        .map(|s| CString::new(s.to_string()).unwrap_or_default())
        .collect();

    log::debug!("filename: {:?}, args: {:?}", p, a);
    unistd::execvp(&p, &a)?;
    log::debug!("finish execvp");
    Ok(())
}

pub fn set_env_val(env: &Vec<String>) {
    for i in 0..env.len() {
        let split_path: Vec<&str> = env[i].split('=').collect();
        env::set_var(split_path[0], split_path[1]);
    }
}
