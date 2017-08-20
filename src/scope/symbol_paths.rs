#![allow(dead_code)]

use parser::ast::*;


#[derive(Debug, Clone, PartialEq)]
pub enum SymbolPathComponent {
    StaticPathComponent(String),
    EvalPathComponent(ExprValue)
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SymbolPathScope(Option<Vec<SymbolPathComponent>>, Option<Symbol>);

impl SymbolPathScope {
    #[inline]
    pub fn append_expr(&mut self, expr: &ExprValue) {
        let comp = SymbolPathComponent::EvalPathComponent(expr.to_owned());
        if let Some(ref mut v) = self.0 {
            v.push(comp);
        } else {
            self.0 = Some(vec![comp]);
        };
    }

    #[inline]
    pub fn append_str(&mut self, s: &str) {
        let comp = SymbolPathComponent::StaticPathComponent(s.to_owned());
        if let Some(ref mut v) = self.0 {
            v.push(comp);
        } else {
            self.0 = Some(vec![comp]);
        };
    }

    #[inline]
    pub fn join_as_expr(&self, s: Option<&str>) -> Option<ExprValue> {
        self.0.as_ref().map(|symbol_path| {
            let expr_components: Vec<Box<ExprValue>> = symbol_path.iter()
                .map(|component| match component {
                    &SymbolPathComponent::StaticPathComponent(ref s) => Box::new(ExprValue::LiteralString(s.to_owned())),
                    &SymbolPathComponent::EvalPathComponent(ref expr) => Box::new(expr.to_owned())
                }).collect();

            let components = if expr_components.len() > 0 { Some(expr_components) } else { None };
            let join_opt = s.map(|s| s.to_owned());
            ExprValue::Apply(ExprApplyOp::JoinString(join_opt), components)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parser::ast::*;

    #[test]
    pub fn test_symbol_path_empty() {
        let symbol_path_scope = SymbolPathScope::default();
        let expr = symbol_path_scope.join_as_expr(None);
        assert_eq!(expr, Some(ExprValue::Apply(ExprApplyOp::JoinString(None), None)))
    }

    #[test]
    pub fn test_symbol_path_expr1() {
        let mut symbol_path_scope = SymbolPathScope::default();
        let expr1 = ExprValue::Expr(ExprOp::Add, Box::new(ExprValue::LiteralNumber(1)), Box::new(ExprValue::LiteralNumber(2)));
        let expr2 = ExprValue::LiteralString("test".to_owned());

        symbol_path_scope.append_expr(&expr1);
        symbol_path_scope.append_expr(&expr2);

        let expr = symbol_path_scope.join_as_expr(None);
        assert_eq!(expr, Some(ExprValue::Apply(ExprApplyOp::JoinString(None), Some(vec![
            Box::new(ExprValue::Expr(ExprOp::Add, Box::new(ExprValue::LiteralNumber(1)), Box::new(ExprValue::LiteralNumber(2)))),
            Box::new(ExprValue::LiteralString("test".to_owned()))
        ]))));
    }

    #[test]
    pub fn test_symbol_path_mixed1() {
        let mut symbol_path_scope = SymbolPathScope::default();
        let expr1 = ExprValue::Expr(ExprOp::Add, Box::new(ExprValue::LiteralNumber(1)), Box::new(ExprValue::LiteralNumber(2)));

        symbol_path_scope.append_expr(&expr1);
        symbol_path_scope.append_str("test");

        let expr = symbol_path_scope.join_as_expr(None);
        assert_eq!(expr, Some(ExprValue::Apply(ExprApplyOp::JoinString(None), Some(vec![
            Box::new(ExprValue::Expr(ExprOp::Add, Box::new(ExprValue::LiteralNumber(1)), Box::new(ExprValue::LiteralNumber(2)))),
            Box::new(ExprValue::LiteralString("test".to_owned()))
        ]))));
    }
}