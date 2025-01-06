use serde::Serialize;

pub trait PklMessage {
    fn message_id() -> u64;

    /// Encode/serialize the message to a byte vector.
    /// Used to ensure the proper message ID/code is associated with each message
    fn encode_msg(&self) -> Result<Vec<u8>, rmp_serde::encode::Error>
    where
        Self: Serialize,
    {
        let mut serialized = Vec::new();
        (Self::message_id(), self).serialize(
            &mut rmp_serde::Serializer::new(&mut serialized)
                .with_struct_map()
                .with_bytes(rmp_serde::config::BytesMode::ForceAll),
        )?;

        Ok(serialized)
    }
}

pub(crate) mod macros {
    macro_rules! impl_pkl_message {
        ($type:ident<$($lt:lifetime),+>, $id:expr) => {
            impl<$($lt),+> $crate::api::msgapi::PklMessage for $type<$($lt),+> {
                fn message_id() -> u64 {
                    $id
                }
            }
        };

        ($type:ty, $id:expr) => {
            impl $crate::api::msgapi::PklMessage for $type {
                fn message_id() -> u64 {
                    $id
                }
            }
        };
    }

    pub(crate) use impl_pkl_message;
}
