mod aggregator;
mod resource;

use once_cell::sync::Lazy;
use opentelemetry_api::global;
// use opentelemetry_api::global::shutdown_tracer_provider;
// use opentelemetry_api::trace::TraceError;
use opentelemetry_api::{
    metrics,
    // trace::{TraceContextExt, Tracer},
    Context,
    KeyValue,
};
use opentelemetry_otlp::{WithExportConfig};
use opentelemetry_sdk::{metrics::MeterProvider, runtime, Resource};
use tonic::transport::{Certificate, Identity};
use tonic::{
    metadata::{MetadataKey, MetadataMap},
    transport::ClientTlsConfig,
};
use url::Url;
use std::{
    env::{remove_var, set_var, var, vars},
    error::Error,
    str::FromStr,
    time::Duration,
};

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

// Use the variables to try and export the example to any external collector that accepts otlp
// like: oltp itself, honeycomb or lightstep
const ENDPOINT: &str = "OTLP_TONIC_ENDPOINT";
const TLS_FILES: &str = "OTLP_TONIC_TLS_PATH";
const CA_DOMAIN: &str = "OTLP_TONIC_CA_DOMAIN";
const HEADER_PREFIX: &str = "OTLP_TONIC_";

fn init_metrics() -> metrics::Result<MeterProvider> {
    let endpoint = var(ENDPOINT).unwrap_or_else(|_| {
        panic!("You must specify and endpoint to connect to with the variable {ENDPOINT:?}.",)
    });
    let endpoint = Url::parse(&endpoint).expect("endpoint is not a valid url");
    remove_var(ENDPOINT);

    let mut metadata = MetadataMap::new();
    for (key, value) in vars()
        .filter(|(name, _)| name.starts_with(HEADER_PREFIX))
        .map(|(name, value)| {
            let header_name = name
                .strip_prefix(HEADER_PREFIX)
                .map(|h| h.replace('_', "-"))
                .map(|h| h.to_ascii_lowercase())
                .unwrap();
            (header_name, value)
        })
    {
        metadata.insert(MetadataKey::from_str(&key).unwrap(), value.parse().unwrap());
    }

    let ca_domain = var(CA_DOMAIN).unwrap_or_else(|_| {
        panic!("You must specify a ca DOMAIN to connect to with the variable {CA_DOMAIN:?}.",)
    });

    let tls_path = var(TLS_FILES).unwrap_or_else(|_| {
        panic!("You must specify a tls root path to connect to with the variable {TLS_FILES:?}.",)
    });
    let ca: Certificate;
    let tls_path= std::path::PathBuf::from(tls_path);
    let ca_file= tls_path.join("ca-cert.pem");
    println!("ca file: {:?}", ca_file);
    let pem = std::fs::read_to_string(ca_file);
    match pem {
        Ok(pem) => ca = Certificate::from_pem(pem),
        Err(err) => panic!("{err}"), 
    }

    let ident: Identity;
    let crt_pem;
    let key_pem;
    let crt_file= tls_path.join("client.crt");
    println!("{:?}", crt_file);
    let pem = std::fs::read_to_string(crt_file);
    match pem {
        Ok(pem) => crt_pem = pem,
        Err(err) => panic!("{err}"), 
    }

    let key_file= tls_path.join("client.key");
    println!("{:?}", key_file);
    let pem = std::fs::read_to_string(key_file);
    match pem {
        Ok(pem) => key_pem = pem,
        Err(err) => panic!("{err}"), 
    }

    ident = Identity::from_pem(crt_pem, key_pem);

    // let export_config = ExportConfig {
    //     endpoint: "http://localhost:4317".to_string(),
    //     ..ExportConfig::default()
    // };
    let kvps = vec![
        KeyValue::new(RESOURCE_KEY, MY_RESOURCE_NAME),
        KeyValue::new(INSTANCE_KEY, MY_INSTANCE_NAME),
    ];
    let kvps = resource_new(kvps, Some(MY_NAME_SPACE));

    opentelemetry_otlp::new_pipeline()
        .metrics(runtime::Tokio)
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(endpoint.as_str())
                .with_metadata(metadata)
                .with_tls_config(
                    ClientTlsConfig::new().domain_name(
                        endpoint
                            .host_str()
                            .expect("the specified endpoint should have a valid host"),
                    )
                    .ca_certificate(ca)
                    .domain_name(ca_domain)
                    .identity(ident),
                ),
        )
        // .with_period(Duration::from_secs(0))
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
    if let Err(std::env::VarError::NotPresent) = var("RUST_LOG") {
        set_var("RUST_LOG", "debug")
    };
    env_logger::init();
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

    let scounter = meter.u64_counter("some_counter")
        .with_description("sync_counter")
        .init();

    scounter.add(&cx, 100, COMMON_ATTRIBUTES.as_ref());
    println!("{:?}", scounter);

    gauge.observe(std::f64::consts::E, COMMON_ATTRIBUTES.as_ref());

    meter.register_callback(&[gauge.as_any()], move |observer| {
        println!("{:?}", gauge);
        observer.observe_f64(&gauge, std::f64::consts::E, COMMON_ATTRIBUTES.as_ref());
    })?;

    counter.observe(std::f64::consts::PI, COMMON_ATTRIBUTES.as_ref());

    meter.register_callback(&[counter.as_any()], move |observer| {
        println!("{:?}", counter);
        observer.observe_f64(&counter, std::f64::consts::PI, COMMON_ATTRIBUTES.as_ref());
    })?;

    let histogram = meter
        .f64_histogram("histogram")
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

    // meter_provider.force_flush(&cx)?;
    std::thread::sleep(Duration::from_millis(100));
    // shutdown_tracer_provider();
    meter_provider.force_flush(&cx)?;
    meter_provider.shutdown()?;

    Ok(())
}
