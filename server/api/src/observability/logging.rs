pub fn init() {
    tracing_subscriber::fmt().try_init().ok();
}
