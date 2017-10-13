
use std::io;

use model::*;
use scope::*;
use processing::*;
use output::*;


// const STRING_HTML_OPEN_INCDOM_PAGE: &'static str = r#"
// <!doctype HTML>
// <html>
//     <head>
//         <meta charset="utf-8" />
//         <!-- <script src="https://unpkg.com/redux@3.7.1/dist/redux.js"></script> -->
//         <!-- <script src="https://ajax.googleapis.com/ajax/libs/incrementaldom/0.5.1/incremental-dom.js" defer="defer"></script> -->
//         <script src="./pkg/redux@3.7.1/dist/redux.js"></script>
//         <script src="./pkg/incrementaldom@0.5.1/incremental-dom.js" defer="defer"></script>
//     </head>
//     <body>"#;

const STRING_HTML_OPEN_INCDOM_PAGE: &'static str = r#"
<!doctype HTML>
<html>
    <head>
        <meta charset="utf-8" />
        <script src="https://unpkg.com/redux@3.7.1/dist/redux.js"></script>
        <script src="https://ajax.googleapis.com/ajax/libs/incrementaldom/0.5.1/incremental-dom.js" defer="defer"></script>
    </head>
    <body>"#;

// Note there cannot be a space after the opening element, otherwise IncrementalDOM will fail
// to match the first child, which results in a flash of unstyled content in the TodoMVC demo
// when link nodes are removed and re-added.
const STRING_HTML_OPEN_ROOT_DIV: &'static str  = r#"
        <div id="root">"#;

const STRING_HTML_CLOSE_ROOT_DIV: &'static str  = r#"</div>
"#;

const STRING_HTML_OPEN_SCRIPT_IIFE: &'static str  = r#"
        <script>
            (function(_global) {
                function Blank() {}
                Blank.prototype = Object.create(null);
                
                let listeners = new Map();

                function setEventListener(key, el, evt, func) {
                    let listener_key = key + '_' + evt;
                    if (listeners.has(listener_key)) {
                        el.removeEventListener(evt, listeners.get(listener_key));
                    };
                    el.addEventListener(evt, func);
                    listeners.set(listener_key, func);
                }

                function markExisting(nodes) {
                    Array.prototype.slice.call(nodes).forEach(function(node) {
                        IncrementalDOM.importNode(node);
                        var data = node['__incrementalDOMData'];
                        data.staticsApplied = true;
                        data.newAttrs = new Blank();
                    });
                }

                function classList() {
                    return [].slice.call(arguments).filter(function(e) { return !!e; }).join(' ');
                }

                function update(root_el, store) {
                    IncrementalDOM.patch(root_el, render.bind(null, store));
                }
"#;

const STRING_JS_OPEN_RENDER: &'static str  = r#"
                function render(store) {
"#;

const STRING_JS_OPEN_ROOT_BINDINGS_DEF: &'static str  = r#"
                function root_bindings(store) {
"#;

const STRING_JS_OPEN_DEFINE_ROUTES: &'static str  = r#"
                function define_routes(store, routes) {
"#;

const STRING_JS_CLOSE_RENDER: &'static str  = r#"                }
"#;

const STRING_HTML_CLOSE_SCRIPT_IIFE: &'static str  = r#"
                document.addEventListener("DOMContentLoaded", function(event) {
                    // Import nodes
                    markExisting(document.querySelectorAll("[key]"));
                    // Define store
                    var store = Redux.createStore(rootReducer, {});
                    // Subscribe
                    store.subscribe(update.bind(null, document.querySelector('#root'), store));
                    // Bind events
                    root_bindings(store);
                });
            })();
        </script>"#;

const STRING_HTML_CLOSE_SCRIPT_ROUTES_IIFE: &'static str  = r#"

                function createHistory() {

                    return {
                        init: function(store) { 
                            window.addEventListener('popstate', function(event) {
                                store.dispatch({type: '@@redux-routing/replace', href: event.state});
                            });
                            
                            store.dispatch({type: '@@redux-routing/replace', href: window.location.pathname });
                        },
                        update: function(action) {
                            switch (action.type) {
                                case '@@redux-routing/navigate':
                                    window.history.pushState(action.href, null, action.href); break;
                                case '@@redux-routing/replace':
                                    window.history.replaceState(action.href, null, action.href); break;
                                default: break;
                            }
                        }
                    };
                }

                function createRoutingMiddleware(routes, history) {
                    return function() {
                        return function(next) {
                            return function(action) {
                                if (!/^@@redux-routing/.test(action.type)) {
                                    return next(action);
                                }

                                var href = action.href;
                                var match = Object.keys(routes).map(function(pattern) { return (href == pattern) ? routes[pattern] : null; })[0];

                                if (match && match.handler) {
                                    match.handler(href);
                                }

                                history.update(action);
                            }
                        }
                    }
                }

                document.addEventListener("DOMContentLoaded", function(event) {
                    // Import nodes
                    markExisting(document.querySelectorAll("[key]"));

                    // Setup router
                    var routes = {};
                    var history = createHistory();
                    var middleware = createRoutingMiddleware(routes, history);
                    var store = Redux.applyMiddleware(middleware)(Redux.createStore)(rootReducer);

                    _global.navigate = function(href) { return { type: '@@redux-routing/navigate', href: href }; };

                    // Define routes
                    define_routes(store, routes);
                    // Subscribe
                    store.subscribe(update.bind(null, document.querySelector('#root'), store));
                    // Bind events
                    root_bindings(store);

                    // Start routing
                    history.init(store);
                });
            })(window);
        </script>"#;

const STRING_HTML_CLOSE_INCDOM_PAGE: &'static str  = r#"
    </body>
</html>
"#;

pub struct PageWriter<'doc> {
    doc: &'doc Document,
    writers: DefaultOutputWritersBoth,
}

impl<'doc> PageWriter<'doc> {
    #[allow(dead_code)]
    pub fn with_doc(doc: &'doc Document) -> Self {
        PageWriter {
            doc: doc,
            writers: Default::default()
        }
    }

    // pub fn push_content_context(&mut self, ctx: &mut Context, _bindings: &BindingContext) -> Result {
    //     ctx.push_child_scope();
    //     if let Some(ref default_reducer_key) = self.doc.default_reducer_key {
    //         ctx.append_action_path_str(default_reducer_key);
    //     };

    //     for reducer_data in self.doc.reducer_key_data.values() {
    //         let reducer_key = &reducer_data.reducer_key;
    //         let binding = BindingType::ReducerPathBinding(reducer_key.to_owned());
    //         ctx.add_sym(&reducer_key, Symbol::binding(&binding));
    //     }
    //     Ok(())
    // }

    #[inline]
    #[allow(unused_variables)]
    pub fn write_component_definitions(&mut self, w: &mut io::Write, ctx: &mut Context, bindings: &BindingContext) -> Result {
        if let Some(comp_iter) = self.doc.get_component_definitions() {
            for (component_ty, comp_def) in comp_iter {
                ctx.push_child_scope();
                let key_binding = ExprValue::Binding(BindingType::ComponentKeyBinding);
                ctx.append_path_expr(&key_binding);

                writeln!(w, "                function component_{}(key, props, store) {{", component_ty)?;
                self.writers.js.write_block(w, self.doc, ctx, bindings, comp_def.root_block(), Some("div"), true)?;
                writeln!(w, "                  component_bindings_{}(key, props, store);", component_ty)?;
                writeln!(w, "\n{}", self::STRING_JS_CLOSE_RENDER)?;

                writeln!(w, "                function component_bindings_{}(key, props, store) {{", component_ty)?;
                if let Some(events_iter) = comp_def.root_block().events_iter() {
                    self.writers.js.write_event_bindings(w, self.doc, ctx, bindings, events_iter)?;
                };
                writeln!(w, "\n{}", self::STRING_JS_CLOSE_RENDER)?;

                ctx.pop_scope();
            }
        };
        Ok(())
    }

    #[inline]
    #[allow(unused_variables)]
    pub fn write_route_definitions(&mut self, w: &mut io::Write, ctx: &mut Context, bindings: &BindingContext) -> Result {
        if let Some(iter) = self.doc.get_routes() {
            for (pattern, route) in iter {
                let func_key = route.function_key();
                writeln!(w, "                function route_{}(path, props, store) {{", func_key)?;
                if let Some(iter) = route.action_ops_iter() {
                    self.writers.js.write_event_action_ops(w, self.doc, ctx, bindings, iter)?;
                };
                writeln!(w, "\n{}", self::STRING_JS_CLOSE_RENDER)?;
            }
        };
        Ok(())
    }

    #[inline]
    #[allow(unused_variables)]
    pub fn write_query_definitions(&mut self, w: &mut io::Write, ctx: &mut Context, bindings: &BindingContext) -> Result {
        if let Some(iter) = self.doc.get_queries() {
            for (name, query) in iter {
                self.writers.js.write_query(w, self.doc, ctx, bindings, query)?;
            }
        };
        Ok(())
    }

    #[inline]
    #[allow(unused_variables)]
    pub fn write_route_registrations(&mut self, w: &mut io::Write, ctx: &mut Context, bindings: &BindingContext) -> Result {
        if let Some(iter) = self.doc.get_routes() {
            writeln!(w, "{}", self::STRING_JS_OPEN_DEFINE_ROUTES)?;
            for (pattern, route) in iter {
                let func_key = route.function_key();
                writeln!(w, "                routes[\"{}\"] = {{ handler: function(path) {{ route_{}(path, {{}}, store); }} }};", pattern, func_key)?;
            }
            writeln!(w, "{}", self::STRING_JS_CLOSE_RENDER)?;
        };
        Ok(())
    }

    #[inline]
    #[allow(unused_variables)]
    pub fn write_root_block_render_definition(&mut self, w: &mut io::Write, ctx: &mut Context, bindings: &BindingContext) -> Result {
        write!(w, "{}", self::STRING_JS_OPEN_RENDER)?;
        self.writers.js.write_block(w, self.doc, ctx, bindings, self.doc.root_block(), None, true)?;
        write!(w, "{}", self::STRING_JS_CLOSE_RENDER)?;
        Ok(())
    }

    #[inline]
    #[allow(unused_variables)]
    pub fn write_initial_event_bindings(&mut self, w: &mut io::Write, ctx: &mut Context, bindings: &BindingContext) -> Result {
        let events_iter = self.writers.html.events_iter();
        self.writers.js.write_bound_events(w, self.doc, ctx, bindings, events_iter)?;
        Ok(())
    }

    #[inline]
    #[allow(unused_variables)]
    pub fn write_root_bindings_definition(&mut self, w: &mut io::Write, ctx: &mut Context, bindings: &BindingContext) -> Result {
        writeln!(w, "{}", self::STRING_JS_OPEN_ROOT_BINDINGS_DEF)?;
        self.write_initial_event_bindings(w, ctx, bindings)?;
        writeln!(w, "{}", self::STRING_JS_CLOSE_RENDER)?;
        Ok(())
    }

    #[inline]
    #[allow(unused_variables)]
    pub fn write_root_html(&mut self, w: &mut io::Write, ctx: &mut Context, bindings: &BindingContext) -> Result {
        // self.push_content_context(ctx, bindings)?;

        write!(w, "{}", self::STRING_HTML_OPEN_ROOT_DIV)?;
        if let Some(ops_iter) = self.doc.root_block().ops_iter() {
            self.writers.html.write_element_ops(w, self.doc, ctx, bindings, ops_iter)?;
        };
        write!(w, "{}", self::STRING_HTML_CLOSE_ROOT_DIV)?;

        // ctx.pop_scope();
        Ok(())
    }

    #[inline]
    #[allow(unused_variables)]
    pub fn write_element_rendering_script_html(&mut self, w: &mut io::Write, ctx: &mut Context, bindings: &BindingContext) -> Result {
        ctx.push_root_content_scope(&self.doc);

        // Define components
        self.write_component_definitions(w, ctx, bindings)?;
        // Define route handlers
        self.write_route_definitions(w, ctx, bindings)?;
        self.write_route_registrations(w, ctx, bindings)?;
        self.write_query_definitions(w, ctx, bindings)?;
        self.write_root_block_render_definition(w, ctx, bindings)?;
        self.write_root_bindings_definition(w, ctx, bindings)?;

        ctx.pop_scope();
        Ok(())
    }

    #[inline]
    #[allow(unused_variables)]
    pub fn write_script_html(&mut self, w: &mut io::Write, ctx: &mut Context, bindings: &BindingContext) -> Result {
        write!(w, "{}", self::STRING_HTML_OPEN_SCRIPT_IIFE)?;

        writeln!(w, "/* {:?} */", self.doc.default_state_map)?;
        writeln!(w, "/* {:?} */", self.doc.root_block().ops_iter().map(|v| v.into_iter().collect::<Vec<&ElementOp>>()))?;
        writeln!(w, "/* {:?} */", self.doc.root_block().events_iter().map(|v| v.into_iter().collect::<Vec<&EventsItem>>()))?;

        self.write_element_rendering_script_html(w, ctx, bindings)?;

        let mut store_writer = StoreWriterJs::default();
        store_writer.write_store(w, self.doc, self.writers.js(), ctx, bindings, self.doc.reducers_iter())?;

        if self.doc.has_routes() {
            write!(w, "{}", self::STRING_HTML_CLOSE_SCRIPT_ROUTES_IIFE)?;        
        } else {
            write!(w, "{}", self::STRING_HTML_CLOSE_SCRIPT_IIFE)?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    #[allow(unused_variables)]
    pub fn write_page(&mut self, w: &mut io::Write, ctx: &mut Context, bindings: &BindingContext) -> Result {
        // if let Some(ref default_reducer_key) = self.doc.default_reducer_key {
        //     ctx.append_action_path_str(default_reducer_key);
        // };

        write!(w, "{}", self::STRING_HTML_OPEN_INCDOM_PAGE)?;

        // let mut events_vec: EventsVec = Default::default();
        // self.output_html.write_html_ops_content(w, ops_iter, base_scope, Some(&mut events_vec))?;

        self.write_root_html(w, ctx, bindings)?;
        writeln!(w, "")?;
        self.write_script_html(w, ctx, bindings)?;

        write!(w, "{}", self::STRING_HTML_CLOSE_INCDOM_PAGE)?;

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::str;
    use processing::*;
    use parser::loc::Loc;

    
    fn create_template() -> Template {
        let node_1 = ContentNodeType::ExpressionValueNode(ExprValue::LiteralString("hi".into()), "45daee35".into());
        let node_2 = ContentNodeType::ElementNode(ElementType { element_ty: "span".into(), element_key: "Cd".into(), attrs: None, lens: None, bindings: None, children: Some(vec![node_1])});
        let node_3 = ContentNodeType::ElementNode(ElementType { element_ty: "div".into(), element_key: "Ab".into(), attrs: None, lens: None, bindings: None, children: Some(vec![node_2])});
        
        let nodes: Vec<Loc<NodeType, (usize, usize)>> = vec![
            Loc {inner: NodeType::ContentNode(node_3), pos: (0,0)}
        ];
        Template { children: nodes }
    }

    fn prepare_document<'a>(template: &'a Template) -> Document {
        let mut ctx = Context::default();
        let mut bindings = BindingContext::default();
        let mut processing = ProcessDocument::from_template(&template);
        assert!(processing.process_document(&mut ctx, &mut bindings).is_ok());
        processing.into()
    }

    #[test]
    pub fn test_output_page_writer_html() {
        let template = create_template();
        let doc = prepare_document(&template);
        let mut page_writer = PageWriter::with_doc(&doc);
 
        let mut ctx = Context::default();
        let bindings = BindingContext::default();
        let mut s: Vec<u8> = Default::default();
        let res = page_writer.write_page(&mut s, &mut ctx, &bindings);
        assert!(res.is_ok());
        assert_diff!(str::from_utf8(&s).unwrap(), r#"
<!doctype HTML>
<html>
    <head>
        <meta charset="utf-8" />
        <!-- <script src="https://unpkg.com/redux@3.7.1/dist/redux.js"></script> -->
        <!-- <script src="https://ajax.googleapis.com/ajax/libs/incrementaldom/0.5.1/incremental-dom.js" defer="defer"></script> -->
        <script src="./pkg/redux@3.7.1/dist/redux.js"></script>
        <script src="./pkg/incrementaldom@0.5.1/incremental-dom.js" defer="defer"></script>
    </head>
    <body>
        <div id="root">
<div key="Ab"><span key="Ab.Cd">hi</span></div>
        </div>

        <script>
            (function() {
                function Blank() {}
                Blank.prototype = Object.create(null);
                
                function markExisting(nodes) {
                    Array.prototype.slice.call(nodes).forEach(function(node) {
                        IncrementalDOM.importNode(node);
                        var data = node['__incrementalDOMData'];
                        data.staticsApplied = true;
                        data.newAttrs = new Blank();
                    });
                }

                function update(root_el, store) {
                    IncrementalDOM.patch(root_el, render.bind(null, store));
                }
/* {} */
/* [ElementOpen("div", "Ab", None, None, None), ElementOpen("span", "Ab.Cd", None, None, None), WriteValue(LiteralString("hi"), "Ab.Cd.45daee35"), ElementClose("span"), ElementClose("div")] */

                function render(store) {
IncrementalDOM.elementOpen("div", "Ab");
IncrementalDOM.elementOpen("span", "Ab.Cd");
IncrementalDOM.text("hi");
IncrementalDOM.elementClose("span");
IncrementalDOM.elementClose("div");
                }

                var rootReducer = Redux.combineReducers({
                });

                document.addEventListener("DOMContentLoaded", function(event) {
                    // Import nodes
                    markExisting(document.querySelectorAll("[key]"));
                    // Define store
                    var store = Redux.createStore(rootReducer, {});
                    // Subscribe
                    store.subscribe(update.bind(null, document.querySelector('#root'), store));
                });
            })();
        </script>
    </body>
</html>
"#, "\n", 0);
    }
}