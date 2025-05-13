mod primitive;
mod structs;

use crate::{
    context::Context,
    pkl::{internal::ObjectMember, PklMod},
    Error, Result,
};

/// Decode a pkl module from a messagepack value
pub(crate) fn decode_module(decoded: &rmpv::Value) -> Result<crate::pkl::PklMod> {
    let root = decoded
        .as_array()
        .context("expected root array for pkl module")?;
    let module_name = root
        .get(1)
        .context("expected root level module name")?
        .as_str()
        .context("PklMod name is not valid utf8")?;
    let module_uri = root
        .get(2)
        .context("expected root level module uri")?
        .as_str()
        .context("PklMod uri is not valid utf8")?;

    let pkl_module = root
        .last()
        .context("expected children in pkl module")?
        .as_array()
        .context("expected array of children")?;

    // parse the members of the module
    let members = pkl_module
        .iter()
        .map(|value| {
            let Some(member_data) = value.as_array() else {
                return Err(Error::Message(
                    "expected array for pkl module member".to_string(),
                ));
            };
            structs::decode_object_member(member_data)
                .map_err(|e| Error::Message(format!("failed to parse pkl object member: {e}")))
        })
        .collect::<Result<Vec<ObjectMember>>>()?;

    Ok(PklMod {
        _module_name: module_name.to_string(),
        _module_uri: module_uri.to_string(),
        members,
    })
}
