use std::{env, path::PathBuf, str::FromStr};

pub struct XdgDir {
    env_var: &'static str,
    home_fallback: Option<&'static str>,
    system_var: Option<&'static str>,
    system_fallback: Option<&'static [&'static str]>,
}

pub mod dirs {
    use super::XdgDir;

    pub const CONFIG: XdgDir = XdgDir {
        env_var: "XDG_CONFIG_HOME",
        home_fallback: Some(".config/"),
        system_var: Some("XDG_CONFIG_DIRS"),
        system_fallback: Some(&["/etc/xdg"]),
    };

    pub const DATA: XdgDir = XdgDir {
        env_var: "XDG_DATA_HOME",
        home_fallback: Some(".local/share/"),
        system_var: Some("XDG_DATA_DIRS"),
        system_fallback: Some(&["/usr/local/share/", "/usr/share/"]),
    };

    pub const CACHE: XdgDir = XdgDir {
        env_var: "XDG_CACHE_HOME",
        home_fallback: Some(".cache/"),
        system_var: None,
        system_fallback: None,
    };

    pub const STATE: XdgDir = XdgDir {
        env_var: "XDG_STATE_HOME",
        home_fallback: Some(".local/state/"),
        system_var: None,
        system_fallback: None,
    };

    pub const RUNTIME: XdgDir = XdgDir {
        env_var: "XDG_RUNTIME_DIR",
        home_fallback: None,
        system_var: None,
        system_fallback: None,
    };
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    #[error("$HOME is not set")]
    NoHome,

    #[error("${0} is not set")]
    EnvVarNotSet(&'static str),

    #[error("Path {0} not found in any available location")]
    NotFound(String),
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

/// Returns the list of system paths for a given XDG basedir, with the provided suffix, based on the relevant environment variable.
/// This does NOT create the directories or check that they exist, only returns the list of candidates.
pub fn xdg_system_dirs(xdg_dir: &XdgDir, suffix: &str) -> Result<Vec<PathBuf>, Error> {
    // Parse the env var, if it is set
    // Note: this follows the same format as PATH, which does not allow for any escaping or quoting of ':' in path names
    if let Some(var) = xdg_dir.system_var {
        if let Ok(val) = env::var(var) {
            if !val.is_empty() {
                return Ok(val
                    .split(':')
                    .map(|p| {
                        let Ok(mut path) = PathBuf::from_str(p);
                        path.push(suffix);
                        path
                    })
                    .collect());
            }
        }
    }

    // If the env var is not set, fall back to the default
    if let Some(paths) = xdg_dir.system_fallback {
        return Ok(paths
            .iter()
            .map(|p| {
                let Ok(mut path) = PathBuf::from_str(p);
                path.push(suffix);
                path
            })
            .collect());
    }

    Err(Error::NotFound(suffix.to_string()))
}

/// Search all relevant paths for the given XDG base directory and find the first one where `suffix` exists.
/// This follows the precedence of searching the path in the user's HOME first, and then system fallbacks (if applicable) in order.
///
/// Notes:
///  - This only checks that the path exists and is accessible, not type (file vs directory) or exact permissions on the file/directory'
///  - Beware of TOCTOU issues
pub fn xdg_location_of(xdg_dir: &XdgDir, suffix: &str) -> Result<PathBuf, Error> {
    // Check user location
    if let Ok(user_loc) = xdg_user_dir(xdg_dir, suffix) {
        if let Ok(user_loc) = user_loc.canonicalize() {
            if user_loc.exists() {
                return Ok(user_loc);
            }
        }
    }

    // Check system locations if not present in any user location
    if let Ok(sys_paths) = xdg_system_dirs(xdg_dir, suffix) {
        for p in sys_paths {
            if let Ok(p) = p.canonicalize() {
                if p.exists() {
                    return Ok(p);
                }
            }
        }
    }

    // Didn't find it
    Err(Error::NotFound(suffix.to_string()))
}
