use env_logger::Env;

fn main() {
    let log_level = std::env::var("MY_LOG_LEVEL").unwrap_or_else(|_| "Debug".to_string());
    let log_style = std::env::var("MY_LOG_STYLE").unwrap_or_else(|_| "always".to_string());

    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", &log_level)
        .write_style_or("MY_LOG_STYLE", &log_style);
    env_logger::init_from_env(env);

    log::info!(
        "Logging initialized. MY_LOG_LEVEL: {}, MY_LOG_STYLE: {}",
        log_level,
        log_style
    );

    vn_vttrpg::init().expect("Failed to initialize!");
}
