pub mod access;
pub mod artifact;
pub mod document;
pub mod download;
pub mod errors;
pub mod feed;
pub mod fetch;
pub mod html;
pub mod identity;
pub mod statutory;
pub mod timely;

pub use access::{ensure_attachment_size, ensure_policy_allowed, JpPolitenessPolicy};
pub use fetch::JpOfficialAdapter;
pub use statutory::statutory_adapter;
pub use timely::timely_adapter;
