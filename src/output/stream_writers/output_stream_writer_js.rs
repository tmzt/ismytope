
use std::io;
use std::iter;
use std::slice::Iter;

use parser::ast::*;
use processing::structs::*;
use scope::scope::*;
use scope::context::*;
use scope::bindings::*;
use output::stream_writers::output_writer::*;
use output::stream_writers::output_stream_writer::*;


// pub type PropIterator = IntoIter<Item = Prop>;
// pub type EventHandlerIterator = IntoIter<Item = EventHandler>;
// pub type BindingIterator = IntoIterator<Item = ElementValueBinding>;

#[derive(Debug, Clone, Default)]
pub struct ElementOpsStreamWriterJs {}

impl ElementOpsStreamWriter for ElementOpsStreamWriterJs {
    fn write_op_element_open<PropIter, EventIter, BindingIter>(&mut self, w: &mut io::Write, expr_writer: &mut ExprWriter, value_writer: &mut ValueWriter, ctx: &mut Context, bindings: &BindingContext, element_tag: &str, element_key: &str, is_void: bool, props: PropIter, events: EventIter, binding: BindingIter) -> Result
        where PropIter : IntoIterator<Item = Prop>, EventIter: IntoIterator<Item = EventHandler>, BindingIter: IntoIterator<Item = ElementValueBinding>
    {
        if !is_void {
            write!(w, "IncrementalDOM.elementOpen(\"{}\", ", element_tag)?;
        } else {
            write!(w, "IncrementalDOM.elementVoid(\"{}\", ", element_tag)?;
        };

        let path_expr = ctx.scope_ref().unwrap().join_path_as_expr(Some("."));
        expr_writer.write_expr(w, value_writer, ctx, bindings, &path_expr)?;

        // write_js_expr_value(w, scope, path_expr)?;
        // write_js_func_params(scope, w)?;
        writeln!(w, ");")?;

        Ok(())
    }

    fn write_op_element_close(&mut self, w: &mut io::Write, expr_writer: &mut ExprWriter, value_writer: &mut ValueWriter, ctx: &mut Context, bindings: &BindingContext, element_tag: &str, element_key: &str) -> Result {
        writeln!(w, "IncrementalDOM.elementClose(\"{}\");", element_tag)?;
        Ok(())
    }

    fn write_op_element_start_block<PropIter: IntoIterator<Item = Prop>>(&mut self, w: &mut io::Write, expr_writer: &mut ExprWriter, value_writer: &mut ValueWriter, ctx: &mut Context, bindings: &BindingContext, block_id: &str, props: PropIter) -> Result {
        Ok(())
    }

    fn write_op_element_end_block(&mut self, w: &mut io::Write, expr_writer: &mut ExprWriter, value_writer: &mut ValueWriter, ctx: &mut Context, bindings: &BindingContext, block_id: &str) -> Result {
        Ok(())
    }

    fn write_op_element_map_collection_to_block(&mut self, w: &mut io::Write, expr_writer: &mut ExprWriter, value_writer: &mut ValueWriter, ctx: &mut Context, bindings: &BindingContext, coll_expr: &ExprValue, block_id: &str) -> Result {
        write!(w, "(")?;
        let binding = BindingType::LoopIndexBinding;
        writeln!(w, ").forEach(__{});", block_id)?;
        Ok(())
    }

    fn write_op_element_instance_component<PropIter, EventIter, BindingIter>(&mut self, w: &mut io::Write, expr_writer: &mut ExprWriter, value_writer: &mut ValueWriter, ctx: &mut Context, bindings: &BindingContext, element_tag: &str, element_key: &str, is_void: bool, props: PropIter, events: EventIter, binding: BindingIter) -> Result
        where PropIter : IntoIterator<Item = Prop>, EventIter: IntoIterator<Item = EventHandler>, BindingIter: IntoIterator<Item = ElementValueBinding>
    {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str;
    use std::iter::empty;
    use scope::context::*;
    use scope::bindings::*;
    use output::stream_writers::output_writer_js::*;


    #[test]
    pub fn test_output_stream_writers_js_ops1() {
        // let scope = Scope::default();
        let mut ctx = Context::default();
        let bindings = BindingContext::default();
        let mut value_writer = ValueWriterJs::default();
        let mut expr_writer = ExpressionWriterJs::default();

        let mut stream_writer = ElementOpsStreamWriterJs::default();

        let mut s: Vec<u8> = Default::default();
        let key = "key".to_owned();
        assert!(
            stream_writer.write_op_element_open(&mut s, &mut expr_writer, &mut value_writer, &mut ctx, &bindings, "span", &key, false, empty(), empty(), empty()).is_ok() &&
            stream_writer.write_op_element_close(&mut s, &mut expr_writer, &mut value_writer, &mut ctx, &bindings, "span", &key).is_ok()
        );
        assert_eq!(str::from_utf8(&s), Ok("IncrementalDOM.elementOpen(\"span\", ().join(\".\"));\nIncrementalDOM.elementClose(\"span\");\n".into()));
    }
}