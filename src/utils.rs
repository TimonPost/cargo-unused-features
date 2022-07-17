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
        .filter_module("unused_features", log_level)
        .filter_module("cargo::core", LevelFilter::Error)
        .init();
}
