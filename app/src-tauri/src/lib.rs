mod applog;
mod apply;
mod collections;
mod dive_log;
mod files;
mod lightroom;
mod settings;
mod tag;
mod update;

use std::sync::Mutex;
use std::sync::mpsc;

use divelens_core::apps::Kept;
use tauri::{Emitter, Manager};

use dive_log::Writes;
use files::config_dir;
use lightroom::Status;
use lightroom::events::{Lightroom, SelectionPreview};
use settings::{LogFile, Settings};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // First: a second launch (Lightroom menu while the app is open but not
        // yet linked) brings this window forward on the requested screen.
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            match lightroom::commands::screen_arg(args) {
                Some(screen) => lightroom::events::show_screen(app, screen),
                None => lightroom::events::bring_to_front(app),
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            settings::dive_apps,
            settings::app_info,
            settings::get_settings,
            settings::update_settings,
            settings::export_settings,
            settings::check_settings_file,
            settings::import_settings,
            dive_log::load_dives,
            dive_log::dive_profile,
            dive_log::set_source_path,
            lightroom::commands::lightroom_status,
            lightroom::commands::lightroom_writing,
            lightroom::commands::initial_screen,
            tag::tag_selection,
            tag::find_selection_photos,
            tag::tag_preview,
            tag::suggest_tag_offsets,
            lightroom::commands::selection_preview,
            collections::find_collection_photos,
            collections::collections_preview,
            collections::suggest_offsets,
            collections::apply_collections,
            collections::collection_sets,
            update::update_status,
            update::check_update,
            update::dismiss_update,
            update::install_update
        ])
        .setup(|app| {
            let settings = Settings::load(&config_dir(app.handle()));
            let version = app.package_info().version.to_string();
            let log_path = app.path().home_dir().ok().map(|home| {
                let path = applog::start(applog::folder(&home), settings.log_level);
                log::info!(
                    "Dive Lens {} on {}, log {}",
                    version,
                    std::env::consts::OS,
                    path.display()
                );
                path
            });
            app.manage(LogFile(log_path));
            let (commands, inbox) = mpsc::channel();
            app.manage(Lightroom {
                status: Mutex::new(Status::default()),
                commands: Mutex::new(commands),
                preview: Mutex::new(SelectionPreview::default()),
            });
            app.manage(collections::CollectionsState::default());
            app.manage(tag::TagState::default());
            app.manage(Kept::default());
            let copy = std::env::current_exe()
                .map(|exe| update::Copy::locate(&exe))
                .unwrap_or(update::Copy::Development);
            log::info!("update: this copy is {copy:?}");
            if let Some(folder) = copy.folder()
                && std::env::args().any(|a| a == update::AFTER_UPDATE)
            {
                update::clean_up(folder);
            }
            app.manage(update::Updates {
                copy,
                phase: Mutex::new(update::Phase::default()),
                found: Mutex::new(None),
            });
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move { update::check_update(handle, true).await });
            // Writes are told to the UI from a thread of their own, in order:
            // never from under the lock that settings changes wait on.
            let (writing, told) = mpsc::channel::<bool>();
            app.manage(Writes::new(move |on| {
                let _ = writing.send(on);
            }));
            let handle = app.handle().clone();
            std::thread::Builder::new()
                .name("lightroom-writing".into())
                .spawn(move || {
                    for on in told {
                        let _ = handle.emit("lightroom-writing", on);
                    }
                })?;
            let handle = app.handle().clone();
            std::thread::Builder::new()
                .name("lightroom-link".into())
                .spawn(move || lightroom::events::lightroom_link(handle, inbox))?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running DiveLens");
}
