//! Safety net: every command the screens call (`invoke("...")` in `src/api.ts`)
//! must be registered in the app's command list in `src/lib.rs`. A command
//! missing from the list still compiles, but its screen fails at run time.

use std::collections::BTreeSet;

/// Every `invoke<...>("name"` or `invoke("name"` in the front end.
fn called_names(api: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for part in api.split("invoke").skip(1) {
        if !(part.starts_with('<') || part.starts_with('(')) {
            continue; // e.g. the import line, or another identifier
        }
        if let Some(rest) = part.split_once('"').map(|(_, r)| r)
            && let Some((name, _)) = rest.split_once('"')
        {
            names.insert(name.to_string());
        }
    }
    names
}

/// Every entry of `tauri::generate_handler![ ... ]`, keeping the last path segment.
fn registered_names(lib: &str) -> BTreeSet<String> {
    let start = lib
        .find("generate_handler![")
        .expect("generate_handler! not found in lib.rs")
        + "generate_handler![".len();
    let end = start
        + lib[start..]
            .find(']')
            .expect("generate_handler! not closed");
    lib[start..end]
        .split(',')
        .map(|entry| entry.trim().rsplit("::").next().unwrap_or("").to_string())
        .filter(|name| !name.is_empty())
        .collect()
}

#[test]
fn every_command_the_screens_call_is_registered() {
    let root = env!("CARGO_MANIFEST_DIR");
    let api = std::fs::read_to_string(format!("{root}/../src/api.ts")).expect("read api.ts");
    let lib = std::fs::read_to_string(format!("{root}/src/lib.rs")).expect("read lib.rs");
    let called = called_names(&api);
    let registered = registered_names(&lib);

    let not_registered: Vec<_> = called.difference(&registered).collect();
    let never_called: Vec<_> = registered.difference(&called).collect();
    assert!(
        not_registered.is_empty() && never_called.is_empty(),
        "called by the screens but not registered: {not_registered:?}; \
         registered but never called: {never_called:?}"
    );
    assert_eq!(called.len(), 27, "expected 27 commands, found {called:?}");
}
