use crate::internal::ObjectMember;

#[derive(Debug)]
pub struct PklMod {
    pub(crate) module_name: String,
    pub(crate) module_uri: String,
    pub(crate) members: Vec<ObjectMember>,
}

impl PklMod {
    pub fn module_name(&self) -> &str {
        &self.module_name
    }

    pub fn module_uri(&self) -> &str {
        &self.module_uri
    }
}
