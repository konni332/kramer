use std::{fs, path::PathBuf};

use directories::ProjectDirs;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

fn log_dir() -> PathBuf {
    let proj = ProjectDirs::from("com", "kramer", "kramer")
        .expect("failed to resolve platform directories");

    #[cfg(target_os = "macos")]
    {
        proj.data_local_dir().to_path_buf().join("Logs")
    }

    #[cfg(target_os = "windows")]
    {
        proj.data_local_dir().to_path_buf().join("logs")
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        proj.state_dir()
            .unwrap_or_else(|| proj.data_local_dir())
            .to_path_buf()
            .join("logs")
    }
}

pub fn log_location() -> PathBuf {
    log_dir().join("kramer.log")
}

pub fn init() -> WorkerGuard {
    let dir = log_dir();
    fs::create_dir_all(&dir).expect("failed to create log directory");

    let file_appender = tracing_appender::rolling::daily(dir, "kramer.log");
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = fmt::layer()
        .with_writer(file_writer)
        .with_ansi(false)
        .with_target(true)
        .with_level(true);

    #[cfg(debug_assertions)]
    let stderr_layer = fmt::layer()
        .with_writer(std::io::stderr)
        .with_ansi(true)
        .with_target(true)
        .with_level(true);

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        // IMPORTANT:
        // crate targets must match where logs originate:
        EnvFilter::new("kramer=info,kramer_core=info")
    });

    let registry = tracing_subscriber::registry().with(filter).with(file_layer);

    #[cfg(debug_assertions)]
    {
        registry.with(stderr_layer).init();
    }

    #[cfg(not(debug_assertions))]
    {
        registry.init();
    }

    guard
}

pub fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let thread = std::thread::current();
        let name = thread.name().unwrap_or("unnamed");

        let location = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_else(|| "unknown".into());

        let payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
            *s
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.as_str()
        } else {
            "non-string panic payload"
        };

        tracing::error!(
            thread = name,
            location = %location,
            message = %payload,
            "panic occurred"
        );
    }));
}
