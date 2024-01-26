use std::fmt;

use serde::{Deserialize, Serialize};

/// Parts of the UK with shared bank holiday dates.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Division {
    /// England and Wales
    EnglandAndWales,
    /// Scotland
    Scotland,
    /// Northern Ireland
    NorthernIreland,
}

impl Division {
    /// Iterator over all known divisions.
    #[inline]
    pub fn all() -> [Division; 3] {
        [
            Division::EnglandAndWales,
            Division::Scotland,
            Division::NorthernIreland,
        ]
    }

    /// English name of division.
    pub fn name(self) -> &'static str {
        match self {
            Division::EnglandAndWales => "England and Wales",
            Division::Scotland => "Scotland",
            Division::NorthernIreland => "Northern Ireland",
        }
    }
}

impl fmt::Display for Division {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}
