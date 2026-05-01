//! Contract management pages.

pub mod add;
// pub mod deploy;  // Removed: deployment requires native SDKs (tokio/mio)
//                  which don't compile to WASM. Use csv-cli for deployment.
pub mod list;
pub mod modal;
pub mod status;

pub use add::AddContract;
// pub use deploy::DeployContract;  // Removed - see above
pub use list::Contracts;
pub use modal::ContractDetailModal;
pub use status::ContractStatus;
