use tempfile::tempdir;
#[cfg(test)]
use xdg_dirs::*;

use serial_test::serial;
use std::{
    env,
    fs::{self, File},
    path::Path,
    path::PathBuf,
};

#[test]
// Safety: serial because env var access must be single-threaded (even with different vars)
#[serial]
fn test_user_config_dir() {
    unsafe { env::set_var("XDG_CONFIG_HOME", "/some/path") };
    assert_eq!(
        xdg_user_dir(&dirs::CONFIG, "test").unwrap(),
        Path::new("/some/path/test")
    );

    unsafe { env::remove_var("XDG_CONFIG_HOME") };
    unsafe { env::set_var("HOME", "/some/home") };
    assert_eq!(
        xdg_user_dir(&dirs::CONFIG, "test").unwrap(),
        Path::new("/some/home/.config/test")
    );
}

#[test]
// Safety: serial because env var access must be single-threaded (even with different vars)
#[serial]
fn test_user_data_dir() {
    unsafe { env::set_var("XDG_DATA_HOME", "/some/path") };
    assert_eq!(
        xdg_user_dir(&dirs::DATA, "test").unwrap(),
        Path::new("/some/path/test")
    );

    unsafe { env::remove_var("XDG_DATA_HOME") };
    unsafe { env::set_var("HOME", "/some/home") };
    assert_eq!(
        xdg_user_dir(&dirs::DATA, "test").unwrap(),
        Path::new("/some/home/.local/share/test")
    );
}

#[test]
// Safety: serial because env var access must be single-threaded (even with different vars)
#[serial]
fn test_user_config_dir_no_home() {
    unsafe { env::remove_var("XDG_CONFIG_HOME") };
    unsafe { env::remove_var("HOME") };
    assert_eq!(xdg_user_dir(&dirs::CONFIG, "test"), Err(Error::NoHome));
}

#[test]
// Safety: serial because env var access must be single-threaded (even with different vars)
#[serial]
fn test_runtime_dir_not_set() {
    unsafe { env::remove_var("XDG_RUNTIME_DIR") };
    assert_eq!(
        xdg_user_dir(&dirs::RUNTIME, "test"),
        Err(Error::EnvVarNotSet("XDG_RUNTIME_DIR"))
    );
}

#[test]
// Safety: serial because env var access must be single-threaded (even with different vars)
#[serial]
fn test_sys_config_dir() {
    unsafe { env::set_var("XDG_CONFIG_DIRS", "/some/path") };
    assert_eq!(
        xdg_system_dirs(&dirs::CONFIG, "test").unwrap(),
        vec![Path::new("/some/path/test")]
    );

    unsafe { env::set_var("XDG_CONFIG_DIRS", "/some/path:/some/other/path") };
    assert_eq!(
        xdg_system_dirs(&dirs::CONFIG, "test").unwrap(),
        vec![
            Path::new("/some/path/test"),
            Path::new("/some/other/path/test")
        ]
    );

    unsafe { env::remove_var("XDG_CONFIG_DIRS") };
    assert_eq!(
        xdg_system_dirs(&dirs::CONFIG, "test").unwrap(),
        vec![Path::new("/etc/xdg/test")]
    );
}

#[test]
// Safety: serial because env var access must be single-threaded (even with different vars)
#[serial]
fn test_sys_data_dir() {
    unsafe { env::set_var("XDG_DATA_DIRS", "/some/path") };
    assert_eq!(
        xdg_system_dirs(&dirs::DATA, "test").unwrap(),
        vec![Path::new("/some/path/test")]
    );

    unsafe { env::set_var("XDG_DATA_DIRS", "/some/path:/some/other/path") };
    assert_eq!(
        xdg_system_dirs(&dirs::DATA, "test").unwrap(),
        vec![
            Path::new("/some/path/test"),
            Path::new("/some/other/path/test")
        ]
    );

    unsafe { env::remove_var("XDG_DATA_DIRS") };
    assert_eq!(
        xdg_system_dirs(&dirs::DATA, "test").unwrap(),
        vec![
            Path::new("/usr/local/share/test"),
            Path::new("/usr/share/test")
        ]
    );
}

#[test]
fn test_sys_cache_dir() {
    assert_eq!(
        xdg_system_dirs(&dirs::CACHE, "test"),
        Err(Error::NotFound("test".to_string()))
    );
}

#[test]
fn test_error_display() {
    let err = Error::NoHome;
    assert_eq!(err.to_string(), "$HOME is not set");

    let err = Error::EnvVarNotSet("SOME_VAR");
    assert_eq!(err.to_string(), "$SOME_VAR is not set");
}

#[test]
// Safety: serial because env var access must be single-threaded (even with different vars)
#[serial]
fn test_xdg_location_of() {
    let mut test_dir = PathBuf::from(tempdir().unwrap().path());

    let mut home_dir = test_dir.clone();
    home_dir.push("home");

    test_dir.push("sys");
    let mut sysa = test_dir.clone();
    sysa.push("a");
    let mut sysb = test_dir.clone();
    sysb.push("b");

    fs::create_dir_all(home_dir.clone()).unwrap();
    fs::create_dir_all(sysa.clone()).unwrap();
    fs::create_dir_all(sysb.clone()).unwrap();

    unsafe { env::set_var("XDG_CONFIG_HOME", home_dir.clone()) };
    unsafe {
        env::set_var(
            "XDG_CONFIG_DIRS",
            format!("{0}:{1}", sysa.display(), sysb.display()),
        )
    };

    let suffix = "xyz";

    assert_eq!(
        Err(Error::NotFound(suffix.to_string())),
        xdg_location_of(&dirs::CONFIG, suffix)
    );

    let mut fb = sysb.clone();
    fb.push(suffix);
    File::create(fb.clone()).unwrap();

    assert_eq!(fb, xdg_location_of(&dirs::CONFIG, suffix).unwrap());

    let mut fa = sysa.clone();
    fa.push(suffix);
    fs::create_dir_all(fa.clone()).unwrap();

    assert_eq!(fa, xdg_location_of(&dirs::CONFIG, suffix).unwrap());

    let mut fh = home_dir.clone();
    fh.push(suffix);
    File::create(fh.clone()).unwrap();

    assert_eq!(fh, xdg_location_of(&dirs::CONFIG, suffix).unwrap());
}
