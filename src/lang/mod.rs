#[cfg(all(feature = "en", feature = "de"))]
compile_error!("feature \"en\" and feature \"de\" cannot be enabled at the same time");

#[allow(dead_code)]
pub mod de;
#[allow(dead_code)]
pub mod en;

#[cfg(feature = "de")]
pub use de as lang;
#[cfg(feature = "en")]
pub use en as lang;
