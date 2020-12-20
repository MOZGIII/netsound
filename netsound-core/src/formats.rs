use super::format::Format;

static FORMATS: &[Format<f32>] = &[
    Format::new(2, 48000),
    Format::new(1, 48000),
    Format::new(2, 24000),
    Format::new(1, 24000),
    Format::new(2, 16000),
    Format::new(1, 16000),
    Format::new(2, 12000),
    Format::new(1, 12000),
    Format::new(2, 8000),
    Format::new(1, 8000),
];

pub fn input() -> &'static [Format<f32>] {
    FORMATS
}

pub fn output() -> &'static [Format<f32>] {
    FORMATS
}
