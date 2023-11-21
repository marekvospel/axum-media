use bytes::{buf::Writer, Bytes, BytesMut};

pub(crate) fn serialize_json<T: serde::Serialize>(
    data: &T,
    buf: &mut Writer<BytesMut>,
) -> Result<(), crate::AnyMediaSerializeError> {
    data.serialize(&mut serde_json::Serializer::new(buf))
        .map_err(|e| e.into())
}

#[cfg(feature = "urlencoded")]
pub(crate) fn serialize_urlencoded<T: serde::Serialize>(
    data: &T,
    buf: &mut Writer<BytesMut>,
) -> Result<(), crate::AnyMediaSerializeError> {
    use std::io::Write;

    serde_urlencoded::to_string(data)
        .map(|s| buf.write_all(s.as_bytes()).unwrap())
        .map_err(|e| e.into())
}

pub(crate) fn deserialize_json<T: serde::de::DeserializeOwned>(
    bytes: &Bytes,
) -> Result<T, crate::AnyMediaDeserializeError> {
    let deserializer = &mut serde_json::Deserializer::from_slice(bytes);

    serde_path_to_error::deserialize(deserializer).map_err(|e| e.into())
}

#[cfg(feature = "urlencoded")]
pub(crate) fn deserialize_urlencoded<T: serde::de::DeserializeOwned>(
    bytes: &Bytes,
) -> Result<T, crate::AnyMediaDeserializeError> {
    serde_urlencoded::from_bytes(bytes).map_err(|e| e.into())
}

#[cfg(feature = "yaml")]
pub(crate) fn serialize_yaml<T: serde::Serialize>(
    data: &T,
    buf: &mut Writer<BytesMut>,
) -> Result<(), crate::AnyMediaSerializeError> {
    serde_yaml::to_writer(buf, data).map_err(|e| e.into())
}

#[cfg(feature = "yaml")]
pub(crate) fn deserialize_yaml<T: serde::de::DeserializeOwned>(
    bytes: &Bytes,
) -> Result<T, crate::AnyMediaDeserializeError> {
    let deserializer = serde_yaml::Deserializer::from_slice(bytes);

    serde_path_to_error::deserialize(deserializer).map_err(|e| e.into())
}

#[cfg(feature = "xml")]
pub(crate) fn serialize_xml<T: serde::Serialize>(
    data: &T,
    buf: &mut Writer<BytesMut>,
) -> Result<(), crate::AnyMediaSerializeError> {
    serde_xml_rs::to_writer(buf, data).map_err(|e| e.into())
}
