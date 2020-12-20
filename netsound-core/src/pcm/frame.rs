//! A PCM [`Frame`] definition.

use super::Sample;

/// A PCM [`Frame`].
///
/// Note: rust implementation for const generics is currently incomplete, and
/// it substantially decreses the use cases of the `Frame` type; conversions
/// are difficult, and overall user experience is terrible.
pub trait Frame: dasp_frame::Frame + Unpin + Send
where
    <Self as dasp_frame::Frame>::Sample: Sample,
{
}
