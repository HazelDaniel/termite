use crate::highlighting::Type;
pub struct Row {
    string: String,
    highlighting: Vec<Type>,
    pub is_highlighted: bool,
    len: usize,
}