use std::io;
use std::fmt::Debug;

use error::*;
use ast::*;
use objects::*;
use output::*;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct DefaultJsWriter {}

impl ObjectWriter<Primitive, JsOutput> for DefaultJsWriter {
    fn write_object(
        &mut self,
        w: &mut io::Write,
        _ctx: &mut OutputContext,
        obj: &Primitive,
    ) -> DocumentProcessingResult<()> {
        match *obj {
            Primitive::Int32Val(n) => write!(w, "{}", n),

            Primitive::BoolVal(b) if b => write!(w, "true"),
            Primitive::BoolVal(_) => write!(w, "false"),

            Primitive::CharVal(c) => write!(w, "'{}'", c),
            Primitive::StringVal(ref s) => write!(w, "\"{}\"", s),
            Primitive::NullVal => write!(w, "null"),
            Primitive::Undefined => write!(w, "undefined"),
        }?;

        Ok(())
    }
}

impl ObjectWriter<LensValue<ProcessedExpression>, JsOutput> for DefaultJsWriter {
    fn write_object(
        &mut self,
        w: &mut io::Write,
        ctx: &mut OutputContext,
        obj: &LensValue<ProcessedExpression>,
    ) -> DocumentProcessingResult<()> {
        debug!(
            "ObjectWriter LensValue<ProcessedExpression> (JS): obj: {:?}",
            obj
        );

        match *obj {
            LensValue::ForLens(..) => Ok(()),
            LensValue::GetLens(ref _alias, box ref value, _) => {
                self.write_object(w, ctx, value)?;
                Ok(())
            }

            LensValue::QueryLens(ref item_key, ref query_call, _) => {
                let name = query_call.name();
                write!(w, "/* query lens: {:?} in {} */", item_key, name)?;
                write!(w, "[]")?;

                Ok(())
            }
        }
    }
}

impl ObjectWriter<ExpressionValue<ProcessedExpression>, JsOutput> for DefaultJsWriter {
    fn write_object(
        &mut self,
        w: &mut io::Write,
        ctx: &mut OutputContext,
        obj: &ExpressionValue<ProcessedExpression>,
    ) -> DocumentProcessingResult<()> {
        match *obj {
            ExpressionValue::Expression(ref e) => self.write_object(w, ctx, e),
            ExpressionValue::Primitive(ref p) => self.write_object(w, ctx, p),
            ExpressionValue::Binding(ref b, _) => self.write_object(w, ctx, b),
            ExpressionValue::BindingShape(ref s, _) => self.write_object(w, ctx, s.binding()),
            ExpressionValue::Lens(ref l, _) => self.write_object(w, ctx, l),
            ExpressionValue::SourceLens(_, _) => Ok(()),
            ExpressionValue::Content(_, _) => Ok(()), // _ => Err(reduction_err_bt!())
        }
    }
}

impl ObjectWriter<ExpressionValue<OutputExpression>, JsOutput> for DefaultJsWriter {
    fn write_object(
        &mut self,
        w: &mut io::Write,
        ctx: &mut OutputContext,
        obj: &ExpressionValue<OutputExpression>,
    ) -> DocumentProcessingResult<()> {
        match *obj {
            ExpressionValue::Expression(ref e) => self.write_object(w, ctx, e),
            ExpressionValue::Primitive(ref p) => self.write_object(w, ctx, p),
            ExpressionValue::Binding(ref b, _) => self.write_object(w, ctx, b),
            _ => Err(try_process_from_err!(format!(
                "Unsupported output expression: {:?}",
                obj
            ))),
        }
    }
}

impl<T: Clone + Debug> ObjectWriter<CommonBindings<T>, JsOutput> for DefaultJsWriter {
    fn write_object(
        &mut self,
        w: &mut io::Write,
        ctx: &mut OutputContext,
        obj: &CommonBindings<T>,
    ) -> DocumentProcessingResult<()> {
        eprintln!(
            "ObjectWriter CommonBindings (JS): obj: {:?}",
            obj
        );

        match *obj {
            CommonBindings::CurrentReducerState(_) => write!(w, "state"),
            CommonBindings::CurrentItem(_) => write!(w, "_item"),
            CommonBindings::CurrentItemIndex => write!(w, "_idx"),
            CommonBindings::NamedReducerKey(ref key, _) => write!(w, "store.getState().{}", key),
            CommonBindings::NamedReducerActionParam(ref ident, _) => write!(w, "action.{}", ident),
            CommonBindings::NamedQueryParam(ref ident, _) => write!(w, "{}", ident),
            CommonBindings::NamedComponentProp(ref ident, _) => write!(w, "props.{}", ident),
            CommonBindings::ComponentPropsObject(_) => write!(w, "props"),
            CommonBindings::NamedEventBoundValue(_, _) => write!(w, "_event.target.value"),
            CommonBindings::NamedElementBoundValue(ref element_key, _) => {
                // Is this element being emitted within a component definition (function)?
                let is_component = ctx.environment()? == Some(OutputScopeEnvironment::Component);
                // Also check if we have an element key in the context (if we are actually being output as HTML)
                let element_key = match ctx.get_element_key()? {
                    Some(ref s) => format!("document.querySelector(\"[key = '{}.{}']\").value", s, element_key),
                    _ if is_component => format!("document.querySelector(\"[key = '\" + props.key + \".{}']\").value", element_key),
                    _ => format!("document.querySelector(\"[key = '{}']\").value", element_key)
                };
                write!(w, "{}", element_key)
            }
            CommonBindings::CurrentElementValue(_) => write!(w, "_event.target.value"),
            CommonBindings::CurrentElementKeyPath => write!(w, "props.key"),
            CommonBindings::PathAlias(ref path, _) => write!(w, "{}", path),
        }?;

        Ok(())
    }
}

impl ObjectWriter<ParamValue<ProcessedExpression>, JsOutput> for DefaultJsWriter {
    fn write_object(
        &mut self,
        w: &mut io::Write,
        ctx: &mut OutputContext,
        obj: &ParamValue<ProcessedExpression>,
    ) -> DocumentProcessingResult<()> {
        debug!(
            "ObjectWriter ParamValue<ProcessedExpression> (JS): obj: {:?}",
            obj
        );
        self.write_object(w, ctx, obj.value())
    }
}

impl ObjectWriter<PipelineComponentValue<ProcessedExpression>, JsOutput> for DefaultJsWriter {
    fn write_object(
        &mut self,
        w: &mut io::Write,
        ctx: &mut OutputContext,
        obj: &PipelineComponentValue<ProcessedExpression>,
    ) -> DocumentProcessingResult<()> {
        debug!(
            "ObjectWriter PipelineComponentValue<ProcessedExpression> (JS): obj: {:?}",
            obj
        );

        match *obj {
            PipelineComponentValue::Member(ref s) => {
                write!(w, "{}", s)?;
                Ok(())
            }

            PipelineComponentValue::MethodCall(ref s, ref params, _) => {
                write!(w, "{}(", s)?;
                if let &Some(ref params) = params {
                    let mut first = true;
                    for param in params {
                        if !first {
                            write!(w, ",")?;
                        }
                        self.write_object(w, ctx, param)?;
                        first = false;
                    }
                }
                write!(w, ")")?;
                Ok(())
            }
        }
    }
}

fn write_pipeline_head<T: Debug>(
    _self: &mut DefaultJsWriter,
    w: &mut io::Write,
    ctx: &mut OutputContext,
    head: &ExpressionValue<T>,
) -> DocumentProcessingResult<()>
where
    DefaultJsWriter: ObjectWriter<ExpressionValue<T>, JsOutput>,
{
    eprintln!("[JS] write_pipeline_head: head: {:?}", head);
    match *head {
        ExpressionValue::Binding(CommonBindings::CurrentReducerState(_), _)
        | ExpressionValue::Binding(CommonBindings::NamedQueryParam(..), _) => {
            write!(w, "wrap(")?;
            _self.write_object(w, ctx, head)?;
            write!(w, ")")?;
        }

        _ => {
            _self.write_object(w, ctx, head)?;
        }
    };

    Ok(())
}

fn get_binding_shape(ctx: &mut OutputContext, binding: &CommonBindings<ProcessedExpression>) -> DocumentProcessingResult<Option<OuterShape>> {
    if let Some(ExpressionValue::BindingShape(BindingShape(_, ref shape), _)) = ctx.find_value(binding)? {
        return Ok(Some(shape.to_owned()));
    };

    Ok(None)
}

fn get_current_reducer_shape(ctx: &mut OutputContext) -> DocumentProcessingResult<Option<OuterShape>> {
    let binding = CommonBindings::CurrentReducerState(Default::default());
    get_binding_shape(ctx, &binding)
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct EvaluateValue<T>(pub ExpressionValue<T>);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct ReducerOutputValue<T>(pub ExpressionValue<T>);

impl ObjectWriter<EvaluateValue<ProcessedExpression>, JsOutput> for DefaultJsWriter {
    fn write_object(
        &mut self,
        w: &mut io::Write,
        ctx: &mut OutputContext,
        obj: &EvaluateValue<ProcessedExpression>,
    ) -> DocumentProcessingResult<()> {
        let shape = match obj.0 {
            ExpressionValue::Binding(ref binding, _) => get_binding_shape(ctx, binding)?,
            _ => None
        };

        match shape {
            Some(OuterShape::Array) | Some(OuterShape::Map) => {
                write!(w, "values(")?;
                self.write_object(w, ctx, &obj.0)?;
                write!(w, ")")?;
            }

            _ => {
                self.write_object(w, ctx, &obj.0)?;
            }
        };

        Ok(())
    }
}

impl ObjectWriter<ReducerOutputValue<ProcessedExpression>, JsOutput> for DefaultJsWriter {
    fn write_object(
        &mut self,
        w: &mut io::Write,
        ctx: &mut OutputContext,
        obj: &ReducerOutputValue<ProcessedExpression>,
    ) -> DocumentProcessingResult<()> {
        let shape = get_current_reducer_shape(ctx)?;

        if shape == Some(OuterShape::Array) || shape == Some(OuterShape::Map) {
            write!(w, "asMap(undefined, ")?;
            self.write_object(w, ctx, &obj.0)?;
            write!(w, ")")?;
        } else {
            self.write_object(w, ctx, &obj.0)?;
        }

        Ok(())
    }
}

impl ObjectWriter<ShapedExpressionValue<ProcessedExpression>, JsOutput> for DefaultJsWriter {
    fn write_object(
        &mut self,
        w: &mut io::Write,
        ctx: &mut OutputContext,
        obj: &ShapedExpressionValue<ProcessedExpression>,
    ) -> DocumentProcessingResult<()> {
        match obj.0 {
            OuterShape::Array | OuterShape::Map => {
                write!(w, "values(")?;
                self.write_object(w, ctx, &obj.1)?;
                write!(w, ")")?;
            }

            _ => {
                self.write_object(w, ctx, &obj.1)?;
            }
        };

        Ok(())
    }
}

impl ObjectWriter<PipelineValue<ProcessedExpression>, JsOutput> for DefaultJsWriter {
    fn write_object(
        &mut self,
        w: &mut io::Write,
        ctx: &mut OutputContext,
        obj: &PipelineValue<ProcessedExpression>,
    ) -> DocumentProcessingResult<()> {
        debug!(
            "ObjectWriter PipelineValue<ProcessedExpression> (JS): obj: {:?}",
            obj
        );

        // Construct composite pipeline (function)
        write!(w, "pipe(")?;
        let mut first = true;
        if obj.has_components() {
            let components = obj.components();
            for component in components {
                if !first { write!(w, ", ")?; }
                self.write_object(w, ctx, component)?;
                first = false;
            }
        };
        write!(w, ")")?;

        // Apply pipeline as a function to a value
        write!(w, "(")?;
        let write_shape = EvaluateValue(obj.head().to_owned());
        self.write_object(w, ctx, &write_shape)?;
        write!(w, ")")?;

        Ok(())
    }
}

///
/// Filter (sql-like) pipelines
///

impl ObjectWriter<FilterValue<ProcessedExpression>, JsOutput> for DefaultJsWriter {
    fn write_object(
        &mut self,
        w: &mut io::Write,
        ctx: &mut OutputContext,
        obj: &FilterValue<ProcessedExpression>,
    ) -> DocumentProcessingResult<()> {
        eprintln!(
            "ObjectWriter FilterValue<ProcessedExpression> (JS): obj: {:?}",
            obj
        );

        // Construct composite pipeline (function)
        write!(w, "pipe(")?;
        let mut first = true;
        if obj.has_components() {
            let components = obj.components();
            for component in components {
                if !first { write!(w, ", ")?; }
                self.write_object(w, ctx, component)?;
                first = false;
            }
        };
        write!(w, ")")?;

        // Apply pipeline as a function to a value
        write!(w, "(")?;
        let write_shape = EvaluateValue(obj.head().to_owned());
        self.write_object(w, ctx, &write_shape)?;
        write!(w, ")")?;

        Ok(())
    }
}

impl ObjectWriter<FilterComponentValue<ProcessedExpression>, JsOutput> for DefaultJsWriter {
    fn write_object(
        &mut self,
        w: &mut io::Write,
        ctx: &mut OutputContext,
        obj: &FilterComponentValue<ProcessedExpression>,
    ) -> DocumentProcessingResult<()> {
        eprintln!(
            "ObjectWriter FilterComponentValue<ProcessedExpression> (JS): obj: {:?}",
            obj
        );

        match *obj {
            FilterComponentValue::Where(ref wc, _) => {
                write!(w, "filterFunc(_item => ")?;
                self.write_object(w, ctx, wc)?;
                write!(w, ")")?;

                Ok(())
            }

            FilterComponentValue::Set(ref v, ref wc, _) => {
                write!(w, "setObjectFunc(_item => ({{")?;

                let mut first = true;
                for set_assignment in v {
                    match *set_assignment {
                        FilterSetAssignment::SetMemberTo(ref s, ref e, _) => {
                            if !first {
                                write!(w, ", ")?;
                            }
                            write!(w, "\"{}\": ", s)?;
                            self.write_object(w, ctx, e)?;
                            first = false;
                        }
                    }
                }
                write!(w, "}})")?;

                if let &Some(ref wc) = wc {
                    write!(w, ", _item => ")?;
                    self.write_object(w, ctx, wc)?;
                };

                write!(w, ")")?;

                Ok(())
            }

            FilterComponentValue::Delete(ref wc, _) => {
                write!(w, "_utils.removeObject(_item => ")?;
                self.write_object(w, ctx, wc)?;
                write!(w, ")")?;

                Ok(())
            }

            FilterComponentValue::Unique(ref mapping, _) => {
                write!(w, "_utils.uniqFunc(_item => ")?;
                self.write_object(w, ctx, mapping)?;
                write!(w, ")")?;

                Ok(())
            }
        }
    }
}

impl ObjectWriter<FilterWhereClause<ProcessedExpression>, JsOutput> for DefaultJsWriter {
    fn write_object(
        &mut self,
        w: &mut io::Write,
        ctx: &mut OutputContext,
        obj: &FilterWhereClause<ProcessedExpression>,
    ) -> DocumentProcessingResult<()> {
        debug!(
            "ObjectWriter FilterWhereClause<ProcessedExpression> (JS): obj: {:?}",
            obj
        );

        let anded_conditions = obj.anded_conditions();
        let mut first = true;
        for cond in anded_conditions {
            if !first {
                write!(w, " && ")?;
            }
            self.write_object(w, ctx, cond)?;
            first = false;
        }

        Ok(())
    }
}

///
/// Reduced pipeline
///

impl ObjectWriter<ReducedPipelineValue<ProcessedExpression>, JsOutput> for DefaultJsWriter {
    fn write_object(
        &mut self,
        w: &mut io::Write,
        ctx: &mut OutputContext,
        obj: &ReducedPipelineValue<ProcessedExpression>,
    ) -> DocumentProcessingResult<()> {
        debug!(
            "ObjectWriter ReducedPipelineValue<ProcessedExpression> (JS): obj: {:?}",
            obj
        );

        let components: Vec<_> = obj.components().collect();

        // Special case, this pipeline returns a scalar value
        let is_scalar = match components.last() {
            Some(&&ReducedPipelineComponent::PipelineOp(ReducedMethodCall::First)) => true,
            Some(&&ReducedPipelineComponent::PipelineOp(ReducedMethodCall::Count(..))) => true,
            Some(&&ReducedPipelineComponent::PipelineOp(ReducedMethodCall::MinBy(..))) => true,
            Some(&&ReducedPipelineComponent::PipelineOp(ReducedMethodCall::MaxBy(..))) => true,
            _ => false
        };

        if !is_scalar {
            write!(w, "Array.from(")?;
        }
        write!(w, "pipe(")?;

        let mut first = true;

        for component in components {
            if !first { writeln!(w, ", ")?; }
            self.write_object(w, ctx, component)?;
            first = false;
        }
        write!(w, ")(values(")?;
        self.write_object(w, ctx, obj.head())?;
        write!(w, "))")?;
        if !is_scalar {
            write!(w, ")")?;
        };
        Ok(())
    }
}

impl ObjectWriter<ReducedPipelineComponent<ProcessedExpression>, JsOutput> for DefaultJsWriter {
    fn write_object(
        &mut self,
        w: &mut io::Write,
        ctx: &mut OutputContext,
        obj: &ReducedPipelineComponent<ProcessedExpression>,
    ) -> DocumentProcessingResult<()> {
        match *obj {
            ReducedPipelineComponent::PipelineOp(ref op) => {
                match *op {
                    ReducedMethodCall::Map(ref expr) => {
                        write!(w, "mapFunc(_item => ")?;
                        self.write_object(w, ctx, expr)?;
                        write!(w, ")")?;
                    }

                    // ReducedMethodCall::MapIf(ref expr, ref cond) => {
                    //     write!(w, ".map(_item => ")?;
                    //     self.write_object(w, ctx, expr)?;
                    //     write!(w, ", _item => ")?;
                    //     self.write_object(w, ctx, cond)?;
                    //     write!(w, ")")?;
                    // }

                    ReducedMethodCall::Filter(ref cond) => {
                        // write!(w, "ng.filter(_item => ")?;
                        write!(w, "filterFunc(_item => ")?;
                        self.write_object(w, ctx, cond)?;
                        write!(w, ")")?;
                    }

                    ReducedMethodCall::Reduce(ref expr, ref initial) => {
                        write!(w, "reduceFunc(_item => ")?;
                        self.write_object(w, ctx, expr)?;
                        write!(w, ", ")?;
                        self.write_object(w, ctx, initial)?;
                        write!(w, ")")?;
                    }

                    ReducedMethodCall::ReduceIf(ref expr, ref cond, ref initial) => {
                        write!(w, "_utils.reduceIf(_item => ")?;
                        self.write_object(w, ctx, expr)?;
                        write!(w, ", ")?;
                        self.write_object(w, ctx, initial)?;
                        write!(w, ", ")?;
                        self.write_object(w, ctx, cond)?;
                        write!(w, ")")?;
                    }

                    // TODO: Fix mapping function
                    ReducedMethodCall::Uniq(_) => {
                        // write!(w, ".filter((function(_keys) {{ return function(_item) {{ let _key = item[_key]; ")?;
                        // write!(
                        //     w,
                        //     "; return !(_keys.has(_key) || !_keys.add(_key) ); }}}})(new Set()))"
                        // )?;
                        write!(w, "uniqFunc")?;
                    }

                    // ReducedMethodCall::UniqByKey(ref key) => {
                    //     write!(w, ".filter((function(_keys) {{ return function(_item) {{ let _key = item[_key]; ")?;
                    //     write!(
                    //         w,
                    //         "; return !(_keys.has(_key) || !_keys.add(_key) ); }}}})(new Set()))"
                    //     )?;
                    // }

                    // ReducedMethodCall::MinBy(ref expr) => {
                    //     // write!(w, ".reduce((v, acc) => (v < acc) ? v : acc)")?;
                    //     write!(w, ".min(_item => ")?;
                    //     self.write_object(w, ctx, expr)?;
                    //     write!(w, ")")?;
                    // }

                    ReducedMethodCall::MaxBy(ref expr) => {
                        // write!(w, ".reduce((v, acc) => (v > acc) ? v : acc)")?;
                        write!(w, "maxByFunc(_item => ")?;
                        self.write_object(w, ctx, expr)?;
                        write!(w, ")")?;
                    }

                    ReducedMethodCall::Count(ref expr) => {
                        // write!(w, ".count(_item => ")?;
                        // self.write_object(w, ctx, expr)?;
                        // write!(w, ")")?;
                        // write!(w, "ng.map((() => { let n = 0; return _item => ")?;
                        // self.write_object(w, ctx, expr)?;
                        // write!(w, "})())")?;

                        write!(w, "_utils.countIfFunc(_item => ")?;
                        self.write_object(w, ctx, expr)?;
                        write!(w, ")")?;
                    }

                    // ReducedMethodCall::FirstWhere(ref cond) => {
                    //     write!(w, ".first(_item => ")?;
                    //     self.write_object(w, ctx, cond)?;
                    //     write!(w, ")")?;
                    // }

                    ReducedMethodCall::First => {
                        // write!(w, ".first()")?;
                        // write!(w, "ng.take(1)")?;
                        write!(w, "first")?;
                    }

                    _ => {
                        return Err(try_eval_from_err!("Reduced method call not supported."));
                    }
                }
                Ok(())
            }

            ReducedPipelineComponent::Member(ref name) => {
                write!(w, ".{}", name)?;
                Ok(())
            }

            // ReducedPipelineComponent::ExpressionValue(ref expr) => self.write_object(w, ctx, expr)
            _ => Ok(()),
        }
    }
}

///
/// Basic and compound expressions
///

impl ObjectWriter<Expression<ProcessedExpression>, JsOutput> for DefaultJsWriter {
    fn write_object(
        &mut self,
        w: &mut io::Write,
        ctx: &mut OutputContext,
        obj: &Expression<ProcessedExpression>,
    ) -> DocumentProcessingResult<()> {
        eprintln!(
            "ObjectWriter Expression<ProcessedExpression> (JS): obj: {:?}",
            obj
        );

        match *obj {
            Expression::Composite(ref c) => self.write_object(w, ctx, c),

            Expression::Path(ref p, _) => self.write_object(w, ctx, p),

            Expression::Ident(ref s, _) => {
                write!(w, "{}", s)?;
                Ok(())
            }

            Expression::RawPath(ref s, _) => {
                write!(w, "{}", s)?;
                Ok(())
            }

            // Expression::Binding(ref b) => {
            //     self.write_object(w, ctx, b)?;
            // }
            Expression::Pipeline(ref p, _) => self.write_object(w, ctx, p),
            Expression::Filter(ref f, _) => self.write_object(w, ctx, f),

            Expression::ReducedPipeline(ref p, _) => self.write_object(w, ctx, p),

            Expression::Group(Some(box ref e)) => {
                write!(w, "(")?;
                self.write_object(w, ctx, e)?;
                write!(w, ")")?;

                Ok(())
            }

            Expression::Group(_) => {
                write!(w, "()")?;
                Ok(())
            }

            Expression::UnaryOp(ref op, box ref a) => {
                match *op {
                    UnaryOpType::Negate => write!(w, "!"),
                }?;

                self.write_object(w, ctx, a)
            }

            Expression::BinaryOp(ref op, box ref a, box ref b) => {
                if let BinaryOpType::Add = *op {
                    match (a, b) {
                        (
                            &ExpressionValue::Binding(CommonBindings::CurrentReducerState(_), _),
                            _,
                        ) => {
                            let shape = get_current_reducer_shape(ctx)?;
                            match shape {
                                Some(OuterShape::Array) | Some(OuterShape::Map) => {
                                    write!(w, "flatten(values(")?;
                                    self.write_object(w, ctx, a)?;
                                    write!(w, "), ")?;
                                    self.write_object(w, ctx, b)?;
                                    write!(w, ")")?;
                                }

                                _ => {
                                    self.write_object(w, ctx, a)?;
                                    write!(w, " + ")?;
                                    self.write_object(w, ctx, b)?;
                                }
                            };

                            return Ok(());
                        }

                        (
                            &ExpressionValue::Expression(Expression::Composite(
                                CompositeValue::ObjectValue(_),
                            )),
                            _,
                        )
                        | (
                            _,
                            &ExpressionValue::Expression(Expression::Composite(
                                CompositeValue::ObjectValue(_),
                            )),
                        ) => {
                            write!(w, "Object.assign({{}}, ")?;
                            self.write_object(w, ctx, a)?;
                            write!(w, ", ")?;
                            self.write_object(w, ctx, b)?;
                            write!(w, ")")?;

                            return Ok(());
                        }

                        _ => {}
                    };
                }

                self.write_object(w, ctx, a)?;

                match *op {
                    BinaryOpType::Add => write!(w, " + "),
                    BinaryOpType::Sub => write!(w, " - "),
                    BinaryOpType::Mul => write!(w, " * "),
                    BinaryOpType::Div => write!(w, " / "),
                    BinaryOpType::EqualTo => write!(w, " == "),
                    BinaryOpType::NotEqualTo => write!(w, " != "),
                    BinaryOpType::LessThan => write!(w, " < "),
                    BinaryOpType::LessThanOrEqualTo => write!(w, " <= "),
                    BinaryOpType::GreaterThan => write!(w, " > "),
                    BinaryOpType::GreaterThanOrEqualTo => write!(w, " >= "),
                }?;

                self.write_object(w, ctx, b)?;

                Ok(())
            }

            Expression::QueryCall(ref query_call, _) => self.write_object(w, ctx, query_call),

            _ => {
                eprintln!("ObjectWriter Expression<ProcessedExpression> (JS): Unsupported Expression: {:?}", obj);
                Err(try_process_from_err!(
                    "Unsupported expression in JS writer."
                ))
            }
        }
    }
}

impl ObjectWriter<Expression<OutputExpression>, JsOutput> for DefaultJsWriter {
    fn write_object(
        &mut self,
        w: &mut io::Write,
        ctx: &mut OutputContext,
        obj: &Expression<OutputExpression>,
    ) -> DocumentProcessingResult<()> {
        eprintln!(
            "ObjectWriter Expression<OutputExpression> (JS): obj: {:?}",
            obj
        );

        match *obj {
            Expression::Composite(ref c) => self.write_object(w, ctx, c),

            Expression::Path(ref p, _) => self.write_object(w, ctx, p),

            Expression::Ident(ref s, _) => {
                write!(w, "{}", s)?;
                Ok(())
            }

            Expression::RawPath(ref s, _) => {
                write!(w, "{}", s)?;
                Ok(())
            }

            _ => {
                eprintln!(
                    "ObjectWriter Expression<OutputExpression> (JS): Unsupported Expression: {:?}",
                    obj
                );
                Err(try_process_from_err!(
                    "Unsupported expression in JS writer."
                ))
            }
        }
    }
}

impl ObjectWriter<QueryCall<ProcessedExpression>, JsOutput> for DefaultJsWriter {
    fn write_object(
        &mut self,
        w: &mut io::Write,
        ctx: &mut OutputContext,
        obj: &QueryCall<ProcessedExpression>,
    ) -> DocumentProcessingResult<()> {
        let name = obj.name();
        let params = obj.params();

        write!(w, "query_{}(store", name)?;

        for param in params {
            write!(w, ", ")?;
            self.write_object(w, ctx, param)?;
        }

        write!(w, ")")?;

        Ok(())
    }
}

impl ObjectWriter<QueryParamValue<ProcessedExpression>, JsOutput> for DefaultJsWriter {
    fn write_object(
        &mut self,
        w: &mut io::Write,
        ctx: &mut OutputContext,
        obj: &QueryParamValue<ProcessedExpression>,
    ) -> DocumentProcessingResult<()> {
        self.write_object(w, ctx, obj.value())
    }
}

impl<T> ObjectWriter<ObjectValue<T>, JsOutput> for DefaultJsWriter
    where DefaultJsWriter: ObjectWriter<ExpressionValue<T>, JsOutput>
{
    fn write_object(
        &mut self,
        w: &mut io::Write,
        ctx: &mut OutputContext,
        obj: &ObjectValue<T>,
    ) -> DocumentProcessingResult<()> {
        let mut first = true;
        write!(w, "{{")?;
        if let Some(&box ref props) = obj.0.as_ref() {
            for prop in props {
                if !first {
                    write!(w, ", ")?;
                }
                write!(w, "\"{}\": ", prop.key())?;
                self.write_object(w, ctx, prop.value())?;
                first = false;
            }
        };
        write!(w, "}}")?;
        Ok(())
    }
}

impl<T> ObjectWriter<ArrayValue<T>, JsOutput> for DefaultJsWriter
    where DefaultJsWriter: ObjectWriter<ExpressionValue<T>, JsOutput>
{
    fn write_object(
        &mut self,
        w: &mut io::Write,
        ctx: &mut OutputContext,
        obj: &ArrayValue<T>,
    ) -> DocumentProcessingResult<()> {
        let mut first = true;
        write!(w, "[")?;
        if let Some(box ref params) = obj.0 {
            for param in params {
                if !first {
                    write!(w, ", ")?;
                }
                self.write_object(w, ctx, param.value())?;
                first = false;
            }
        };
        write!(w, "]")?;
        Ok(())
    }
}

impl<T> ObjectWriter<ArrayOf<T>, JsOutput> for DefaultJsWriter
    where DefaultJsWriter: ObjectWriter<T, JsOutput>
{
    fn write_object(
        &mut self,
        w: &mut io::Write,
        ctx: &mut OutputContext,
        obj: &ArrayOf<T>,
    ) -> DocumentProcessingResult<()> {
        let mut first = true;
        write!(w, "[")?;
        if let Some(box ref params) = obj.0 {
            for param in params {
                if !first {
                    write!(w, ", ")?;
                }
                self.write_object(w, ctx, param)?;
                first = false;
            }
        };
        write!(w, "]")?;
        Ok(())
    }
}

impl<T> ObjectWriter<MapValue<T>, JsOutput> for DefaultJsWriter
    where DefaultJsWriter: ObjectWriter<ObjectValue<T>, JsOutput>
{
    fn write_object(
        &mut self,
        w: &mut io::Write,
        ctx: &mut OutputContext,
        obj: &MapValue<T>,
    ) -> DocumentProcessingResult<()> {
        let auto_id = obj.0.as_ref().map(|s| s.to_owned())
            .unwrap_or("id".to_owned());
        write!(w, "asMap(e => e.{}, [", auto_id)?;
        if let Some(box ref entries) = obj.1 {
            let mut first = true;
            for entry in entries {
                if !first {
                    write!(w, ", ")?;
                }
                self.write_object(w, ctx, entry)?;
                first = false;
            }
        };
        write!(w, "])")?;
        Ok(())
    }
}

impl<T> ObjectWriter<CompositeValue<T>, JsOutput> for DefaultJsWriter
    where DefaultJsWriter : ObjectWriter<ArrayValue<T>, JsOutput>,
        DefaultJsWriter: ObjectWriter<ObjectValue<T>, JsOutput>,
        DefaultJsWriter: ObjectWriter<MapValue<T>, JsOutput>
{
    fn write_object(
        &mut self,
        w: &mut io::Write,
        ctx: &mut OutputContext,
        obj: &CompositeValue<T>,
    ) -> DocumentProcessingResult<()> {
        match *obj {
            CompositeValue::ArrayValue(ref value) => self.write_object(w, ctx, value),
            CompositeValue::ObjectValue(ref value) => self.write_object(w, ctx, value),
            CompositeValue::MapValue(ref value) => self.write_object(w, ctx, value),
        }
    }
}

impl ObjectWriter<PathValue<ProcessedExpression>, JsOutput> for DefaultJsWriter {
    fn write_object(
        &mut self,
        w: &mut io::Write,
        ctx: &mut OutputContext,
        obj: &PathValue<ProcessedExpression>,
    ) -> DocumentProcessingResult<()> {
        debug!(
            "ObjectWriter PathValue<ProcessedExpression> (JS): obj: {:?}",
            obj
        );

        write_pipeline_head(self, w, ctx, obj.head())?;

        if let Some(components) = obj.components() {
            for component in components {
                write!(w, ".{}", component)?;
            }
        }

        Ok(())
    }
}

impl ObjectWriter<PathComponentValue<ProcessedExpression>, JsOutput> for DefaultJsWriter {
    fn write_object(
        &mut self,
        w: &mut io::Write,
        ctx: &mut OutputContext,
        obj: &PathComponentValue<ProcessedExpression>,
    ) -> DocumentProcessingResult<()> {
        debug!(
            "ObjectWriter PathComponentValue<ProcessedExpression> (JS): obj: {:?}",
            obj
        );

        match *obj {
            PathComponentValue::Member(ref s, _) => {
                write!(w, "{}", s)?;
                Ok(())
            }

            PathComponentValue::MethodCall(ref s, ref params, _) => {
                write!(w, "{}(", s)?;
                if let &Some(ref params) = params {
                    let mut first = true;
                    for param in params {
                        if !first {
                            write!(w, ",")?;
                        }
                        self.write_object(w, ctx, param)?;
                        first = false;
                    }
                }
                write!(w, ")")?;
                Ok(())
            }
        }
    }
}

impl ObjectWriter<PathValue<OutputExpression>, JsOutput> for DefaultJsWriter {
    fn write_object(
        &mut self,
        w: &mut io::Write,
        ctx: &mut OutputContext,
        obj: &PathValue<OutputExpression>,
    ) -> DocumentProcessingResult<()> {
        debug!(
            "ObjectWriter PathValue<OutputExpression> (JS): obj: {:?}",
            obj
        );

        write_pipeline_head(self, w, ctx, obj.head())?;

        if let Some(components) = obj.components() {
            for component in components {
                write!(w, ".{}", component)?;
            }
        }

        Ok(())
    }
}

impl ObjectWriter<PathComponentValue<OutputExpression>, JsOutput> for DefaultJsWriter {
    fn write_object(
        &mut self,
        w: &mut io::Write,
        _ctx: &mut OutputContext,
        obj: &PathComponentValue<OutputExpression>,
    ) -> DocumentProcessingResult<()> {
        debug!(
            "ObjectWriter PathComponentValue<OutputExpression> (JS): obj: {:?}",
            obj
        );

        match *obj {
            PathComponentValue::Member(ref s, _) => {
                write!(w, "{}", s)?;
                Ok(())
            }

            PathComponentValue::MethodCall(ref s, ref _params, _) => {
                write!(w, "{}(...)", s)?;
                // if let &Some(ref params) = params {
                //     let mut first = true;
                //     for param in params {
                //         if !first { write!(w, ",")?; }
                //         self.write_object(w, ctx, param)?;
                //         first = false;
                //     }
                // }
                // write!(w, ")")?;
                Ok(())
            }
        }
    }
}

///
/// Components
///

impl ObjectWriter<Component<ProcessedExpression>, JsOutput> for DefaultJsWriter {
    fn write_object(
        &mut self,
        w: &mut io::Write,
        ctx: &mut OutputContext,
        obj: &Component<ProcessedExpression>,
    ) -> DocumentProcessingResult<()> {
        debug!(
            "ObjectWriter Component<ProcessedExpression> (JS): obj: {:?}",
            obj
        );

        ctx.push_child_scope_with_environment(OutputScopeEnvironment::Component);

        // NOTE: Writing of the function has moved to the internal template

        let block = obj.block();
        self.write_object(w, ctx, block)?;

        ctx.pop_scope();
        Ok(())
    }
}

///
/// Content expressions and elements
///

impl ObjectWriter<Block<ProcessedExpression>, JsOutput> for DefaultJsWriter {
    fn write_object(
        &mut self,
        w: &mut io::Write,
        ctx: &mut OutputContext,
        obj: &Block<ProcessedExpression>,
    ) -> DocumentProcessingResult<()> {
        debug!(
            "ObjectWriter Block<ProcessedExpression> (JS): obj: {:?}",
            obj
        );

        if let Some(ops) = obj.ops() {
            for op in ops {
                self.write_object(w, ctx, op)?;
            }
        }

        Ok(())
    }
}

// Event binding (within attribute list)
impl ObjectWriter<ElementEventBindingOutput<ProcessedExpression>, JsOutput> for DefaultJsWriter {
    fn write_object(
        &mut self,
        w: &mut io::Write,
        ctx: &mut OutputContext,
        obj: &ElementEventBindingOutput<ProcessedExpression>,
    ) -> DocumentProcessingResult<()> {
        eprintln!(
            "ObjectWriter ElementEventBindingOutput<ProcessedExpression> (JS): obj: {:?}",
            obj
        );

        let name = obj.0.as_ref().map(|s| s.to_owned()).unwrap_or("click".to_owned());
        let key = &obj.1;

        write!(
            w,
            ", \"on{}\", e => _events.{}(e, ",
            name,
            key
        )?;

        // let obj: ObjectValue<OutputExpression> =
        //     TryEvalFrom::try_eval_from(&obj.2, ctx)?;
        let props = &obj.2;

        self.write_object(w, ctx, props)?;

        write!(
            w,
            ")"
        )?;

        Ok(())
    }
}

fn write_open<'s>(
    _self: &'s mut DefaultJsWriter,
    w: &mut io::Write,
    ctx: &mut OutputContext,
    desc: &ElementDescriptor<ProcessedExpression>,
    is_void: bool,
) -> DocumentProcessingResult<()> {
    let tag = desc.tag();
    let element_key = ctx.get_element_key()?
        .map(|s| format!("{}.{}", s, desc.key()))
        .unwrap_or_else(|| desc.key().to_owned());

    let string_props = desc.string_props();

    let is_input = desc.tag() == "input";
    let type_prop = string_props.get("type").map(|s| s.as_str());
    let is_checkbox = is_input && type_prop == Some("checkbox");
    let is_textbox = is_input && type_prop.is_none() ||  type_prop == Some("text") || type_prop == Some("password");

    if !is_void {
        write!(w, "IncrementalDOM.elementOpen(\"{}\", ", tag)?;
    } else {
        write!(w, "IncrementalDOM.elementVoid(\"{}\", ", tag)?;
    };

    // Is this element being emitted within a component definition (function)?
    let key_string = match ctx.environment()? {
        Some(OutputScopeEnvironment::Component) => format!("props.key + \".{}\"", element_key),
        _ => format!("\"{}\"", element_key),
    };

    write!(w, "{}, [", key_string)?;

    write!(w, "\"key\", {}", key_string)?;

    let props: Vec<_> = desc.props()
        .map(|prop| (prop.name().to_owned(), prop.expr().to_owned()))
        .collect();

    let const_props: Vec<_> = props
        .iter()
        .filter_map(|prop| match prop {
            _ if prop.1.is_primitive() => Some(prop.to_owned()),
            _ => None,
        })
        .collect();
    let eval_props: Vec<_> = props
        .iter()
        .filter_map(|prop| match prop {
            _ if !prop.1.is_primitive() => Some(prop.to_owned()),
            _ => None,
        })
        .collect();

    // let const_prop_map: HashMap<_, _> = const_props.iter().map(|prop| (prop.name(), prop.to_owned()))

    // let eval_props: Vec<(String, ExpressionValue<OutputExpression>)> = TryEvalFrom::try_eval_from(&eval_props, ctx)?;

    // statics
    for prop in const_props {
        let (name, expr) = prop;
        write!(w, ", \"{}\", ", name)?;
        _self.write_object(w, ctx, &expr)?;
    }
    write!(w, "]")?;

    // events (cannot be static)
    if let Some(events) = desc.events() {
        for event_binding in events {
            let event_output: ElementEventBindingOutput<ProcessedExpression> = event_binding.into();
            _self.write_object(w, ctx, &event_output)?;
        }
    }

    // attributes
    for prop in eval_props {
        write!(w, ", ")?;

        let (name, expr) = prop;
        let use_classes = name == "class" && expr.is_object();

        if (tag == "input" || tag == "button") && name == "disabled" {
            _self.write_object(w, ctx, &expr)?;
            write!(w, " ? 'disabled' : null, ")?;

            _self.write_object(w, ctx, &expr)?;
            write!(w, " ? 'disabled' : null")?;

            continue;
        };

        write!(w, "\"{}\", ", name)?;
        if use_classes {
            write!(w, "classes(")?;
        }
        _self.write_object(w, ctx, &expr)?;
        if use_classes {
            write!(w, ")")?;
        }
    }

    if let Some(value_binding) = desc.value_binding() {
        if is_checkbox {
            if let Some(read_expr) = value_binding.read_expr() {
                write!(w, ", ")?;

                _self.write_object(w, ctx, read_expr)?;
                write!(w, " ? 'checked' : null, ")?;

                _self.write_object(w, ctx, read_expr)?;
                write!(w, " ? 'checked' : null")?;
            };
        } else if is_textbox {
            if let Some(read_expr) = value_binding.read_expr() {
                write!(w, ", \"value\", ")?;
                _self.write_object(w, ctx, read_expr)?;
            };
        }
    };
    write!(w, ")")?;

    // Needed to update a value when the value would (nominally) be the same as the attribute, such as for inputs
    if let Some(read_expr) = desc.value_binding().and_then(|b| b.read_expr()) {
        if is_textbox {
            write!(w, ".value = ")?;
            _self.write_object(w, ctx, read_expr)?;
        } else if is_checkbox {
            write!(w, ".checked = ")?;
            _self.write_object(w, ctx, read_expr)?;
        }
    }

    writeln!(w, ";")?;

    Ok(())
}

fn write_comp_desc<'s>(
    _self: &'s mut DefaultJsWriter,
    w: &mut io::Write,
    ctx: &mut OutputContext,
    comp_desc: &ComponentInstanceDescriptor<ProcessedExpression>,
    item_key: Option<&str>,
    is_map: bool,
) -> DocumentProcessingResult<()> {
    let component_key = comp_desc.key();
    let tag = comp_desc.tag();

    // let component_props: Vec<ElementPropValue<ProcessedExpression>> = props.as_ref().map(|v| v.iter().map(|p| p.to_owned()).collect()).unwrap_or_default();
    let component_props = comp_desc.props();

    // Is this component instance (function call) being emitted within a component definition (function)?
    let key_string = match ctx.environment()? {
        Some(OutputScopeEnvironment::Component) if is_map => {
            format!("props.key + \".{}.\" + _item[0]", component_key)
        }
        Some(OutputScopeEnvironment::Component) => format!("props.key + \".{}\"", component_key),
        _ => format!("\"{}\"", component_key),
    };

    write!(w, "{}Component(", tag)?;
    if let Some(component_props) = component_props {
        write!(w, "{{\"key\": {}", key_string)?;

        // let props = component_props.iter()
        //     .map(|prop| {
        //         let val;

        //         if let Some(item_key) = item_key {
        //             val = "_item[1]";
        //         } else {
        //             val = prop.key().to_owned();
        //         };

        //         val
        //     });

        // if let Some(item_key) = item_key {
        //     if !first {
        //         write!(w, ", ")?;
        //     }
        //     write!(w, "{}: _item[1]", item_key)?;
        //     first = false;
        // };

        for prop in component_props {
            let key = prop.name();

            write!(w, ", {}: ", prop.name())?;
            if item_key == Some(key) {
                write!(w, "_item[1]")?;
            } else {
                _self.write_object(w, ctx, prop.expr())?;
            };
        }

        // if is_map {
        //     // FIXME
        //     let key = format!("\"{}_\" + _item[1].id", comp_desc.key());

        //     if !first { write!(w, ", ")?; }
        //     write!(w, "key: {}", key)?;
        //     first = false;
        // }

        write!(w, "}}")?;
    };
    writeln!(w, ");")?;

    Ok(())
}

impl ObjectWriter<ElementOp<ProcessedExpression>, JsOutput> for DefaultJsWriter {
    fn write_object(
        &mut self,
        w: &mut io::Write,
        ctx: &mut OutputContext,
        obj: &ElementOp<ProcessedExpression>,
    ) -> DocumentProcessingResult<()> {
        debug!(
            "ObjectWriter ElementOp<ProcessedExpression> (JS): obj: {:?}",
            obj
        );

        match *obj {
            ElementOp::ElementOpen(ref desc, _) => {
                // ctx.push_child_scope();
                // ctx.bind_element_key(desc.key(), None)?;
                write_open(self, w, ctx, desc, false)?;
                Ok(())
            }

            ElementOp::ElementClose(ref tag) => {
                writeln!(w, "IncrementalDOM.elementClose(\"{}\");", tag)?;
                // ctx.pop_scope();
                Ok(())
            }

            ElementOp::ElementVoid(ref desc, _) => {
                // ctx.push_child_scope();
                // ctx.bind_element_key(desc.key(), None)?;
                write_open(self, w, ctx, desc, true)?;
                // ctx.pop_scope();
                Ok(())
            }

            ElementOp::SkipNode => {
                writeln!(w, "// old SkipNode")?;
                Ok(())
            }

            ElementOp::SkipOuterElement(ref e) => {
                match *e {
                    SkipElementOp::ElementOpen(..) | SkipElementOp::ElementVoid(..) | SkipElementOp::WriteValue(..) => {
                        writeln!(w, "IncrementalDOM.skipNode();")?;
                    }

                    _ => {}
                };

                Ok(())
            }

            ElementOp::SkipElement(..) => {
                // Don't call skipNode() for children
                writeln!(w, "// not skipping element")?;
                Ok(())
            }

            ElementOp::StartBlock(_) => Ok(()),

            ElementOp::EndBlock(_) => Ok(()),

            ElementOp::MapCollection(_, _, _, _) => Ok(()),

            ElementOp::WriteValue(ref e, _) => {
                write!(w, "IncrementalDOM.text(")?;
                self.write_object(w, ctx, e)?;
                writeln!(w, ");")?;

                Ok(())
            }

            ElementOp::InstanceComponent(ref comp_desc, _) => {
                write_comp_desc(self, w, ctx, comp_desc, None, false)
            }

            ElementOp::MapInstanceComponent(ref comp_desc, ref item_key, ref coll, _) => {
                write!(w, "for (const _item of enumerate(values(")?;
                self.write_object(w, ctx, coll)?;
                writeln!(w, "))) {{")?;

                let item_key = item_key.as_ref().map(|s| s.as_str());
                write_comp_desc(self, w, ctx, comp_desc, item_key, true)?;
                writeln!(w, "}}")?;

                Ok(())
            }
        }
    }
}

/// Store and reducers

impl ObjectWriter<ReducerAction<ProcessedExpression>, JsOutput> for DefaultJsWriter {
    fn write_object(
        &mut self,
        w: &mut io::Write,
        ctx: &mut OutputContext,
        action: &ReducerAction<ProcessedExpression>,
    ) -> DocumentProcessingResult<()> {
        let expr = action.expr();

        if expr.is_none() {
            write!(w, "state || null;")?;
            return Ok(());
        };

        let expr = expr.unwrap();

        let reducer_output = ReducerOutputValue(expr.to_owned());
        self.write_object(w, ctx, &reducer_output)?;

        Ok(())
    }
}

/// Queries

impl ObjectWriter<Query<ProcessedExpression>, JsOutput> for DefaultJsWriter {
    fn write_object(
        &mut self,
        w: &mut io::Write,
        ctx: &mut OutputContext,
        query: &Query<ProcessedExpression>,
    ) -> DocumentProcessingResult<()> {
        for component in query.components() {
            eprintln!(
                "ObjectWriter QueryComponentValue<ProcessedExpression> (JS): component: {:?}",
                component
            );

            match *component {
                QueryComponent::CaseWhere(box ref expr, box ref cond, _) => {
                    write!(w, "        if (")?;
                    self.write_object(w, ctx, cond)?;
                    write!(w, ") {{ return ")?;
                    self.write_object(w, ctx, expr)?;
                    writeln!(w, "; }}")?;
                }
            }
        }

        Ok(())
    }
}

/// Actions

impl ObjectWriter<ActionOpOutput<ProcessedExpression>, JsOutput> for DefaultJsWriter {
    fn write_object(
        &mut self,
        w: &mut io::Write,
        ctx: &mut OutputContext,
        obj: &ActionOpOutput<ProcessedExpression>,
    ) -> DocumentProcessingResult<()> {
        let is_route_dispatch = ctx.environment()? == Some(OutputScopeEnvironment::RouteDispatchAction);
        let action = &obj.1;
        let prefix = obj.0.as_ref().map(|s| format!("props.{}", s)).unwrap_or_else(|| "props".to_owned());
        match *action {
            ActionOp::DispatchAction(ref name, ref props, _)
            | ActionOp::DispatchActionTo(ref name, ref props, _, _) => {
                let action = match *action {
                    ActionOp::DispatchActionTo(_, _, ref target, _) => {
                        format!("{}.{}", target.to_uppercase(), name.to_uppercase())
                    }
                    _ => name.to_uppercase(),
                };
                write!(w, "            store.dispatch({{type: \"{}\"", action)?;
                if let Some(box ref props) = *props {
                    for prop in props {
                        write!(w, ", \"{}\": ", prop.key())?;
                        if is_route_dispatch || prop.value().is_primitive() {
                            self.write_object(w, ctx, prop.value())?;
                        } else {
                            write!(w, "{}.{}", prefix, prop.key())?;
                        }
                    }
                };
                writeln!(w, "}});")?;
            }

            ActionOp::Navigate(ref prop, _) => {
                write!(w, "store.dispatch(navigate(")?;
                if  is_route_dispatch || prop.is_primitive() {
                    self.write_object(w, ctx, prop)?;
                } else {
                    write!(w, "{}", prefix)?;
                };
                write!(w, "));")?;
            }
        };

        Ok(())
    }
}

/// Routes

impl ObjectWriter<RouteActionValue<ProcessedExpression>, JsOutput> for DefaultJsWriter {
    fn write_object(
        &mut self,
        w: &mut io::Write,
        ctx: &mut OutputContext,
        obj: &RouteActionValue<ProcessedExpression>,
    ) -> DocumentProcessingResult<()> {
        match *obj {
            RouteActionValue::Block(ref block, _) => {
                self.write_object(w, ctx, block)?;
            }

            RouteActionValue::Actions(Some(ref actions), _) => for action in actions {
                ctx.push_child_scope_with_environment(OutputScopeEnvironment::RouteDispatchAction);
                let  action_output = ActionOpOutput(None, action.to_owned());
                self.write_object(w, ctx, &action_output)?;
                ctx.pop_scope();
            },
            RouteActionValue::Actions(..) => {}
        }

        Ok(())
    }
}
