use std::ops::Deref;

use crate::internal::ObjectMember;

#[derive(Debug)]
pub struct StructNode<'a> {
    pub _pkl_ident: &'a str, // The Pkl identifier for this struct
    pub members: &'a [ObjectMember],
    pub is_dependency: bool,
    pub parent_module_name: &'a str,
    pub pub_struct: bool,
}

#[derive(Debug)]
pub struct EnumNode<'a> {
    _pkl_ident: &'a str, // The Pkl identifier for this enum
    _variants: Vec<String>,
    _is_dependency: bool,
}

#[derive(Debug)]
pub enum NodeType<'a> {
    Struct(StructNode<'a>),
    Enum(EnumNode<'a>),
    // You might add other node types if needed, like for opaque values, though
    // opaque values might not need their own nodes if they don't have dependencies.
}

impl NodeType<'_> {
    pub fn parent_module_name(&self) -> Option<&str> {
        match self {
            NodeType::Enum { .. } => None,

            NodeType::Struct(StructNode {
                parent_module_name, ..
            }) => Some(parent_module_name),
        }
    }

    // pub fn is_dependency(&self) -> bool {
    //     match self {
    //         NodeType::Struct(StructNode { is_dependency, .. })
    //         | NodeType::Enum(EnumNode { is_dependency, .. }) => *is_dependency,
    //     }
    // }

    // pub fn pkl_ident(&self) -> &str {
    //     match &self {
    //         NodeType::Struct(StructNode { pkl_ident, .. })
    //         | NodeType::Enum(EnumNode { pkl_ident, .. }) => pkl_ident,
    //     }
    // }
}

#[derive(Debug)]
pub struct Node<'a> {
    pub name: String, // The Rust-friendly name (e.g., "Example", "AnonMap", "Mode")
    pub node_type: NodeType<'a>,
}

impl<'a> Deref for Node<'a> {
    type Target = NodeType<'a>;

    fn deref(&self) -> &Self::Target {
        &self.node_type
    }
}

impl Node<'_> {
    pub fn name(&self) -> &str {
        &self.name
    }

    // #[inline]
    // pub fn pkl_ident(&self) -> &str {
    //     self.node_type.pkl_ident()
    // }

    // #[inline]
    // pub fn is_dependency(&self) -> bool {
    //     self.node_type.is_dependency()
    // }
}
