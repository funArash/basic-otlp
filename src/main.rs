mod aggregator;
mod resource;

use once_cell::sync::Lazy;
use opentelemetry_api::global;
use opentelemetry_api::{metrics, Context, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{metrics::MeterProvider, runtime, Resource};
use std::{
    env::{remove_var, set_var, var, vars},
    error::Error,
    str::FromStr,
    time::Duration,
};
use tonic::transport::{Certificate, Identity};
use tonic::{
    metadata::{MetadataKey, MetadataMap},
    transport::ClientTlsConfig,
};
use url::Url;

use crate::aggregator::MyAggregationSelector;
use crate::resource::resource_new;

const MY_NAME_SPACE: &str = "MyNameSpace";
const RESOURCE_KEY: &str = "resource";
const INSTANCE_KEY: &str = "instance";
const MY_RESOURCE_NAME: &str = "MyResource";
const MY_INSTANCE_NAME: &str = "MyInstance";

// Use the variables to try and export the example to any external collector that accepts otlp
// like: oltp itself, honeycomb or lightstep
const ENDPOINT: &str = "OTLP_TONIC_ENDPOINT";
const TLS_PATH: &str = "OTLP_TONIC_TLS_PATH";
const TLS_CA: &str = "OTLP_TONIC_TLS_CA";
const TLS_CLIENT_CERT: &str = "OTLP_TONIC_TLS_CLIENT_CERT";
const TLS_CLIENT_KEY: &str = "OTLP_TONIC_TLS_CLIENT_KEY";
const CA_DOMAIN: &str = "OTLP_TONIC_CA_DOMAIN";
const HEADER_PREFIX: &str = "OTLP_TONIC_";

fn init_metrics() -> metrics::Result<MeterProvider> {
    let endpoint = var(ENDPOINT).unwrap_or_else(|_| "https://localhost:4317".to_string());
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

    let ca_domain = var(CA_DOMAIN).unwrap_or_else(|_| "localhost".to_string());

    let tls_path = var(TLS_PATH).unwrap_or_else(|_| var("PWD").unwrap());

    let tls_ca = var(TLS_CA).unwrap_or_else(|_| "inter.cert".to_string());

    let tls_client_cert = var(TLS_CLIENT_CERT).unwrap_or_else(|_| "client.cert".to_string());

    let tls_client_key = var(TLS_CLIENT_KEY).unwrap_or_else(|_| "client.key".to_string());

    let tls_path = std::path::PathBuf::from(tls_path);
    let ca_file = tls_path.join(tls_ca);
    println!("ca file: {:?}", ca_file);
    let pem = std::fs::read_to_string(ca_file);
    let ca: Certificate = match pem {
        Ok(pem) => Certificate::from_pem(pem),
        Err(err) => panic!("{err}"),
    };

    let crt_file = tls_path.join(tls_client_cert);
    println!("{:?}", crt_file);
    let pem = std::fs::read_to_string(crt_file);
    let crt_pem = match pem {
        Ok(pem) => pem,
        Err(err) => panic!("{err}"),
    };

    let key_file = tls_path.join(tls_client_key);
    println!("{:?}", key_file);
    let pem = std::fs::read_to_string(key_file);
    let key_pem = match pem {
        Ok(pem) => pem,
        Err(err) => panic!("{err}"),
    };

    let ident: Identity = Identity::from_pem(crt_pem, key_pem);

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
                    ClientTlsConfig::new()
                        .domain_name(
                            endpoint
                                .host_str()
                                .expect("the specified endpoint should have a valid host"),
                        )
                        .domain_name(ca_domain)
                        .ca_certificate(ca)
                        .identity(ident),
                ),
        )
        // .with_period(Duration::from_secs(0))
        .with_aggregation_selector(MyAggregationSelector)
        .with_resource(Resource::new(kvps))
        .build()
}

static COMMON_ATTRIBUTES: Lazy<[KeyValue; 1]> = Lazy::new(|| [KeyValue::new("A", "1")]);

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    // By binding the result to an unused variable, the lifetime of the variable
    // matches the containing block, reporting traces and metrics during the whole
    // execution.
    if let Err(std::env::VarError::NotPresent) = var("RUST_LOG") {
        set_var("RUST_LOG", "debug")
    };
    env_logger::init();
    let meter_provider = init_metrics()?;
    let cx = Context::new();

    let meter = global::meter("ex.com/basic");

    let gauge = meter
        .f64_observable_gauge("gauge")
        .with_description("A gauge set to e")
        .init();

    let counter = meter
        .f64_observable_counter("counter")
        .with_description("A counter set to pi")
        .init();

    let scounter = meter
        .u64_counter("some_counter")
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

    std::thread::sleep(Duration::from_millis(100));
    meter_provider.force_flush(&cx)?;
    meter_provider.shutdown()?;

    Ok(())
}
