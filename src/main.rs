mod resource;
mod aggregator;

use once_cell::sync::Lazy;
use opentelemetry_api::global;
// use opentelemetry_api::global::shutdown_tracer_provider;
// use opentelemetry_api::trace::TraceError;
use opentelemetry_api::{
    metrics,
    // trace::{TraceContextExt, Tracer},
    Context, KeyValue,
};
use opentelemetry_otlp::{ExportConfig, WithExportConfig};
use opentelemetry_sdk::{metrics::MeterProvider, runtime, Resource};
use std::error::Error;
use std::time::Duration;

use crate::aggregator::MyAggregationSelector;
use crate::resource::resource_new;

// fn init_tracer() -> Result<sdktrace::Tracer, TraceError> {
//     opentelemetry_otlp::new_pipeline()
//         .tracing()
//         .with_exporter(
//             opentelemetry_otlp::new_exporter()
//                 .tonic()
//                 .with_endpoint("http://localhost:4317"),
//         )
//         .with_trace_config(
//             sdktrace::config().with_resource(Resource::new(vec![KeyValue::new(
//                 opentelemetry_semantic_conventions::resource::SERVICE_NAME,
//                 "trace-demo",
//             )])),
//         )
//         .install_batch(runtime::Tokio)
// }

const MY_NAME_SPACE: &str = "MyNameSpace";
const RESOURCE_KEY: &str = "resource";
const INSTANCE_KEY: &str = "instance";
const MY_RESOURCE_NAME: &str = "MyResource";
const MY_INSTANCE_NAME: &str = "MyInstance";

fn init_metrics() -> metrics::Result<MeterProvider> {
    let export_config = ExportConfig {
        endpoint: "http://localhost:4317".to_string(),
        ..ExportConfig::default()
    };
    let kvps = vec![KeyValue::new(RESOURCE_KEY, MY_RESOURCE_NAME), KeyValue::new(INSTANCE_KEY,MY_INSTANCE_NAME)];
    let kvps = resource_new(kvps, Some(MY_NAME_SPACE));

    opentelemetry_otlp::new_pipeline()
        .metrics(runtime::Tokio)
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_export_config(export_config),
        )
        .with_period(Duration::from_secs(1))
        .with_aggregation_selector(MyAggregationSelector)
        .with_resource(Resource::new(kvps))
        .build()
}

// const LEMONS_KEY: Key = Key::from_static_str("lemons");
// const ANOTHER_KEY: Key = Key::from_static_str("ex.com/another");

static COMMON_ATTRIBUTES: Lazy<[KeyValue; 1]> = Lazy::new(|| {
    [
        // LEMONS_KEY.i64(10),
        KeyValue::new("A", "1"),
        // KeyValue::new("B", "2"),
        // KeyValue::new("C", "3"),
    ]
});

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    // By binding the result to an unused variable, the lifetime of the variable
    // matches the containing block, reporting traces and metrics during the whole
    // execution.
    // let _ = init_tracer()?;
    let meter_provider = init_metrics()?;
    let cx = Context::new();

    // let tracer = global::tracer("ex.com/basic");
    let meter = global::meter("ex.com/basic");

    let gauge = meter
        .f64_observable_gauge("gauge")
        .with_description("A gauge set to e")
        .init();

    let counter = meter
        .f64_observable_counter("counter")
        .with_description("A counter set to pi")
        .init();

    meter.register_callback(&[gauge.as_any()], move |observer| {
        println!("{:?}", gauge);
        observer.observe_f64(&gauge, std::f64::consts::E, COMMON_ATTRIBUTES.as_ref());
    })?;

    meter.register_callback(&[counter.as_any()], move |observer| {
        println!("{:?}", counter);
        observer.observe_f64(&counter, std::f64::consts::PI, COMMON_ATTRIBUTES.as_ref());
    })?;

    let histogram = meter.f64_histogram("histogram")
        .with_unit(metrics::Unit::new("cm"))
        .with_description("Some HSTG")
        .init();
    histogram.record(&cx, 5.5, COMMON_ATTRIBUTES.as_ref());

    // tracer.in_span("operation", |cx| {
    //     let span = cx.span();
    //     span.add_event(
    //         "Nice operation!".to_string(),
    //         vec![Key::new("bogons").i64(100)],
    //     );
    //     span.set_attribute(ANOTHER_KEY.string("yes"));

    //     tracer.in_span("Sub operation...", |cx| {
    //         let span = cx.span();
    //         span.set_attribute(LEMONS_KEY.string("five"));

    //         span.add_event("Sub span event", vec![]);

    //         histogram.record(&cx, 1.3, &[]);
    //     });
    // });

    std::thread::sleep(Duration::from_secs(60));
    // shutdown_tracer_provider();
    meter_provider.force_flush(&cx)?;
    meter_provider.shutdown()?;

    Ok(())
}
