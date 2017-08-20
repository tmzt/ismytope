#![allow(dead_code)]

use parser::ast::*;
use parser::util::allocate_element_key;


#[derive(Debug, Clone)]
pub struct Symbols {
    map_id: String,
    parent_map_id: Option<String>,
    symbol_map: SymbolMap
}

impl Default for Symbols {
    fn default() -> Symbols {
        Symbols::new(None)
    }
}

impl Symbols {
    pub fn new(parent_map_id: Option<&str>) -> Symbols {
        Symbols {
            map_id: allocate_element_key(),
            parent_map_id: parent_map_id.map(|s| s.to_owned()),
            symbol_map: Default::default()
        }
    }

    pub fn map_id(&self) -> &str {
        &self.map_id
    }

    pub fn parent_map_id(&self) -> Option<&str> {
        self.parent_map_id.as_ref().map(|s| s.as_str())
    }

    pub fn add_sym(&mut self, key: &str, sym: Symbol) {
        self.symbol_map.insert(key.to_owned(), sym);
    }

    pub fn get_sym(&self, key: &str) -> Option<&Symbol> {
        self.symbol_map.get(key)
    }
}

#[cfg(test)]
mod tests {
    use parser::ast::*;
    use scope::symbols::*;

    #[test]
    pub fn test_symbols() {
        let mut symbols = Symbols::new(None);
        symbols.add_sym("abc", Symbol::prop("xyz", "s1"));
        // assert!(match symbols.get_sym("abc").map(|s| s.sym_ref()) { Some(&SymbolReferenceType::ResolvedReference("abc".to_owned(), ResolvedSymbolType::PropReference("abc".to_owned()), _)) => true, _ => false);

        assert_eq!(symbols.get_sym("abc").map(|s| s.sym_ref()), Some(&SymbolReferenceType::ResolvedReference("xyz".to_owned(), ResolvedSymbolType::PropReference("xyz".to_owned()), Some("s1".to_owned()))));
    }
}