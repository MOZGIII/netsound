use libpulse_binding::sample::Spec;
use libpulse_binding::stream::Direction;
use libpulse_simple_binding::Simple;

pub fn build_psimple(sample_spec: Spec, direction: Direction) -> Simple {
    assert!(sample_spec.is_valid());

    let stream_name = format!("netsound stream {:?}", direction);
    Simple::new(
        None,
        "netsound",
        direction,
        None,
        &stream_name,
        &sample_spec,
        None,
        None,
    )
    .unwrap()
}
