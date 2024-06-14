pub mod type_constants {
    pub const OBJECT_MEMBER: u64 = 16;
    pub const TYPED_DYNAMIC: u64 = 1;
    pub const MAP: u64 = 2;
    pub const MAPPING: u64 = 3;
    pub const LIST: u64 = 4;
    pub const LISTING: u64 = 5;
    /// collections example at https://pkl-lang.org/main/current/language-tutorial/01_basic_config.html#collections returns 18 for the listing
    pub const DYNAMIC_LISTING: u64 = 18;
    pub const SET: u64 = 6;
    pub const DURATION: u64 = 7;
    pub const DATA_SIZE: u64 = 8;
    pub const PAIR: u64 = 9;
    pub const INT_SEQ: u64 = 10;
    pub const REGEX: u64 = 11;
    pub const CLASS: u64 = 12;
    pub const TYPE_ALIAS: u64 = 13;
    pub const PKL_LIST: u64 = 5;
}