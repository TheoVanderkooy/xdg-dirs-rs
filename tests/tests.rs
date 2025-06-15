#[cfg(test)]
use xdg_dirs_rs::*;

use serial_test::serial;
use std::{env, path::Path};

#[test]
#[serial] // because changing environment variables is not safe to do concurrently in different tests
fn test_config_dir() {
    env::set_var("XDG_CONFIG_HOME", "/some/path");
    assert_eq!(
        xdg_user_dir(&dirs::CONFIG, "test").unwrap(),
        Path::new("/some/path/test")
    );

    env::remove_var("XDG_CONFIG_HOME");
    env::set_var("HOME", "/some/home");
    assert_eq!(
        xdg_user_dir(&dirs::CONFIG, "test").unwrap(),
        Path::new("/some/home/.config/test")
    );
}

#[test]
#[serial] // because changing environment variables is not safe to do concurrently in different tests
fn test_data_dir() {
    env::set_var("XDG_DATA_HOME", "/some/path");
    assert_eq!(
        xdg_user_dir(&dirs::DATA, "test").unwrap(),
        Path::new("/some/path/test")
    );

    env::remove_var("XDG_DATA_HOME");
    env::set_var("HOME", "/some/home");
    assert_eq!(
        xdg_user_dir(&dirs::DATA, "test").unwrap(),
        Path::new("/some/home/.local/share/test")
    );
}

#[test]
#[serial] // because changing environment variables is not safe to do concurrently in different tests
fn test_config_dir_no_home() {
    env::remove_var("XDG_CONFIG_HOME");
    env::remove_var("HOME");
    assert_eq!(xdg_user_dir(&dirs::CONFIG, "test"), Err(Error::NoHome));
}

#[test]
#[serial] // because changing environment variables is not safe to do concurrently in different tests
fn test_runtime_dir_not_set() {
    env::remove_var("XDG_RUNTIME_DIR");
    assert_eq!(
        xdg_user_dir(&dirs::RUNTIME, "test"),
        Err(Error::EnvVarNotSet("XDG_RUNTIME_DIR"))
    );
}

#[test]
fn test_error_display() {
    let err = Error::NoHome;
    assert_eq!(err.to_string(), "$HOME is not set");

    let err = Error::EnvVarNotSet("SOME_VAR");
    assert_eq!(err.to_string(), "$SOME_VAR is not set");
}
