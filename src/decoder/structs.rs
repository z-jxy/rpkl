use crate::{
    pkl::internal::PklNonPrimitive,
    utils,
    value::{datasize::DataSizeUnit, DataSize},
    Error,
};

#[inline]
pub fn decode_datasize(type_id: u64, slots: &[rmpv::Value]) -> PklNonPrimitive {
    let float = slots[0].as_f64().expect("expected float for data size");
    let size_unit = slots[1].as_str().expect("expected size type");
    let ds = DataSize::new(float, DataSizeUnit::from(size_unit));
    PklNonPrimitive::DataSize(type_id, ds)
}

#[inline]
pub fn decode_intseq(type_id: u64, slots: &[rmpv::Value]) -> PklNonPrimitive {
    // nothing is done with 'step' slot of the int seq structure from pkl
    let start = slots[0].as_i64().expect("expected start for int seq");
    let end = slots[1].as_i64().expect("expected end for int seq");
    PklNonPrimitive::IntSeq(type_id, start, end)
}

pub fn decode_duration(type_id: u64, slots: &[rmpv::Value]) -> Result<PklNonPrimitive, Error> {
    // need u64 to convert to Duration
    let float_time = slots[0].as_f64().expect("expected float for duration") as u64;
    let duration_unit = slots[1].as_str().expect("expected time type");
    let duration = match duration_unit {
        "min" => {
            let Some(d) = utils::duration::from_mins(float_time) else {
                return Err(Error::DecodeError(format!(
                    "failed to parse duration from mins: {float_time}"
                )));
            };
            d
        }
        "h" => {
            let Some(d) = utils::duration::from_hours(float_time) else {
                return Err(Error::DecodeError(format!(
                    "failed to parse duration from hours: {float_time}"
                )));
            };
            d
        }
        "d" => {
            let Some(d) = utils::duration::from_days(float_time) else {
                return Err(Error::DecodeError(format!(
                    "failed to parse duration from days: {float_time}"
                )));
            };
            d
        }
        "ns" => std::time::Duration::from_nanos(float_time),
        "us" => std::time::Duration::from_micros(float_time),
        "ms" => std::time::Duration::from_millis(float_time),
        "s" => std::time::Duration::from_secs(float_time),
        _ => {
            return Err(Error::DecodeError(format!(
                "unsupported duration_unit, got {duration_unit:?}"
            )));
        }
    };
    Ok(PklNonPrimitive::Duration(type_id, duration))
}

#[inline]
pub fn decode_regex(type_id: u64, slots: &[rmpv::Value]) -> PklNonPrimitive {
    let pattern = slots[0].as_str().expect("expected pattern for regex");
    PklNonPrimitive::Regex(type_id, pattern.to_string())
}
