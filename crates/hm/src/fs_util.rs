//! Small filesystem helpers for atomic, permission-restricted writes.
//!
//! The main entry point is [`write_atomic_restricted`]. It is used by
//! [`crate::creds_store`] (file-backed credential store) and by
//! `Config::save`, both of which write into `~/.harmont/`
//! (see `config::user_config_dir`).

use anyhow::{Context, Result};
use std::path::Path;

/// Write `contents` to `path` atomically with `file_mode`, ensuring the
/// parent directory exists and is set to `dir_mode`.
///
/// On Unix the target file is created with `OpenOptions::mode(file_mode)`
/// before any bytes are written, closing the TOCTOU window that
/// `fs::write(…)` + `set_permissions(…)` opens. The parent directory is
/// created with `DirBuilder::mode(dir_mode)`; if the directory already
/// exists with a looser mode, it is tightened.
///
/// On non-Unix platforms the mode arguments are ignored and the function
/// falls back to `std::fs::create_dir_all` + tempfile + rename.
///
/// Atomicity: contents are written to a sibling tempfile and then
/// `rename`d over `path`, so readers always observe either the full old
/// contents or the full new contents — never a truncated file.
///
/// # Errors
///
/// Returns an error if `path` has no parent or no file-name component,
/// the parent directory cannot be created or chmod'd to `dir_mode`, the
/// tempfile cannot be opened with `file_mode` or written, or the final
/// `rename` over `path` fails. The tempfile is cleaned up on rename
/// failure so secret material doesn't linger.
pub fn write_atomic_restricted(
    path: &Path,
    contents: &[u8],
    file_mode: u32,
    dir_mode: u32,
) -> Result<()> {
    let parent = path
        .parent()
        .with_context(|| format!("{} has no parent directory", path.display()))?;

    create_dir_with_mode(parent, dir_mode)
        .with_context(|| format!("creating {}", parent.display()))?;

    // Tempfile name is unique per process (pid) and per target filename,
    // which is sufficient because write_atomic_restricted is never called
    // concurrently on the same target from within a single process.
    let file_name = path
        .file_name()
        .with_context(|| format!("{} has no file name", path.display()))?
        .to_os_string();
    let mut tmp_name = file_name;
    tmp_name.push(format!(".tmp.{}", std::process::id()));
    let tmp_path = parent.join(&tmp_name);

    write_file_with_mode(&tmp_path, contents, file_mode)
        .with_context(|| format!("writing {}", tmp_path.display()))?;

    let persist_result = std::fs::rename(&tmp_path, path)
        .with_context(|| format!("renaming {} -> {}", tmp_path.display(), path.display()));

    if persist_result.is_err() {
        // Clean up the orphaned tempfile — it may contain secret material
        // and we don't want it sitting at an unexpected path.
        let _ = std::fs::remove_file(&tmp_path);
    }
    persist_result?;

    Ok(())
}

/// Remove a file if it exists; silently return `Ok(())` if it does not.
///
/// # Errors
///
/// Returns an error if `remove_file` fails for any reason other than
/// `NotFound` (typically permission denied or the path being a
/// non-empty directory).
pub fn remove_if_exists(path: &Path) -> Result<()> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e).with_context(|| format!("removing {}", path.display())),
    }
}

#[cfg(unix)]
fn create_dir_with_mode(dir: &Path, mode: u32) -> std::io::Result<()> {
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
    if dir.exists() {
        let current = std::fs::metadata(dir)?.permissions().mode() & 0o777;
        if current != mode {
            std::fs::set_permissions(dir, std::fs::Permissions::from_mode(mode))?;
        }
    } else {
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(mode)
            .create(dir)?;
    }
    Ok(())
}

#[cfg(not(unix))]
fn create_dir_with_mode(dir: &Path, _mode: u32) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)
}

#[cfg(unix)]
fn write_file_with_mode(path: &Path, contents: &[u8], mode: u32) -> std::io::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(mode)
        .open(path)?;
    f.write_all(contents)?;
    f.sync_all()?;
    Ok(())
}

#[cfg(not(unix))]
fn write_file_with_mode(path: &Path, contents: &[u8], _mode: u32) -> std::io::Result<()> {
    std::fs::write(path, contents)
}

#[cfg(all(test, unix))]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn writes_file_and_dir_with_requested_modes() {
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("sub").join("creds");
        write_atomic_restricted(&target, b"hello", 0o600, 0o700).unwrap();

        assert_eq!(std::fs::read(&target).unwrap(), b"hello");
        let file_mode = std::fs::metadata(&target).unwrap().permissions().mode() & 0o777;
        let dir_mode = std::fs::metadata(target.parent().unwrap())
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(
            file_mode, 0o600,
            "file mode must be 0o600, got {file_mode:o}"
        );
        assert_eq!(dir_mode, 0o700, "dir mode must be 0o700, got {dir_mode:o}");
    }

    #[test]
    fn overwrites_existing_file_preserving_mode() {
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("creds");
        write_atomic_restricted(&target, b"v1", 0o600, 0o700).unwrap();
        write_atomic_restricted(&target, b"v2", 0o600, 0o700).unwrap();

        assert_eq!(std::fs::read(&target).unwrap(), b"v2");
        let mode = std::fs::metadata(&target).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }

    #[test]
    fn tightens_existing_dir_with_looser_mode() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("loose");
        std::fs::create_dir(&dir).unwrap();
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o755)).unwrap();

        let target = dir.join("creds");
        write_atomic_restricted(&target, b"x", 0o600, 0o700).unwrap();

        let dir_mode = std::fs::metadata(&dir).unwrap().permissions().mode() & 0o777;
        assert_eq!(dir_mode, 0o700);
    }

    #[test]
    fn remove_if_exists_is_idempotent() {
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("nothing");
        remove_if_exists(&target).unwrap();
        std::fs::write(&target, "x").unwrap();
        remove_if_exists(&target).unwrap();
        assert!(!target.exists());
    }
}
