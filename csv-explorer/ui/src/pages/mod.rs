/// Page components module.

pub mod home;
pub mod stats;
pub mod rights;
pub mod transfers;
pub mod seals;
pub mod wallet;

// Re-export all page components for use in routing
pub use home::Home;
pub use stats::Stats;
pub use rights::RightsList;
pub use transfers::TransfersList;
pub use seals::SealsList;
pub use wallet::Wallet;
