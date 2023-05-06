use opentelemetry_api::KeyValue;

const NAME_SPACE_KEY: &str = "namespace";
pub fn resource_new(mut kvps: Vec<KeyValue>, namespace: Option<&'static str>) -> Vec<KeyValue> {
    if let Some(namespace) = namespace {
        kvps.push(KeyValue::new(NAME_SPACE_KEY, namespace));
    }

    kvps
}
