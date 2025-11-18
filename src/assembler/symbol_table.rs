//! Symbol table management for labels

use crate::assembler::Symbol;

/// Symbol table for managing label definitions
#[derive(Debug, Clone)]
pub struct SymbolTable {
    symbols: Vec<Symbol>,
}

impl SymbolTable {
    /// Create a new empty symbol table
    pub fn new() -> Self {
        Self {
            symbols: Vec::new(),
        }
    }

    /// Add a symbol to the table
    ///
    /// Returns Ok(()) on success, Err with duplicate symbol if name already exists
    ///
    /// Takes &str and only allocates if insertion succeeds (optimization)
    pub fn add_symbol(
        &mut self,
        name: &str,
        value: u16,
        kind: crate::assembler::SymbolKind,
        defined_at: usize,
    ) -> Result<(), Symbol> {
        // Check for duplicates
        if let Some(existing) = self.lookup_symbol(name) {
            return Err(existing.clone());
        }

        // Only allocate String if we're actually inserting
        self.symbols.push(Symbol {
            name: name.to_string(),
            value,
            kind,
            defined_at,
        });

        Ok(())
    }

    /// Look up a symbol by name (case-sensitive)
    pub fn lookup_symbol(&self, name: &str) -> Option<&Symbol> {
        self.symbols.iter().find(|s| s.name == name)
    }

    /// Look up a symbol by name (case-insensitive, no allocation)
    ///
    /// This is more efficient than uppercasing the query and calling lookup_symbol
    /// because it avoids allocating a new String.
    pub fn lookup_symbol_ignore_case(&self, name: &str) -> Option<&Symbol> {
        self.symbols
            .iter()
            .find(|s| s.name.eq_ignore_ascii_case(name))
    }

    /// Get all symbols
    pub fn symbols(&self) -> &[Symbol] {
        &self.symbols
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_table_add_lookup() {
        let mut table = SymbolTable::new();

        assert!(table
            .add_symbol("START", 0x8000, crate::assembler::SymbolKind::Label, 1)
            .is_ok());
        assert!(table
            .add_symbol("LOOP", 0x8010, crate::assembler::SymbolKind::Label, 5)
            .is_ok());

        let start = table.lookup_symbol("START").unwrap();
        assert_eq!(start.name, "START");
        assert_eq!(start.value, 0x8000);

        let loop_sym = table.lookup_symbol("LOOP").unwrap();
        assert_eq!(loop_sym.value, 0x8010);

        assert!(table.lookup_symbol("UNDEFINED").is_none());
    }

    #[test]
    fn test_symbol_table_duplicate() {
        let mut table = SymbolTable::new();

        assert!(table
            .add_symbol("START", 0x8000, crate::assembler::SymbolKind::Label, 1)
            .is_ok());
        let result = table.add_symbol("START", 0x9000, crate::assembler::SymbolKind::Label, 10);
        assert!(result.is_err());

        // Original symbol should still be there
        let start = table.lookup_symbol("START").unwrap();
        assert_eq!(start.value, 0x8000);
    }

    // T035: Unit test for adding constant to table
    #[test]
    fn test_add_constant() {
        let mut table = SymbolTable::new();

        assert!(table
            .add_symbol("MAX", 255, crate::assembler::SymbolKind::Constant, 1)
            .is_ok());

        let constant = table.lookup_symbol("MAX").unwrap();
        assert_eq!(constant.name, "MAX");
        assert_eq!(constant.value, 255);
        assert_eq!(constant.kind, crate::assembler::SymbolKind::Constant);
        assert_eq!(constant.defined_at, 1);
    }

    // T036: Unit test for adding label to table
    #[test]
    fn test_add_label() {
        let mut table = SymbolTable::new();

        assert!(table
            .add_symbol("START", 0x8000, crate::assembler::SymbolKind::Label, 5)
            .is_ok());

        let label = table.lookup_symbol("START").unwrap();
        assert_eq!(label.name, "START");
        assert_eq!(label.value, 0x8000);
        assert_eq!(label.kind, crate::assembler::SymbolKind::Label);
        assert_eq!(label.defined_at, 5);
    }

    // T037: Unit test for lookup returning correct kind
    #[test]
    fn test_lookup_returns_correct_kind() {
        let mut table = SymbolTable::new();

        // Add a constant
        table
            .add_symbol("MAX", 255, crate::assembler::SymbolKind::Constant, 1)
            .unwrap();

        // Add a label
        table
            .add_symbol("LOOP", 0x1000, crate::assembler::SymbolKind::Label, 10)
            .unwrap();

        // Verify constant lookup
        let max = table.lookup_symbol("MAX").unwrap();
        assert_eq!(max.kind, crate::assembler::SymbolKind::Constant);
        assert_eq!(max.value, 255);

        // Verify label lookup
        let loop_label = table.lookup_symbol("LOOP").unwrap();
        assert_eq!(loop_label.kind, crate::assembler::SymbolKind::Label);
        assert_eq!(loop_label.value, 0x1000);
    }
}
