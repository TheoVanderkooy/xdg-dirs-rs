use std::{env, path::PathBuf, str::FromStr};

pub struct XdgDir {
    env_var: &'static str,
    home_fallback: Option<&'static str>,
    #[allow(unused)] // TODO use this
    system_fallback_var: Option<&'static str>,
    #[allow(unused)] // TODO use this
    system_fallback_dirs: Option<&'static [&'static str]>,
}

pub mod dirs {
    use super::XdgDir;

    pub const CONFIG: XdgDir = XdgDir {
        env_var: "XDG_CONFIG_HOME",
        home_fallback: Some(".config/"),
        system_fallback_var: Some("XDG_CONFIG_DIRS"),
        system_fallback_dirs: Some(&["/etc/xdg"]),
    };

    pub const DATA: XdgDir = XdgDir {
        env_var: "XDG_DATA_HOME",
        home_fallback: Some(".local/share/"),
        system_fallback_var: Some("XDG_DATA_DIRS"),
        system_fallback_dirs: Some(&["/usr/local/share/", "/usr/share/"]),
    };

    pub const CACHE: XdgDir = XdgDir {
        env_var: "XDG_CACHE_HOME",
        home_fallback: Some(".cache/"),
        system_fallback_var: None,
        system_fallback_dirs: None,
    };

    pub const STATE: XdgDir = XdgDir {
        env_var: "XDG_STATE_HOME",
        home_fallback: Some(".local/state/"),
        system_fallback_var: None,
        system_fallback_dirs: None,
    };

    pub const RUNTIME: XdgDir = XdgDir {
        env_var: "XDG_RUNTIME_DIR",
        home_fallback: None,
        system_fallback_var: None,
        system_fallback_dirs: None,
    };
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("$HOME is not set")]
    NoHome,

    #[error("${0} is not set")]
    EnvVarNotSet(&'static str),

    #[error("Some other error")]
    Other,
}

/// Returns the user-path of a given XDG basedir, with the provided suffix, based on the relevant environment variables.
/// This does NOT create the directory or check that it exists, and does not fall back to system-wide defaults if it is missing or user-level values are not set.
pub fn xdg_user_dir(xdg_dir: &XdgDir, suffix: &str) -> Result<PathBuf, Error> {
    let mut config_path = env::var(xdg_dir.env_var)
        // Check the normal environment variable first
        .map(|p| {
            let Ok(path) = PathBuf::from_str(&*p);
            path
        })
        // If not set, check the default value under $HOME (or return error if that doesn't apply)
        .or_else(|_| match xdg_dir.home_fallback {
            Some(home_dir) => env::var("HOME")
                .map(|p| {
                    let Ok(mut home_path) = PathBuf::from_str(&*p);
                    home_path.push(home_dir);
                    home_path
                })
                .map_err(|_| Error::NoHome),
            None => Err(Error::EnvVarNotSet(xdg_dir.env_var)),
        });

    if let Ok(ref mut path) = config_path {
        path.push(suffix);
    }

    config_path
}

pub fn xdg_config_dir(suffix: &str) -> Result<PathBuf, Error> {
    xdg_user_dir(&dirs::CONFIG, suffix)
}

/*
TODO:
 - for config/data/cache: should have a way to find any existing directory, including the system fallbacks
*/
