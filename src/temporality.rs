use opentelemetry_sdk::metrics::{data::Temporality, reader::TemporalitySelector, InstrumentKind};

#[derive(Debug)]
pub struct MyTemporalitySelector;

impl TemporalitySelector for MyTemporalitySelector {
    fn temporality(&self, kind: InstrumentKind) -> Temporality {
        match kind {
            InstrumentKind::UpDownCounter | InstrumentKind::ObservableUpDownCounter => {
                Temporality::Cumulative
            }
            InstrumentKind::Counter
            | InstrumentKind::ObservableCounter
            | InstrumentKind::ObservableGauge
            | InstrumentKind::Histogram => Temporality::Delta,
        }
    }
}
