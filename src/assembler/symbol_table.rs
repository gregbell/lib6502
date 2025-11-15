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
    pub fn add_symbol(
        &mut self,
        name: String,
        address: u16,
        defined_at: usize,
    ) -> Result<(), Symbol> {
        // Check for duplicates
        if let Some(existing) = self.lookup_symbol(&name) {
            return Err(existing.clone());
        }

        self.symbols.push(Symbol {
            name,
            address,
            defined_at,
        });

        Ok(())
    }

    /// Look up a symbol by name
    pub fn lookup_symbol(&self, name: &str) -> Option<&Symbol> {
        self.symbols.iter().find(|s| s.name == name)
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

        assert!(table.add_symbol("START".to_string(), 0x8000, 1).is_ok());
        assert!(table.add_symbol("LOOP".to_string(), 0x8010, 5).is_ok());

        let start = table.lookup_symbol("START").unwrap();
        assert_eq!(start.name, "START");
        assert_eq!(start.address, 0x8000);

        let loop_sym = table.lookup_symbol("LOOP").unwrap();
        assert_eq!(loop_sym.address, 0x8010);

        assert!(table.lookup_symbol("UNDEFINED").is_none());
    }

    #[test]
    fn test_symbol_table_duplicate() {
        let mut table = SymbolTable::new();

        assert!(table.add_symbol("START".to_string(), 0x8000, 1).is_ok());
        let result = table.add_symbol("START".to_string(), 0x9000, 10);
        assert!(result.is_err());

        // Original symbol should still be there
        let start = table.lookup_symbol("START").unwrap();
        assert_eq!(start.address, 0x8000);
    }
}
