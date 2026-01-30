fn main() {
    tracing_subscriber::fmt::init();

    if let Ok(sessions_dir) = pi_session_manager::scanner::get_sessions_dir() {
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
            rt.block_on(async {
                pi_session_manager::scanner_scheduler::ScannerScheduler::new(sessions_dir, 30)
                    .start()
                    .await;
            });
        });
    }

    pi_session_manager::run()
}