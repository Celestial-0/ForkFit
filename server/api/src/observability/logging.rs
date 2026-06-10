use tracing_subscriber::{prelude::*, EnvFilter, Registry};

pub fn init() {
    // 1. Initialize OpenTelemetry tracing
    let tracer = super::tracing::init_tracing();

    // 2. Configure log level filter
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    // 3. Configure trace telemetry layer
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // 4. Configure output format (JSON vs Text)
    let log_format = std::env::var("LOG_FORMAT").unwrap_or_else(|_| "text".to_string());

    if log_format.eq_ignore_ascii_case("json") {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .json()
            .with_target(true)
            .with_thread_ids(true)
            .with_line_number(true);
            
        Registry::default()
            .with(env_filter)
            .with(telemetry_layer)
            .with(fmt_layer)
            .try_init()
            .ok();
    } else {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_target(true);
            
        Registry::default()
            .with(env_filter)
            .with(telemetry_layer)
            .with(fmt_layer)
            .try_init()
            .ok();
    }
}
