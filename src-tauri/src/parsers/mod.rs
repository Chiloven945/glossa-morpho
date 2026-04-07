//! Format adapter stubs for future importer implementations.
//!
//! Start with json.rs, yaml.rs and properties.rs.

use std::path::Path;

pub trait FormatAdapter {
    fn detect(&self, path: &Path, content: &[u8]) -> bool;
    fn import(&self) -> Result<(), String>;
    fn export(&self) -> Result<(), String>;
}
