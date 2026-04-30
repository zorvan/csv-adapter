//! Rights management pages.

pub mod consume;
pub mod create;
pub mod journey;
pub mod list;
pub mod show;
pub mod transfer;

pub use consume::ConsumeRight;
pub use create::CreateRight;
pub use journey::RightJourney;
pub use list::Rights;
pub use show::ShowRight;
pub use transfer::TransferRight;
