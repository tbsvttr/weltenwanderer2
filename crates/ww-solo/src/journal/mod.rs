//! Journaling system for recording solo session events.

pub mod entry;
pub mod log;

pub use entry::JournalEntry;
pub use log::Journal;
