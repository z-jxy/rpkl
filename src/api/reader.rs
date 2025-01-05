use std::io::Write;

use crate::utils::macros::_warn;

use super::{
    evaluator::responses::PklServerResponseRaw,
    external_reader::{
        outgoing::{
            ListModulesResponse, ListResourcesResponse, ReadModuleResponse, ReadResourceResponse,
        },
        PklModuleReader, PklResourceReader,
    },
    msgapi::PklMessage,
};

pub fn handle_list_resources<W: Write>(
    resource_readers: &[Box<dyn PklResourceReader>],
    msg: &PklServerResponseRaw,
    writer: &mut W,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = msg.response.as_map().unwrap();

    // TODO: could add `with-serde` feature to rmpv to make this easier
    // but might be overkill for messages with a small number of fields

    let evaluator_id: i64 = extract_field(response, "evaluatorId")?;
    let request_id: i64 = extract_field(response, "requestId")?;
    let uri: &str = extract_field(response, "uri")?;

    let uri_scheme = parse_scheme(uri).expect("Invalid URI, this is a bug");

    let Some(reader) = resource_readers.iter().find(|r| r.scheme() == uri_scheme) else {
        _warn!("No reader found for scheme: {:?}", uri);
        writer.write_all(
            &ListResourcesResponse {
                request_id,
                evaluator_id,
                path_elements: None,
                error: Some(format!("No reader found for scheme: {:?}", uri)),
            }
            .encode_msg()?,
        )?;
        writer.flush()?;
        return Ok(());
    };

    let data = reader.list(uri);

    let out_msg = match data {
        Ok(elements) => ListResourcesResponse {
            request_id,
            evaluator_id,
            path_elements: Some(elements),
            error: None,
        },
        Err(e) => ListResourcesResponse {
            request_id,
            evaluator_id,
            path_elements: None,
            error: Some(e.to_string()),
        },
    };

    writer.write_all(&out_msg.encode_msg()?)?;
    writer.flush()?;

    Ok(())
}

pub fn handle_list_modules<W: Write>(
    module_readers: &[Box<dyn PklModuleReader>],
    msg: &PklServerResponseRaw,
    writer: &mut W,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = msg.response.as_map().unwrap();

    // TODO: could add `with-serde` feature to rmpv to make this easier
    // but might be overkill for messages with a small number of fields

    let evaluator_id: i64 = extract_field(response, "evaluatorId")?;
    let request_id: i64 = extract_field(response, "requestId")?;
    let uri: &str = extract_field(response, "uri")?;

    let uri_scheme = parse_scheme(uri).expect("Invalid URI, this is a bug");

    let Some(reader) = module_readers.iter().find(|r| r.scheme() == uri_scheme) else {
        _warn!("No reader found for scheme: {:?}", uri);
        writer.write_all(
            &ListModulesResponse {
                request_id,
                evaluator_id,
                path_elements: None,
                error: Some(format!("No reader found for scheme: {:?}", uri)),
            }
            .encode_msg()?,
        )?;
        writer.flush()?;
        return Ok(());
    };

    let data = reader.list(uri);

    let out_msg = match data {
        Ok(elements) => ListModulesResponse {
            request_id,
            evaluator_id,
            path_elements: Some(elements),
            error: None,
        },
        Err(e) => ListModulesResponse {
            request_id,
            evaluator_id,
            path_elements: None,
            error: Some(e.to_string()),
        },
    };

    writer.write_all(&out_msg.encode_msg()?)?;
    writer.flush()?;

    Ok(())
}

pub fn handle_read_resource<W: Write>(
    resource_readers: &[Box<dyn PklResourceReader>],
    msg: &PklServerResponseRaw,
    writer: &mut W,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = msg.response.as_map().unwrap();

    let evaluator_id: i64 = extract_field(response, "evaluatorId")?;
    let request_id: i64 = extract_field(response, "requestId")?;
    let uri: &str = extract_field(response, "uri")?;

    let uri_scheme = parse_scheme(uri).expect("Invalid URI, this is a bug");

    let Some(reader) = resource_readers.iter().find(|r| r.scheme() == uri_scheme) else {
        _warn!("No reader found for scheme: {:?}", uri);
        writer.write_all(
            &ReadResourceResponse {
                request_id,
                evaluator_id,
                contents: None,
                error: Some(format!("No reader found for scheme: {:?}", uri)),
            }
            .encode_msg()?,
        )?;
        writer.flush()?;
        return Ok(());
    };

    let data = reader.read(uri);

    let out_msg = match data {
        Ok(data) => ReadResourceResponse {
            request_id,
            evaluator_id,
            contents: Some(data),
            error: None,
        },
        Err(e) => ReadResourceResponse {
            request_id,
            evaluator_id,
            contents: None,
            error: Some(e.to_string()),
        },
    };

    let serialized = out_msg.encode_msg()?;

    writer.write_all(&serialized)?;
    writer.flush()?;

    Ok(())
}

pub fn handle_read_module<W: Write>(
    module_readers: &[Box<dyn PklModuleReader>],
    msg: &PklServerResponseRaw,
    writer: &mut W,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = msg.response.as_map().unwrap();

    let evaluator_id: i64 = extract_field(response, "evaluatorId")?;
    let request_id: i64 = extract_field(response, "requestId")?;
    let uri: &str = extract_field(response, "uri")?;

    let uri_scheme = parse_scheme(uri).expect("Invalid URI, this is a bug");

    let Some(reader) = module_readers.iter().find(|r| r.scheme() == uri_scheme) else {
        _warn!("No reader found for scheme: {:?}", uri);
        writer.write_all(
            &ReadModuleResponse {
                request_id,
                evaluator_id,
                contents: None,
                error: Some(format!("No reader found for scheme: {:?}", uri)),
            }
            .encode_msg()?,
        )?;
        writer.flush()?;
        return Ok(());
    };

    let data = reader.read(uri);

    let out_msg = match data {
        Ok(data) => ReadModuleResponse {
            request_id,
            evaluator_id,
            contents: Some(data),
            error: None,
        },
        Err(e) => ReadModuleResponse {
            request_id,
            evaluator_id,
            contents: None,
            error: Some(e.to_string()),
        },
    };

    let serialized = out_msg.encode_msg()?;

    writer.write_all(&serialized)?;
    writer.flush()?;

    Ok(())
}

struct MapValue<'a>(&'a rmpv::Value);

impl<'a> TryFrom<MapValue<'a>> for i64 {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: MapValue<'a>) -> Result<Self, Self::Error> {
        match value.0 {
            rmpv::Value::Integer(n) => n.as_i64().ok_or_else(|| "Failed to convert to i64".into()),
            _ => Err("Expected integer value".into()),
        }
    }
}

impl<'a> TryFrom<MapValue<'a>> for &'a str {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: MapValue<'a>) -> Result<Self, Self::Error> {
        match value.0 {
            rmpv::Value::String(s) => s
                .as_str()
                .ok_or_else(|| "Failed to get str from string".into()),
            _ => Err("Expected string value".into()),
        }
    }
}

impl<'a> TryFrom<MapValue<'a>> for String {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: MapValue<'a>) -> Result<Self, Self::Error> {
        match value.0 {
            rmpv::Value::String(s) => Ok(s
                .as_str()
                .ok_or_else(|| "Failed to get str from string")?
                .to_owned()),
            _ => Err("Expected string value".into()),
        }
    }
}

fn parse_scheme(uri: &str) -> Option<&str> {
    match uri.find(':') {
        Some(pos) => {
            let scheme = &uri[..pos];
            if !scheme.is_empty()
                && scheme
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '.' || c == '-')
            {
                Some(scheme)
            } else {
                None
            }
        }
        None => None,
    }
}

// Helper function to extract fields from response map
fn extract_field<'a, T>(
    map: &'a [(rmpv::Value, rmpv::Value)],
    field: &str,
) -> Result<T, Box<dyn std::error::Error>>
where
    T: TryFrom<MapValue<'a>, Error = Box<dyn std::error::Error>>,
{
    map.iter()
        .find(|(k, _)| k.as_str() == Some(field))
        .map(|(_, v)| MapValue(v))
        .ok_or_else(|| format!("Field not found: {}", field).into())
        .and_then(|v| v.try_into())
}
