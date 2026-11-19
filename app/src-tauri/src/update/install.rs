//! Replacing the plug-in folder (spec F13): what this copy is, the staging,
//! the swap, the installer, and starting the new version.

use std::path::{Path, PathBuf};

use super::Failure;

/// The plug-in identifier, as `Info.lua` declares it.
const PLUGIN_ID: &str = "com.querton.divelens";
/// Next to the plug-in folder: where an update is prepared.
const STAGING: &str = ".DiveLens-update";
/// Next to the plug-in folder: the previous version, until the next launch.
const PREVIOUS: &str = ".DiveLens-previous";
/// Windows: where the installer is downloaded, in the temporary folder.
const INSTALLER_DIR: &str = "DiveLens-update";

/// What this copy of the app is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Copy {
    /// In a plug-in folder (`*.lrplugin`), as installed. Whether the update
    /// can replace it is only checked when the diver installs
    /// ([`check_installable`]).
    Plugin { folder: PathBuf },
    /// Anywhere else, such as a development build: never updated.
    Development,
}

impl Copy {
    /// Finds the plug-in folder holding the app at `exe` (the running
    /// program), from the path alone: reading next to the app at launch
    /// could make macOS ask for access to the folder holding it (a zip copy
    /// in Documents, say) when the diver did nothing. On macOS, an app
    /// opened from a downloaded zip may run from a temporary read-only copy
    /// (App Translocation): the folder is looked for where it really is.
    pub fn locate(exe: &Path) -> Self {
        let exe = original_path(exe).unwrap_or_else(|| exe.to_path_buf());
        match container(&exe).and_then(Path::parent) {
            Some(folder) if folder.extension().is_some_and(|e| e == "lrplugin") => Copy::Plugin {
                folder: folder.to_path_buf(),
            },
            _ => Copy::Development,
        }
    }

    pub fn folder(&self) -> Option<&Path> {
        match self {
            Copy::Plugin { folder } => Some(folder),
            Copy::Development => None,
        }
    }

    /// The update is offered for installing here (it may still turn out
    /// impossible: [`check_installable`]).
    pub fn installable(&self) -> bool {
        matches!(self, Copy::Plugin { .. })
    }
}

/// Before downloading: `folder` is a Dive Lens plug-in folder that this
/// app can replace.
pub fn check_installable(folder: &Path) -> Result<(), InstallError> {
    if !is_plugin_folder(folder) {
        return Err(InstallError::new(
            Failure::NotInstallable,
            format!("{} is not a Dive Lens plug-in folder", folder.display()),
        ));
    }
    if !writable(folder) {
        return Err(InstallError::new(
            Failure::NotInstallable,
            format!("{} cannot be changed", folder.display()),
        ));
    }
    Ok(())
}

/// What sits in the plug-in folder: the app bundle on macOS
/// (`Dive Lens.app/Contents/MacOS/<program>`), the program itself elsewhere.
fn container(exe: &Path) -> Option<&Path> {
    if cfg!(target_os = "macos") {
        let bundle = exe.parent()?.parent()?.parent()?;
        (bundle.extension()? == "app").then_some(bundle)
    } else {
        Some(exe)
    }
}

/// A Dive Lens plug-in folder as installed: its `Info.lua` names the
/// plug-in. A development folder (`.lrdevplugin`, the repository's) is
/// never one.
pub fn is_plugin_folder(folder: &Path) -> bool {
    if folder.extension().is_some_and(|e| e == "lrdevplugin") {
        return false;
    }
    std::fs::read_to_string(folder.join("Info.lua"))
        .is_ok_and(|info| info.contains(&format!("\"{PLUGIN_ID}\"")))
}

/// Whether a folder can be created in `dir`.
fn can_write_in(dir: &Path) -> bool {
    let probe = dir.join(format!(".DiveLens-write-test-{}", std::process::id()));
    let made = std::fs::create_dir(&probe).is_ok();
    if made {
        let _ = std::fs::remove_dir(&probe);
    }
    made
}

/// The update replaces the folder's content (Windows) or the folder itself
/// in the folder holding it (macOS): both must be writable.
fn writable(folder: &Path) -> bool {
    can_write_in(folder) && folder.parent().is_some_and(can_write_in)
}

/// The version written in a plug-in folder's `Info.lua`: the string given
/// to `display` (the full version), wherever the word also appears.
pub fn plugin_version(folder: &Path) -> Option<String> {
    let info = std::fs::read_to_string(folder.join("Info.lua")).ok()?;
    info.match_indices("display").find_map(|(at, word)| {
        let value = info[at + word.len()..].trim_start().strip_prefix('=')?;
        let value = value.trim_start().strip_prefix('"')?;
        Some(value[..value.find('"')?].to_string())
    })
}

/// Passed to the new version when an update starts it.
pub const AFTER_UPDATE: &str = "--after-update";

/// Removes what the update left, once the new version runs: next to the
/// plug-in folder, the previous version (the app that installed the update
/// ran from it until it quit) and anything prepared but not installed; the
/// Windows installer in the temporary folder.
pub fn clean_up(folder: &Path) {
    let mut left = vec![std::env::temp_dir().join(INSTALLER_DIR)];
    if let Some(parent) = folder.parent() {
        left.extend([parent.join(PREVIOUS), parent.join(STAGING)]);
    }
    for path in left {
        if path.exists() {
            match std::fs::remove_dir_all(&path) {
                Ok(()) => log::info!("update: removed {}", path.display()),
                Err(e) => log::warn!("update: {} not removed: {e}", path.display()),
            }
        }
    }
}

/// What went wrong while installing, with the details for the log.
#[derive(Debug)]
pub struct InstallError {
    pub reason: Failure,
    pub detail: String,
}

impl InstallError {
    fn new(reason: Failure, detail: impl Into<String>) -> Self {
        Self {
            reason,
            detail: detail.into(),
        }
    }
}

fn io(reason: Failure, what: &str) -> impl FnOnce(std::io::Error) -> InstallError + '_ {
    move |e| InstallError::new(reason, format!("{what}: {e}"))
}

/// The folder where the update is prepared, emptied.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))] // Windows runs the installer.
fn fresh_staging(folder: &Path) -> Result<PathBuf, InstallError> {
    let parent = folder
        .parent()
        .ok_or_else(|| InstallError::new(Failure::Files, "no parent folder"))?;
    let staging = parent.join(STAGING);
    if staging.exists() {
        std::fs::remove_dir_all(&staging).map_err(io(Failure::Files, "old staging"))?;
    }
    std::fs::create_dir(&staging).map_err(io(Failure::Files, "staging"))?;
    Ok(staging)
}

/// Checks an unpacked plug-in folder before it replaces the installed one:
/// a Dive Lens plug-in of the version announced, with the app inside.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))] // Windows runs the installer.
pub fn check_package(staged: &Path, version: &str, app: &str) -> Result<(), InstallError> {
    if !is_plugin_folder(staged) {
        return Err(InstallError::new(
            Failure::Package,
            format!("{} is not a Dive Lens plug-in", staged.display()),
        ));
    }
    let found = plugin_version(staged);
    if found.as_deref() != Some(version) {
        return Err(InstallError::new(
            Failure::Package,
            format!("plug-in version {found:?}, {version} announced"),
        ));
    }
    if !staged.join(app).exists() {
        return Err(InstallError::new(
            Failure::Package,
            format!("{app} missing from the package"),
        ));
    }
    Ok(())
}

/// Puts `staged` in place of `folder` (same name, same place) by two
/// renames on the same disk; the previous version is kept next to it until
/// the next launch. When the second rename fails, the previous version is
/// put back.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))] // Windows runs the installer.
pub fn swap(folder: &Path, staged: &Path) -> Result<(), InstallError> {
    let parent = folder
        .parent()
        .ok_or_else(|| InstallError::new(Failure::Files, "no parent folder"))?;
    let previous = parent.join(PREVIOUS);
    if previous.exists() {
        std::fs::remove_dir_all(&previous).map_err(io(Failure::Files, "old previous version"))?;
    }
    std::fs::rename(folder, &previous)
        .map_err(io(Failure::Files, "moving the installed version"))?;
    if let Err(e) = std::fs::rename(staged, folder) {
        let restored = std::fs::rename(&previous, folder);
        return Err(InstallError::new(
            Failure::Files,
            format!("new version not moved in: {e}; previous version put back: {restored:?}"),
        ));
    }
    Ok(())
}

/// macOS: unpacks the zipped plug-in folder `bytes` (already verified) next
/// to `folder`, checks it and swaps it in. Returns the new app, to start.
#[cfg(target_os = "macos")]
pub fn install(folder: &Path, bytes: &[u8], version: &str) -> Result<PathBuf, InstallError> {
    let staging = fresh_staging(folder)?;
    let installed = install_from(folder, &staging, bytes, version);
    // Nothing prepared is left behind, installed or not.
    let _ = std::fs::remove_dir_all(&staging);
    installed
}

#[cfg(target_os = "macos")]
fn install_from(
    folder: &Path,
    staging: &Path,
    bytes: &[u8],
    version: &str,
) -> Result<PathBuf, InstallError> {
    use std::process::Command;
    const APP: &str = "Dive Lens.app";

    let zip = staging.join("update.zip");
    std::fs::write(&zip, bytes).map_err(io(Failure::Files, "writing the download"))?;
    // ditto keeps the app bundle as macOS expects it (links, permissions,
    // signature), as the release workflow made it.
    let out = Command::new("/usr/bin/ditto")
        .args(["-x", "-k"])
        .arg(&zip)
        .arg(staging)
        .output()
        .map_err(io(Failure::Package, "ditto"))?;
    if !out.status.success() {
        return Err(InstallError::new(
            Failure::Package,
            format!("ditto: {}", String::from_utf8_lossy(&out.stderr).trim()),
        ));
    }
    let _ = std::fs::remove_file(&zip);
    let staged = staging.join("DiveLens.lrplugin");
    check_package(&staged, version, APP)?;
    let out = Command::new("/usr/bin/codesign")
        .args(["--verify", "--deep", "--strict"])
        .arg(staged.join(APP))
        .output()
        .map_err(io(Failure::Package, "codesign"))?;
    if !out.status.success() {
        return Err(InstallError::new(
            Failure::Package,
            format!("codesign: {}", String::from_utf8_lossy(&out.stderr).trim()),
        ));
    }
    swap(folder, &staged)?;
    Ok(folder.join(APP))
}

/// macOS: starts `app` once this process has quit (a second instance
/// started before would only bring this one forward), on Settings.
#[cfg(target_os = "macos")]
pub fn relaunch(app: &Path) -> std::io::Result<()> {
    use std::process::{Command, Stdio};
    Command::new("/bin/sh")
        .arg("-c")
        .arg(
            "while kill -0 \"$1\" 2>/dev/null; do sleep 0.2; done; \
             exec /usr/bin/open \"$2\" --args --after-update --screen settings",
        )
        .arg("sh")
        .arg(std::process::id().to_string())
        .arg(app)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
}

/// Windows: writes the installer `bytes` (already verified) to the
/// temporary folder and starts it, silent, on `folder`; it closes this app
/// if still running, replaces the folder's content and starts the new app
/// (`/update=1`, see packaging/windows/divelens.iss). The caller quits.
#[cfg(windows)]
pub fn install(folder: &Path, bytes: &[u8], version: &str) -> Result<PathBuf, InstallError> {
    use std::os::windows::process::CommandExt;
    use std::process::{Command, Stdio};
    const DETACHED_PROCESS: u32 = 0x0000_0008;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;

    let dir = std::env::temp_dir().join(INSTALLER_DIR);
    if dir.exists() {
        let _ = std::fs::remove_dir_all(&dir);
    }
    std::fs::create_dir_all(&dir).map_err(io(Failure::Files, "temporary folder"))?;
    let setup = dir.join(format!("DiveLens-{version}-Windows-Setup.exe"));
    std::fs::write(&setup, bytes).map_err(io(Failure::Files, "writing the installer"))?;
    Command::new(&setup)
        .args([
            "/VERYSILENT",
            "/SUPPRESSMSGBOXES",
            "/NORESTART",
            "/CLOSEAPPLICATIONS",
            "/update=1",
        ])
        .arg(format!("/DIR={}", folder.display()))
        .arg(format!("/LOG={}", dir.join("setup.log").display()))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP)
        .spawn()
        .map_err(io(Failure::Package, "starting the installer"))?;
    Ok(setup)
}

/// Windows: the installer starts the new app itself.
#[cfg(windows)]
pub fn relaunch(_installer: &Path) -> std::io::Result<()> {
    Ok(())
}

#[cfg(not(any(target_os = "macos", windows)))]
pub fn install(_folder: &Path, _bytes: &[u8], _version: &str) -> Result<PathBuf, InstallError> {
    Err(InstallError::new(
        Failure::NotInstallable,
        "unsupported system",
    ))
}

#[cfg(not(any(target_os = "macos", windows)))]
pub fn relaunch(_app: &Path) -> std::io::Result<()> {
    Ok(())
}

/// macOS: where a translocated app really is (`None` when it is not
/// translocated, or macOS cannot tell).
#[cfg(target_os = "macos")]
fn original_path(exe: &Path) -> Option<PathBuf> {
    use std::ffi::{CStr, c_char, c_void};
    use std::os::unix::ffi::OsStrExt;

    type CFTypeRef = *const c_void;
    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        fn CFURLCreateFromFileSystemRepresentation(
            allocator: CFTypeRef,
            buffer: *const u8,
            length: isize,
            is_directory: u8,
        ) -> CFTypeRef;
        fn CFURLGetFileSystemRepresentation(
            url: CFTypeRef,
            resolve_against_base: u8,
            buffer: *mut u8,
            max_length: isize,
        ) -> u8;
        fn CFRelease(cf: CFTypeRef);
    }
    #[link(name = "Security", kind = "framework")]
    unsafe extern "C" {
        fn SecTranslocateIsTranslocatedURL(
            path: CFTypeRef,
            is_translocated: *mut u8,
            error: *mut CFTypeRef,
        ) -> u8;
        fn SecTranslocateCreateOriginalPathForURL(
            translocated_path: CFTypeRef,
            error: *mut CFTypeRef,
        ) -> CFTypeRef;
    }

    let bytes = exe.as_os_str().as_bytes();
    // SAFETY: CoreFoundation and Security calls on a URL made here and
    // released here; the buffer outlives the call that fills it.
    unsafe {
        let url = CFURLCreateFromFileSystemRepresentation(
            std::ptr::null(),
            bytes.as_ptr(),
            bytes.len() as isize,
            0,
        );
        if url.is_null() {
            return None;
        }
        let mut translocated = 0u8;
        let asked = SecTranslocateIsTranslocatedURL(url, &mut translocated, std::ptr::null_mut());
        let mut found = None;
        if asked != 0 && translocated != 0 {
            let original = SecTranslocateCreateOriginalPathForURL(url, std::ptr::null_mut());
            if !original.is_null() {
                let mut buffer = vec![0u8; 4096];
                if CFURLGetFileSystemRepresentation(
                    original,
                    1,
                    buffer.as_mut_ptr(),
                    buffer.len() as isize,
                ) != 0
                {
                    let path = CStr::from_ptr(buffer.as_ptr() as *const c_char);
                    found = Some(PathBuf::from(std::ffi::OsStr::from_bytes(path.to_bytes())));
                }
                CFRelease(original);
            }
        }
        CFRelease(url);
        if let Some(path) = &found {
            log::info!("update: app translocated, really at {}", path.display());
        }
        found
    }
}

#[cfg(not(target_os = "macos"))]
fn original_path(_exe: &Path) -> Option<PathBuf> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::update::test_support::{plugin, temp_dir};

    #[test]
    fn finds_the_installed_plug_in_folder_only() {
        let dir = temp_dir("locate");
        let folder = plugin(&dir, "DiveLens.lrplugin", "2.1.0");
        assert!(is_plugin_folder(&folder));
        assert_eq!(plugin_version(&folder).as_deref(), Some("2.1.0"));

        // As the app finds itself.
        let exe = if cfg!(target_os = "macos") {
            let macos = folder.join("Dive Lens.app/Contents/MacOS");
            std::fs::create_dir_all(&macos).unwrap();
            macos.join("divelens")
        } else {
            folder.join("Dive Lens.exe")
        };
        assert_eq!(
            Copy::locate(&exe),
            Copy::Plugin {
                folder: folder.clone()
            }
        );
        assert!(check_installable(&folder).is_ok());

        // A development build, in the repository's target folder.
        let build = dir.join("target/release");
        std::fs::create_dir_all(&build).unwrap();
        assert_eq!(Copy::locate(&build.join("divelens")), Copy::Development);

        // The repository's development plug-in folder.
        let dev = plugin(&dir, "DiveLens.lrdevplugin", "2.1.0");
        assert!(!is_plugin_folder(&dev));
        let dev_exe = if cfg!(target_os = "macos") {
            dev.join("Dive Lens.app/Contents/MacOS/divelens")
        } else {
            dev.join("Dive Lens.exe")
        };
        assert_eq!(Copy::locate(&dev_exe), Copy::Development);

        // Another plug-in.
        let other = dir.join("Other.lrplugin");
        std::fs::create_dir_all(&other).unwrap();
        std::fs::write(
            other.join("Info.lua"),
            "return { LrToolkitIdentifier = \"com.example.other\" }",
        )
        .unwrap();
        assert!(!is_plugin_folder(&other));
        assert!(!is_plugin_folder(&dir.join("Missing.lrplugin")));
        // Found by its name, refused when installing.
        let refused = check_installable(&other).unwrap_err();
        assert_eq!(refused.reason, Failure::NotInstallable);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn reads_the_version_of_the_repository_plug_in() {
        let plugin =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../plugin/DiveLens.lrdevplugin");
        assert_eq!(
            plugin_version(&plugin).as_deref(),
            Some(env!("CARGO_PKG_VERSION"))
        );
    }

    #[test]
    fn a_package_must_be_the_version_announced_with_the_app() {
        let dir = temp_dir("package");
        let staged = plugin(&dir, "DiveLens.lrplugin", "2.1.0");
        assert!(check_package(&staged, "2.1.0", "Dive Lens.exe").is_ok());
        let wrong = check_package(&staged, "2.2.0", "Dive Lens.exe").unwrap_err();
        assert_eq!(wrong.reason, Failure::Package);
        let no_app = check_package(&staged, "2.1.0", "Dive Lens.app").unwrap_err();
        assert_eq!(no_app.reason, Failure::Package);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn swaps_the_folder_and_cleans_up_at_the_next_launch() {
        let dir = temp_dir("swap");
        let folder = plugin(&dir, "DiveLens.lrplugin", "2.0.0");
        let staging = fresh_staging(&folder).unwrap();
        let staged = plugin(&staging, "DiveLens.lrplugin", "2.1.0");

        swap(&folder, &staged).unwrap();
        assert_eq!(plugin_version(&folder).as_deref(), Some("2.1.0"));
        let previous = dir.join(PREVIOUS);
        assert_eq!(plugin_version(&previous).as_deref(), Some("2.0.0"));
        // Nothing next to the folder that Lightroom would load.
        let plugins: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|x| x == "lrplugin"))
            .collect();
        assert_eq!(plugins.len(), 1);

        clean_up(&folder);
        assert!(!previous.exists());
        assert!(!dir.join(STAGING).exists());
        assert_eq!(plugin_version(&folder).as_deref(), Some("2.1.0"));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_failed_swap_puts_the_previous_version_back() {
        let dir = temp_dir("rollback");
        let folder = plugin(&dir, "DiveLens.lrplugin", "2.0.0");
        let missing = dir.join(STAGING).join("DiveLens.lrplugin");
        let failed = swap(&folder, &missing).unwrap_err();
        assert_eq!(failed.reason, Failure::Files);
        assert_eq!(plugin_version(&folder).as_deref(), Some("2.0.0"));
        assert!(!dir.join(PREVIOUS).exists());
        std::fs::remove_dir_all(dir).unwrap();
    }

    /// Installs a real package over a copy of itself, as an update would
    /// (macOS): unpacking with its app bundle intact, checking it, swapping
    /// the folder. Set `DIVELENS_TEST_PACKAGE` to a zip the release
    /// workflow made (or one made the same way, its app signed), then run
    /// `cargo test -p divelens -- --ignored installs_a_real_package`.
    #[cfg(target_os = "macos")]
    #[test]
    #[ignore = "needs a real package: DIVELENS_TEST_PACKAGE"]
    fn installs_a_real_package() {
        let zip = std::env::var("DIVELENS_TEST_PACKAGE").expect("DIVELENS_TEST_PACKAGE");
        let bytes = std::fs::read(&zip).unwrap();
        let dir = temp_dir("real");
        // The installed copy: the same package, with a file of its own.
        let unpacked = std::process::Command::new("/usr/bin/ditto")
            .args(["-x", "-k", &zip])
            .arg(&dir)
            .status()
            .unwrap();
        assert!(unpacked.success());
        let folder = dir.join("DiveLens.lrplugin");
        let version = plugin_version(&folder).expect("a plug-in version");
        std::fs::write(folder.join("left-over.txt"), "previous version").unwrap();

        let app = install(&folder, &bytes, &version).unwrap();
        assert_eq!(app, folder.join("Dive Lens.app"));
        assert!(!folder.join("left-over.txt").exists());
        assert!(dir.join(PREVIOUS).join("left-over.txt").exists());
        assert!(!dir.join(STAGING).exists());
        let verified = std::process::Command::new("/usr/bin/codesign")
            .args(["--verify", "--deep", "--strict"])
            .arg(&app)
            .status()
            .unwrap();
        assert!(verified.success(), "the app bundle came through intact");

        // A package of another version than announced is refused, and the
        // installed one is left as it is.
        let refused = install(&folder, &bytes, "99.0.0").unwrap_err();
        assert_eq!(refused.reason, Failure::Package);
        assert_eq!(plugin_version(&folder).as_deref(), Some(version.as_str()));

        clean_up(&folder);
        assert!(!dir.join(PREVIOUS).exists());
        assert!(!dir.join(STAGING).exists());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
