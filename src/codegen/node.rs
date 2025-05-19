use crate::internal::ObjectMember;

#[derive(Debug, Clone)]
pub struct StructNodeRef<'a> {
    pub _pkl_ident: &'a str,
    pub members: &'a [ObjectMember],
    pub is_dependency: bool,
    pub parent_module_name: &'a str,
    pub pub_struct: bool,
}
