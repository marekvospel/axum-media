use bytes::{buf::Writer, BytesMut};

pub(crate) fn serialize_json<T: serde::Serialize>(
    data: &T,
    buf: &mut Writer<BytesMut>,
) -> Result<(), crate::AnyMediaError> {
    data.serialize(&mut serde_json::Serializer::new(buf))
        .map_err(|e| e.into())
}

#[cfg(feature = "urlencoded")]
pub(crate) fn serialize_urlencoded(
    data: &impl serde::Serialize,
    buf: &mut Writer<BytesMut>,
) -> Result<(), crate::AnyMediaError> {
    use std::io::Write;

    serde_urlencoded::to_string(data)
        .map(|s| buf.write_all(s.as_bytes()).unwrap())
        .map_err(|e| e.into())
}
