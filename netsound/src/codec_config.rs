#[derive(Debug)]
pub enum CodecToUse {
    Opus,
    Raw,
}

impl std::str::FromStr for CodecToUse {
    type Err = anyhow::Error;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        Ok(match name {
            "opus" => CodecToUse::Opus,
            "raw" => CodecToUse::Raw,
            name => return Err(anyhow::format_err!("codec {:?} is not available", name)),
        })
    }
}
