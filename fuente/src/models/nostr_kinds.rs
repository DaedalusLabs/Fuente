// UNENCRYPTED STATIC KINDS - MEANT FOR LOCAL STORAGE
pub const NOSTR_KIND_CONSUMER_PROFILE: u32 = 8991;
pub const NOSTR_KIND_CONSUMER_PROFILE_ADDRESS: u32 = 8992;
pub const NOSTR_KIND_CONSUMER_ORDER_REQUEST: u32 = 8993;
pub const NOSTR_KIND_COMMERCE_ORDER_CONFIRMATION: u32 = 8994;
pub const NOSTR_KIND_DRIVER_PROFILE: u32 = 8995;
pub const NOSTR_KIND_PRESIGNED_URL_REQ: u32 = 9995;

// PUBLIC REPLEACEABLE - ONLY ONE CAN EXIST
pub const NOSTR_KIND_COMMERCE_PROFILE: u32 = 18990;
pub const NOSTR_KIND_COMMERCE_PRODUCTS: u32 = 18991;
pub const NOSTR_KIND_COURIER_PROFILE: u32 = 18992;

// ENCRYPTED STATIC KINDS - MEANT FOR STORING ON RELAYS
pub const NOSTR_KIND_CONSUMER_GIFTWRAP: u32 = 8992;
// PARAMETERIZED KINDS - Only one per d-tag
pub const NOSTR_KIND_ORDER_STATE: u32 = 38996;
pub const NOSTR_KIND_CONSUMER_REPLACEABLE_GIFTWRAP: u32 = 38992;
pub const NOSTR_KIND_SERVER_CONFIG: u32 = 38993;
pub const NOSTR_KIND_CONSUMER_REGISTRY: u32 = 38994;

// Ephemeral kinds - ARE NOT STORED AND MUST BE LIVE TO RECEIVE
pub const NOSTR_KIND_SERVER_REQUEST: u32 = 28990;
pub const NOSTR_KIND_DRIVER_STATE: u32 = 28991;
pub const NOSTR_KIND_ADMIN_REQUEST: u32 = 28992;
pub const NOSTR_KIND_PRESIGNED_URL_RESP: u32 = 29996;
