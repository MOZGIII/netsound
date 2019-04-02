use super::psimple::Simple;
use super::pulse::sample;
use super::pulse::stream::Direction;

pub fn build_psimple(direction: Direction) -> Simple {
    let spec = sample::Spec {
        format: sample::SAMPLE_FLOAT32,
        channels: 2,
        rate: 44100,
    };
    assert!(spec.is_valid());

    let stream_name = format!("netsound stream {:?}", direction);
    Simple::new(
        None,
        "netsound",
        direction,
        None,
        &stream_name,
        &spec,
        None,
        None,
    )
    .unwrap()
}
