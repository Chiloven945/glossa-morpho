pub mod commands {
    pub mod entry;
    pub mod project;
    pub mod treemap;
}

mod errors;
mod models;
mod state;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::project::bootstrap_workspace,
            commands::project::create_project,
            commands::project::open_project,
            commands::project::save_project,
            commands::entry::update_entry,
            commands::entry::bulk_replace,
            commands::treemap::build_treemap
        ])
        .run(tauri::generate_context!())
        .expect("failed to run glossa-morpho")
}
