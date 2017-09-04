pub mod expr_writers;
pub mod ops_writers;
pub mod stream_writers;
pub mod block_writers;

pub use self::expr_writers::*;
pub use self::ops_writers::*;
pub use self::stream_writers::*;
pub use self::block_writers::*;

use std::io;
use parser::*;
use scope::*;
use processing::*;


pub trait OutputWriter {
    type E: ExpressionWriter;
}

#[derive(Debug, Default)]
pub struct DefaultOutputWriter<E: ExpressionWriter> {
    value_writer: E::V,
    expression_writer: E,
    events: EventsVec
}

impl<V: ValueWriter, E: ExpressionWriter<V = V>> OutputWriter for DefaultOutputWriter<E> {
    type E = E;
}

impl<E: ExpressionWriter> ExprWriter for DefaultOutputWriter<E> {
    type E = E;

    fn write_expr(&mut self, w: &mut io::Write, ctx: &mut Context, bindings: &BindingContext, expr: &ExprValue) -> Result {
        self.expression_writer.write_expr_to(w, &mut self.value_writer, ctx, bindings, expr)
    }
}

// impl<E: ExpressionWriter> DefaultOutputWriter<E> {
//     fn add_event(&mut self, ctx: &mut Context, bindings: &BindingContext, element_key: &str, event_handler: &EventHandler) -> Result {
//         let complete_key = ctx.join_path_with(Some("."), element_key);
//         let event = event_handler.create_event(&complete_key, ctx.scope().id());
//         self.events.push(event);

//         Ok(())
//     }

//     pub fn events_iter<'a>(&'a self) -> Option<impl IntoIterator<Item = &'a EventsItem>> {
//         let len = self.events.len();
//         if len > 0 { Some(self.events.iter()) } else { None}
//     }
// }

#[derive(Debug, Default)]
pub struct DefaultOutputWriters {}

pub type DefaultOutputWriterHtml = DefaultOutputWriter<ExpressionWriterHtml>;
pub type DefaultOutputWriterJs = DefaultOutputWriter<ExpressionWriterJs>;

pub trait OutputWritersBoth {
    type Html: OutputWriter;
    type Js: OutputWriter;

    fn html(&mut self) -> &mut DefaultOutputWriterHtml;
    fn js(&mut self) -> &mut DefaultOutputWriterJs;
}

#[derive(Debug, Default)]
pub struct DefaultOutputWritersBoth {
    pub html: DefaultOutputWriterHtml,
    pub js: DefaultOutputWriterJs
}

impl OutputWritersBoth for DefaultOutputWritersBoth {
    type Html = DefaultOutputWriterHtml;
    type Js = DefaultOutputWriterJs;

    fn html(&mut self) -> &mut DefaultOutputWriterHtml { &mut self.html }
    fn js(&mut self) -> &mut DefaultOutputWriterJs { &mut self.js }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::str;
    use scope::context::*;
    use scope::bindings::*;
    use processing::structs::ElementOp;


    fn create_document<'a>(template: &'a Template) -> Document {
        let mut ctx = Context::default();
        let mut bindings = BindingContext::default();
        let mut processing = ProcessDocument::from_template(&template);
        assert!(processing.process_document(&mut ctx, &mut bindings).is_ok());
        processing.into()
    }

    #[test]
    pub fn test_output_default_writers() {
        let mut ctx = Context::default();
        // ctx.append_path_str("Ab");
        let bindings = BindingContext::default();

        let op_1 = ElementOp::ElementOpen("span".into(), "Ab.Cd".into(), None, None, None);
        let op_2 = ElementOp::ElementClose("span".into());

        let template = Template::new(vec![]);
        let doc = create_document(&template);
        let mut writers = DefaultOutputWritersBoth::default();

        {
            let writer = writers.html();
            let mut s: Vec<u8> = Default::default();
            assert!(
                writer.write_element_op(&mut s, &doc, &mut ctx, &bindings, &op_1).is_ok() &&
                writer.write_element_op(&mut s, &doc, &mut ctx, &bindings, &op_2).is_ok()
            );
            assert_eq!(str::from_utf8(&s), Ok(indoc![r#"
            <span key="Ab.Cd"></span>"#
            ]));
        }

        {
            let writer = writers.html();
            let mut s: Vec<u8> = Default::default();
            let ops = vec![op_1.clone(), op_2.clone()];
            assert!(writer.write_element_ops(&mut s, &doc, &mut ctx, &bindings, ops.iter()).is_ok());
            assert_eq!(str::from_utf8(&s), Ok(indoc![r#"
            <span key="Ab.Cd"></span>"#
            ]));
        }

        {
            let writer = writers.js();
            let mut s: Vec<u8> = Default::default();
            assert!(
                writer.write_element_op(&mut s, &doc, &mut ctx, &bindings, &op_1).is_ok() &&
                writer.write_element_op(&mut s, &doc, &mut ctx, &bindings, &op_2).is_ok()
            );
            assert_eq!(str::from_utf8(&s), Ok(indoc![r#"
                IncrementalDOM.elementOpen("span", "Ab.Cd");
                IncrementalDOM.elementClose("span");
            "#]));
        }

        {
            let writer = writers.js();
            let mut s: Vec<u8> = Default::default();
            let ops = vec![op_1.clone(), op_2.clone()];
            assert!(writer.write_element_ops(&mut s, &doc, &mut ctx, &bindings, ops.iter()).is_ok());
            assert_eq!(str::from_utf8(&s), Ok(indoc![r#"
                IncrementalDOM.elementOpen("span", "Ab.Cd");
                IncrementalDOM.elementClose("span");
            "#]));
        }
    }

    #[test]
    pub fn test_output_default_writers_both() {
        let mut ctx = Context::default();
        let template = Template::new(vec![]);
        let doc = create_document(&template);
        // ctx.append_path_str("Ab");
        let bindings = BindingContext::default();

        let op_1 = ElementOp::ElementOpen("span".into(), "Ab.Cd".into(), None, None, None);
        let op_2 = ElementOp::ElementClose("span".into());

        {
            let mut writers = DefaultOutputWritersBoth::default();
            let mut s_html: Vec<u8> = Default::default();
            let mut s_js: Vec<u8> = Default::default();
            let ops = vec![op_1.clone(), op_2.clone()];
            assert!(writers.html().write_element_ops(&mut s_html, &doc, &mut ctx, &bindings, ops.iter()).is_ok());
            assert!(writers.js().write_element_ops(&mut s_js, &doc, &mut ctx, &bindings, ops.iter()).is_ok());

            assert_eq!(str::from_utf8(&s_html), Ok(indoc![r#"
            <span key="Ab.Cd"></span>"#
            ]));

            assert_eq!(str::from_utf8(&s_js), Ok(indoc![r#"
                IncrementalDOM.elementOpen("span", "Ab.Cd");
                IncrementalDOM.elementClose("span");
            "#]));
        }
    }
}