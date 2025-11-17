//! ACPI device tree

use alloc::string::String;
use alloc::vec::Vec;

/// ACPI device tree node
pub struct AcpiNode {
    pub name: String,
    pub device_id: u64,
    pub children: Vec<AcpiNode>,
}

/// Parse ACPI device tree
pub fn parse_acpi_tree() -> Option<AcpiNode> {
    // Parse ACPI tables
    // Build device tree
    None
}

