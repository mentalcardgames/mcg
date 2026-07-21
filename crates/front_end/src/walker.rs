///    There are two ASTs used in this front-end:
///    Spanned AST and an unspanned (lowered) AST.
///
///    The Spanned is used for validation and parsing for
///    better Error-reporting.
///
///    The unspanned AST is given to the game engine.
///
///    The Spanned AST gets a generated Walker-Implementation as well.
///    This Walker/Visitor can be used for validation.
///
///    The type NodeKind is also generated in code_gen.
///    How NodeKind is used: Look at symbol.rs or semantic.rs.


use crate::spans::*;

use crate::ast::ast_spanned::NodeKind;

pub trait AstPass {
    fn enter_node<T: Walker>(&mut self, node: &T)
    where
        Self: Sized;
    fn exit_node<T: Walker>(&mut self, node: &T)
    where
        Self: Sized;
}

pub trait Walker {
    fn walk<V: AstPass>(&self, visitor: &mut V);
    fn kind(&self) -> Option<NodeKind<'_>>;
}

impl<T> Walker for Vec<T>
where
    T: Walker,
{
    fn walk<V: AstPass>(&self, visitor: &mut V) {
        // We iterate through the vector and call walk on every element.
        // If T is Spanned<something>, it uses your Spanned implementation.
        for item in self {
            item.walk(visitor);
        }
    }

    fn kind(&self) -> Option<NodeKind<'_>> {
        // Usually, vectors aren't considered a single "node" in the AST
        // sense for kind-tracking, so we often return a generic tag.
        None
    }
}

impl<T, S> Walker for (T, S)
where
    T: Walker,
    S: Walker,
{
    fn walk<V: AstPass>(&self, visitor: &mut V) {
        // We iterate through the tuple and call walk on every element.
        // If T is Spanned<something>, it uses your Spanned implementation.
        self.0.walk(visitor);
        self.1.walk(visitor);
    }

    fn kind(&self) -> Option<NodeKind<'_>> {
        // Usually, tuples aren't considered a single "node" in the AST
        // sense for kind-tracking, so we often return a generic tag.
        None
    }
}

impl<T, S, P> Walker for (T, S, P)
where
    T: Walker,
    S: Walker,
    P: Walker,
{
    fn walk<V: AstPass>(&self, visitor: &mut V) {
        // We iterate through the typle and call walk on every element.
        // If T is Spanned<something>, it uses your Spanned implementation.
        self.0.walk(visitor);
        self.1.walk(visitor);
        self.2.walk(visitor);
    }

    fn kind(&self) -> Option<NodeKind<'_>> {
        // Usually, tuples aren't considered a single "node" in the AST
        // sense for kind-tracking, so we often return a generic tag.
        None
    }
}

impl Walker for i32 {
    fn walk<V: AstPass>(&self, _: &mut V) {}

    fn kind(&self) -> Option<NodeKind<'_>> {
        None
    }
}

impl<T> Walker for Spanned<T>
where
    T: Walker,
{
    fn walk<V: AstPass>(&self, visitor: &mut V) {
        self.node.walk(visitor);
    }

    fn kind(&self) -> Option<NodeKind<'_>> {
        self.node.kind()
    }
}

impl<T> Walker for Box<T>
where
    T: Walker,
{
    fn walk<V: AstPass>(&self, visitor: &mut V) {
        self.as_ref().walk(visitor);
    }

    fn kind(&self) -> Option<NodeKind<'_>> {
        self.as_ref().kind()
    }
}

impl Walker for String {
    fn walk<V: AstPass>(&self, _: &mut V) {}

    fn kind(&self) -> Option<NodeKind<'_>> {
        None
    }
}
