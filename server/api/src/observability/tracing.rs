use opentelemetry::global;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::trace::{TracerProvider, Tracer};

pub fn init_tracing() -> Tracer {
    let provider = TracerProvider::builder().build();
    let tracer = provider.tracer("api");
    global::set_tracer_provider(provider);
    tracer
}
