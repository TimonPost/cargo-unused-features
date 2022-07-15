use log::LevelFilter;

pub fn initialize_logger(log_level: Option<String>) {
    let log_level = match log_level {
        Some(log_level) => match log_level.as_str() {
            "trace" => LevelFilter::Trace,
            "debug" => LevelFilter::Debug,
            "info" => LevelFilter::Info,
            "warn" => LevelFilter::Warn,
            "error" => LevelFilter::Error,
            "off" => LevelFilter::Off,
            _ => LevelFilter::Info,
        },
        None => LevelFilter::Info,
    };

    env_logger::builder()
        .filter_module("cargo::core", LevelFilter::Error)
        .filter_module("cargo_prune_features", log_level)
        .init();
}
