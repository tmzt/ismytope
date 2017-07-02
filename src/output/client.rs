

use std::io;
use std::fmt;
use parser::ast::*;

mod format_html {
    use std::fmt::{self, Write};
    use itertools;
    use parser::ast::*;

    pub struct FormatHtml {
    }

    impl FormatHtml {
        pub fn new() -> FormatHtml {
            FormatHtml {}
        }

        pub fn write_js_expr_value(&self, w: &mut fmt::Write, node: &ExprValue, var_prefix: Option<&str>) -> fmt::Result {
            match node {
                // TODO: Handle the case where quotes appear in the string
                &ExprValue::LiteralString(ref s) => { write!(w, "\"{}\"", s)?; },
                &ExprValue::LiteralNumber(ref n) => { write!(w, "{}", n)?; },

                &ExprValue::VariableReference(ref s) => {
                    if let Some(ref prefix) = var_prefix {
                        write!(w, "{}{}", prefix, s)?;
                    } else {
                        write!(w, "{}", s)?;
                    }
                },

                &ExprValue::Expr(ref sym, ref l, ref r) => {
                    write!(w, "{:?} {:?} {:?}", l, sym, r)?;
                }
            }
            Ok(())
        }

        pub fn write_computed_expr_value(&self, w: &mut fmt::Write, node: &ExprValue, var_prefix: Option<&str>) -> fmt::Result {
            match node {
                &ExprValue::LiteralString(ref s) => { write!(w, "{}", s)?; },
                &ExprValue::LiteralNumber(ref n) => { write!(w, "{}", n)?; },

                &ExprValue::VariableReference(ref s) => {
                    if let Some(ref prefix) = var_prefix {
                        write!(w, "{}{}", prefix, s)?;
                    } else {
                        write!(w, "{}", s)?;
                    }
                },

                &ExprValue::Expr(ref sym, ref l, ref r) => {
                    write!(w, "{:?} {:?} {:?}", l, sym, r)?;
                }
            }
            Ok(())
        }

        #[allow(dead_code)]
        pub fn write_html_content(&self, w : &mut fmt::Write, node: &ContentNodeType) -> fmt::Result {
            // Write node
            match node {
                &ContentNodeType::ElementNode(ref element_data) => {
                    let element_tag = element_data.element_ty.to_lowercase();
                    let mut attrs_str = String::new();

                    if let Some(ref attrs) = element_data.attrs {
                        for &(ref key, ref expr) in attrs.iter() {
                            let mut expr_str = String::new();
                            self.write_computed_expr_value(&mut expr_str, &expr, None)?;
                            write!(attrs_str, " {}=\"{}\"", key, expr_str)?;
                        }
                    }

                    // For now, assume these are HTML nodes
                    write!(w, "<{}{}>",
                        element_tag,
                        attrs_str
                    )?;

                    if let Some(ref children) = element_data.children {
                        for ref child in children {
                            self.write_html_content(w, child)?;
                        }
                    }

                    write!(w, "</{}>", element_tag)?;
                },
                &ContentNodeType::ExpressionValueNode(ref expr) => {
                    let mut expr_str = String::new();
                    self.write_computed_expr_value(&mut expr_str, expr, None)?;
                    write!(w, "{}", expr_str)?;
                }
            }
            Ok(())
        }

        pub fn write_js_incdom_element(&self, w: &mut fmt::Write, element_data: &ElementType, var_prefix: Option<&str>) -> fmt::Result {
            let element_tag = element_data.element_ty.to_lowercase();
            // TODO:
            // Elements which have varying content should have a key allocated
            // otherwise we should be able to omit it in the IncrementalDOM calls.
            /*
            let element_key = element_data.element_key.as_ref().unwrap_or_else(|| allocate_element_key());
            */

            /*
            let mut element_key = String::new();
            if let Some(ref defined_key) = element_data.element_key {
                write!(element_key, "{}", defined_key)?;
            } else {
                write!(element_key, "{}", allocate_element_key())?;
            }
            */

            let mut attrs_str = String::new();

            // Collect (static) attrs first
            if let Some(ref attrs) = element_data.attrs {
                write!(attrs_str, "{}", itertools::join(attrs.iter().map(
                    |&(ref key, ref expr)| {
                        let mut expr_str = String::new();
                        self.write_computed_expr_value(&mut expr_str, &expr, None).expect("Could not write attribute value in DOM node.");
                        format!("\"{}\", \"{}\"", key, expr_str)
                    }
                ), ", "))?;
            }

            if let Some(ref children) = element_data.children {
                // Open element
                writeln!(w, "IncrementalDOM.elementOpen(\"{}\", null, [{}], []);",
                    element_tag,
                    attrs_str
                )?;

                // Output children
                for child in children {
                    self.write_js_incdom_content(w, child, var_prefix)?;
                }

                // Close element
                writeln!(w, "IncrementalDOM.elementClose(\"{}\");", element_tag)?;
            } else {
                // Void element
                writeln!(w, "IncrementalDOM.elementVoid(\"{}\", null, [{}], []);",
                    element_tag,
                    attrs_str
                )?;
            }
            Ok(())
        }

        pub fn write_js_incdom_content(&self, w: &mut fmt::Write, node: &ContentNodeType, var_prefix: Option<&str>) -> fmt::Result {
            match node {
                &ContentNodeType::ElementNode(ref element_data) => {
                    self.write_js_incdom_element(w, element_data, var_prefix)?;
                },
                &ContentNodeType::ExpressionValueNode(ref expr) => {
                    let mut expr_str = String::new();
                    self.write_js_expr_value(&mut expr_str, &expr, var_prefix)?;
                    writeln!(w, "IncrementalDOM.text({});", expr_str)?;
                }
            };
            Ok(())
        }

        pub fn write_js_function(&self, w: &mut fmt::Write, component_data: &ComponentDefinitionType) -> fmt::Result {
            writeln!(w, "function {}(props) {{", &component_data.name)?;
            if let Some(ref children) = component_data.children {
                for child in children.iter() {
                    match child {
                        &NodeType::ContentNode(ref content) => {
                            self.write_js_incdom_content(w, content, Some("props.".into()))?;
                        },
                        _ => {}
                    }
                }
            }
            writeln!(w, "}};")?;
            Ok(())
        }

        #[allow(dead_code)]
        pub fn write_html_document(&self, w : &mut fmt::Write, ast: &Template) -> fmt::Result {
            writeln!(w, "<!doctype HTML>")?;
            writeln!(w, "<html>")?;
            writeln!(w, "<head>")?;
            writeln!(w, "<script src=\"https://unpkg.com/redux@3.7.1/dist/redux.js\"></script>")?;
            writeln!(w, "<script src=\"https://ajax.googleapis.com/ajax/libs/incrementaldom/0.5.1/incremental-dom.js\" defer=\"defer\"></script>", )?;
            writeln!(w, "<script>", )?;

            writeln!(w, "function render(store) {{")?;
                // Define components
                for ref loc in ast.children.iter() {
                    match &loc.inner {
                        &NodeType::ComponentDefinitionNode(ref component_data) => {
                            self.write_js_function(w, component_data)?;
                        },
                        _ => {}
                    }
                }
                writeln!(w, "")?;
                writeln!(w, "")?;

                // Render content nodes

                for ref loc in ast.children.iter() {
                    match &loc.inner {
                        &NodeType::ContentNode(ref content) => {
                            self.write_js_incdom_content(w, content, Some("store.getState().".into()))?;
                        },
                        _ => {},
                    }
                }
            writeln!(w, "}}")?;

            writeln!(w, "function update(root_el, store) {{")?;
            writeln!(w, "  IncrementalDOM.patch(root_el, render.bind(null, store));")?;
            writeln!(w, "}}")?;

            writeln!(w, "function counterReducer(state, action) {{")?;
            writeln!(w, "  if ('undefined' !== typeof action && 'INCREMENT' == action.type) {{ return state + 1; }}")?;
            writeln!(w, "  return state || 0;")?;
            writeln!(w, "}}")?;

            writeln!(w, "document.addEventListener(\"DOMContentLoaded\", function(event) {{")?;
            writeln!(w, "  var root_el = document.querySelector(\"#root\");")?;
            writeln!(w, "  var rootReducer = Redux.combineReducers({{")?;
            writeln!(w, "    counter: counterReducer,")?;
            writeln!(w, "  }});")?;
            writeln!(w, "  var store = Redux.createStore(rootReducer, {{}});")?;
            writeln!(w, "  store.subscribe(function() {{ update(root_el, store); }});")?;
            writeln!(w, "  store.dispatch({{ type: \"START\" }});")?;

            writeln!(w, "  var increment_el = document.querySelector(\"a[href='#increment']\");")?;
            writeln!(w, "  increment_el.onclick = function() {{ store.dispatch({{ type: \"INCREMENT\" }}); }};")?;

            //writeln!(w, "  setTimeout(function() {{ update(root_el); }}, 0);")?;
            writeln!(w, "}});")?;

            writeln!(w, "</script>\n</head>")?;

            writeln!(w, "<body>")?;
            writeln!(w, "<div id=\"root\">")?;

            /*
            for ref loc in ast.children.iter() {
                match &loc.inner {
                    &NodeType::ContentNode(ref content) => {
                        self.write_html_content(w, &content)?
                    },
                    _ => {}
                }
            }
            */

            writeln!(w, "</div>")?;
            writeln!(w, "</body>")?;
            writeln!(w, "</html>")?;
            Ok(())
        }
    }
}

use self::format_html::FormatHtml;

pub type Result = io::Result<fmt::Result>;

pub struct ClientOutput {
}

impl ClientOutput {
    pub fn new() -> ClientOutput {
        ClientOutput {}
    }

    pub fn write_html(&self, w : &mut io::Write, ast: &Template) -> Result {
        let format = FormatHtml::new();

        let mut doc_str = String::new();
        if let Err(e) = format.write_html_document(&mut doc_str, ast) {
            return Ok(Err(e));
        }

        if let Err(e) = w.write_fmt(format_args!("{}", doc_str)) {
            return Err(e);
        }

        Ok(Ok(()))
    }
}