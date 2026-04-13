pub mod commands {
    pub mod entry;
    pub mod export;
    pub mod import;
    pub mod project;
    pub mod treemap;
}

mod editing;
mod errors;
mod exporters;
mod models;
mod parsers;
mod project_format;
mod state;
mod utils;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::project::bootstrap_workspace,
            commands::project::create_project,
            commands::project::open_project,
            commands::project::save_project,
            commands::project::save_project_as,
            commands::project::update_project_metadata,
            commands::project::create_resource_file,
            commands::project::rename_resource_file,
            commands::project::delete_resource_file,
            commands::entry::create_entry,
            commands::entry::delete_entry,
            commands::entry::delete_entries,
            commands::entry::update_entry,
            commands::entry::bulk_replace,
            commands::import::preview_import,
            commands::import::commit_import,
            commands::export::export_project,
            commands::export::export_project_batch,
            commands::treemap::build_treemap
        ])
        .run(tauri::generate_context!())
        .expect("failed to run glossa-morpho")
}
