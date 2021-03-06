use std::marker::PhantomData;
use expressions::*;
use objects::*;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StoreDefinition<T>(Option<Vec<StoreRootScopeNode<T>>>, PhantomData<T>);

impl<T> StoreDefinition<T> {
    pub fn new(children: Option<Vec<StoreRootScopeNode<T>>>) -> Self {
        StoreDefinition(children, Default::default())
    }

    pub fn children<'a>(&'a self) -> Option<impl Iterator<Item = &'a StoreRootScopeNode<T>>> {
        self.0.as_ref().map(|v| v.iter())
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StoreApiChildNode<T> {
    Resource(
        String,
        Option<Box<Vec<StoreChildScopeNode<T>>>>,
        PhantomData<T>,
    ),
    Methods(Option<Vec<MethodType>>),
    Method(MethodType),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExternReducerModuleNode(String);

impl ExternReducerModuleNode {
    pub fn new(name: String) -> Self {
        ExternReducerModuleNode(name)
    }

    pub fn name(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExternReducerNode(String);

impl ExternReducerNode {
    pub fn new(name: String) -> Self {
        ExternReducerNode(name)
    }

    pub fn name(&self) -> &str {
        &self.0
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StoreCommonNode<T> {
    LetNode(String, Option<ExpressionValue<T>>, PhantomData<T>),
    ApiNode(String, Option<Vec<StoreApiChildNode<T>>>, PhantomData<T>),
    ExternReducerModuleNode(ExternReducerModuleNode, PhantomData<T>),
    ExternReducerNode(ExternReducerNode, PhantomData<T>),
    ChildScopeNode(String, Option<Vec<StoreChildScopeNode<T>>>),
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StoreRootScopeNode<T> {
    Common(StoreCommonNode<T>, PhantomData<T>),
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StoreChildScopeNode<T> {
    Common(StoreCommonNode<T>, PhantomData<T>),
    Action(ReducerAction<T>, PhantomData<T>),
}
