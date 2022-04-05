#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use std::sync::mpsc::channel;
use tauri::RunEvent;

mod orchestra;

fn main() {
  let (frontend_cmd_tx, frontend_cmd_rx) = channel();
  let (backend_cmd_tx, backend_cmd_rx) = channel();
  let (clock_cmd_tx, clock_cmd_rx) = channel();

  std::thread::spawn(|| {
    orchestra::run_clock(clock_cmd_rx);
  });

  let moved_clock_cmd_tx = clock_cmd_tx.clone();
  std::thread::spawn(|| {
    orchestra::run_frontend(frontend_cmd_tx, backend_cmd_rx);
  });

  std::thread::spawn(|| {
    orchestra::run_backend(backend_cmd_tx, frontend_cmd_rx, moved_clock_cmd_tx);
    // orchestra::mock_backend(backend_cmd_tx, frontend_cmd_rx);
  });

  let context = tauri::generate_context!("tauri.conf.json");
  let app = tauri::Builder::default()
    .build(context)
    .expect("Failed building");
  app.run(move |_handle, event| {
    match event {
      RunEvent::ExitRequested { window_label, api, .. } => {
        api.prevent_exit();
        let _ = clock_cmd_tx.send(orchestra::MiningCommand::Terminate);
        std::thread::spawn(|| {
          std::thread::sleep(std::time::Duration::from_secs(5));
          std::process::exit(0)
        });
      }
      _ => {}
    }
  });
}
