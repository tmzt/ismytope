
use std::collections::hash_map::HashMap;
use parser::ast::*;
use parser::store::*;

#[derive(Debug)]
pub struct ReducerKeyData {
    pub reducer_key: String,
    pub default_expr: Option<ExprValue>,
    pub actions: Option<Vec<ReducerActionData>>,
}

impl ReducerKeyData {
    pub fn from_name(reducer_key: &str) -> ReducerKeyData {
        ReducerKeyData {
            reducer_key: String::from(reducer_key),
            default_expr: None,
            actions: Some(Vec::new()),
        }
    }
}

#[derive(Debug)]
pub struct ReducerActionData {
    pub action_type: String,
    pub state_expr: Option<ActionStateExprType>,
}

impl ReducerActionData {
    pub fn from_name(action_name: &str) -> ReducerActionData {
        let action_type = action_name.to_uppercase();

        ReducerActionData {
            action_type: String::from(action_type),
            state_expr: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ElementOp {
    ElementOpen(String, Option<String>, Option<Vec<(String, ExprValue)>>, Option<EventHandlersVec>),
    ElementVoid(String, Option<String>, Option<Vec<(String, ExprValue)>>, Option<EventHandlersVec>),
    ElementClose(String),
    WriteValue(ExprValue, Option<String>),
    InstanceComponent(String, Option<String>, Option<Vec<(String, ExprValue)>>),
}

pub type OpsVec = Vec<ElementOp>;
pub type ComponentMap<'inp> = HashMap<&'inp str, Component<'inp>>;

#[derive(Debug, Clone)]
pub struct Component<'input> {
    pub name: &'input str,
    pub ops: Option<OpsVec>,
    pub uses: Option<Vec<&'input str>>,
    pub child_map: Option<ComponentMap<'input>>,
}
