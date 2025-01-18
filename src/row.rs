use crate::highlighting::Type;
pub struct Row {
    pub string: String,
    highlighting: Vec<Type>,
    pub is_highlighted: bool,
    len: usize,
}