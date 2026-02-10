use ropey::Rope;


#[derive(Debug, Clone)]
pub struct Document {
    pub(crate) rope: Rope,
}