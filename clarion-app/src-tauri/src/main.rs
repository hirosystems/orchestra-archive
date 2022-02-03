#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use std::sync::mpsc::channel;

mod orchestra;

fn main() {

  let (frontend_cmd_tx, frontend_cmd_rx) = channel();
  let (backend_cmd_tx, backend_cmd_rx) = channel();

  std::thread::spawn(|| {
    orchestra::run_frontend(frontend_cmd_tx, backend_cmd_rx);
  });

  std::thread::spawn(|| {
    orchestra::mock_backend(backend_cmd_tx, frontend_cmd_rx);
    // orchestra::run_backend(backend_cmd_tx, frontend_cmd_rx);
  });

  let context = tauri::generate_context!("tauri.conf.json");
  let app = tauri::Builder::default().build(context).expect("Failed building");
  app.run(|handle, event| {});
}
