#[allow(dead_code)]
pub mod de;
#[allow(dead_code)]
pub mod en;

#[cfg(feature = "de")]
pub use de as lang;

#[cfg(not(feature = "de"))]
pub use en as lang;
