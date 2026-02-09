use crate::{ast::ast::SID, spans::*};

pub trait Lower<T> {
    fn lower(&self) -> T;
}


impl<T, U> Lower<U> for Spanned<T>
where
    T: Lower<U>,
{
    fn lower(&self) -> U {
        // We strip the span and delegate the lowering to the inner node
        self.node.lower()
    }
}

impl Lower<String> for SID {
    fn lower(&self) -> String {
      self.node.clone()
    }
}

impl Lower<i32> for i32 {
    fn lower(&self) -> i32 {
        *self
    }
}

// For Box
impl<T, U> Lower<Box<U>> for Box<T>
where
    T: Lower<U>,
{
    fn lower(&self) -> Box<U> {
        // Use .as_ref() to get a reference to the inner T, 
        // then call lower() to get a U, then wrap it in a new Box.
        Box::new(self.as_ref().lower())
    }
}

// For Vectors
impl<T, U> Lower<Vec<U>> for Vec<T>
where
    T: Lower<U>,
{
    fn lower(&self) -> Vec<U> {
        self.iter().map(|item| item.lower()).collect()
    }
}

// For Options
impl<T, U> Lower<Option<U>> for Option<T>
where
    T: Lower<U>,
{
    fn lower(&self) -> Option<U> {
        self.as_ref().map(|item| item.lower())
    }
}

// For those specific Tuples in your errors
impl<T1, U1, T2, U2> Lower<(U1, U2)> for (T1, T2)
where
    T1: Lower<U1>,
    T2: Lower<U2>,
{
    fn lower(&self) -> (U1, U2) {
        (self.0.lower(), self.1.lower())
    }
}

// For the 3-tuple (String, String, IntExpr) seen in your logs
impl<T1, U1, T2, U2, T3, U3> Lower<(U1, U2, U3)> for (T1, T2, T3)
where
    T1: Lower<U1>,
    T2: Lower<U2>,
    T3: Lower<U3>,
{
    fn lower(&self) -> (U1, U2, U3) {
        (self.0.lower(), self.1.lower(), self.2.lower())
    }
}
