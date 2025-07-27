pub mod type_constants {
    pub const TYPED_DYNAMIC: u64 = 1;
    pub const MAP: u64 = 2;
    pub const MAPPING: u64 = 3;
    pub const LIST: u64 = 4;
    pub const LISTING: u64 = 5;
    pub const SET: u64 = 6;
    pub const DURATION: u64 = 7;
    pub const DATA_SIZE: u64 = 8;
    pub const PAIR: u64 = 9;
    pub const INT_SEQ: u64 = 10;
    pub const REGEX: u64 = 11;
    pub const _CLASS: u64 = 12;
    pub const TYPE_ALIAS: u64 = 13;
    pub const BYTES: u64 = 15; // 0xF (pkl 0.29)
    pub const OBJECT_MEMBER: u64 = 16;
    /// collections example at <https://pkl-lang.org/main/current/language-tutorial/01_basic_config.html#collections> returns 18 for the listing
    pub const DYNAMIC_MAPPING: u64 = 17;
    pub const DYNAMIC_LISTING: u64 = 18;

    #[allow(dead_code)]
    pub(crate) fn pkl_type_id_str(type_id: u64) -> &'static str {
        match type_id {
            TYPED_DYNAMIC => "TypedDynamic",
            MAP => "Map",
            MAPPING => "Mapping",
            LIST => "List",
            LISTING => "Listing",
            SET => "Set",
            DURATION => "Duration",
            DATA_SIZE => "DataSize",
            PAIR => "Pair",
            INT_SEQ => "IntSeq",
            REGEX => "Regex",
            _CLASS => "Class",
            TYPE_ALIAS => "TypeAlias",
            OBJECT_MEMBER => "ObjectMember",
            DYNAMIC_MAPPING => "DynamicMapping",
            DYNAMIC_LISTING => "DynamicListing",
            _ => "UNKNOWN",
        }
    }
}
