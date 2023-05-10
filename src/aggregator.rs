use opentelemetry_sdk::metrics::{reader::AggregationSelector, Aggregation, InstrumentKind};

#[derive(Debug)]
pub struct MyAggregationSelector;

// const HISTOGRAM_AGGREGATION_KEY: &str = "[histogram]";

impl AggregationSelector for MyAggregationSelector {
    fn aggregation(&self, kind: InstrumentKind) -> Aggregation {
        match kind {
            InstrumentKind::Counter
            | InstrumentKind::UpDownCounter
            | InstrumentKind::ObservableCounter
            | InstrumentKind::ObservableUpDownCounter => Aggregation::Sum,
            InstrumentKind::ObservableGauge => Aggregation::LastValue,
            InstrumentKind::Histogram => Aggregation::ExplicitBucketHistogram {
                boundaries: vec![
                    0.0, 5.0, 10.0, 25.0, 50.0, 75.0, 100.0, 250.0, 500.0, 750.0, 1000.0, 2500.0,
                    5000.0, 7500.0, 10000.0,
                ],
                record_min_max: true,
            },
        }
    }
}
