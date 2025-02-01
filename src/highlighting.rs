use termion::color;
pub enum Type {
    None,
    Number,
    Pattern,
    String,
    Character,
    Comment,
    MultilineComment,
    PrimaryKeyword,
    SecondaryKeyword,
    KnownItem
}

impl Type {
    pub fn to_color(&self) -> impl color::Color {
        match self {
            Type::Number => color::Rgb(220, 163, 163),
            Type::Pattern => color::Rgb(38, 139, 210),
            Type::String => color::Rgb(211, 54, 130),
            Type::Character => color::Rgb(108, 113, 196),
            Type::Comment | Type::MultilineComment => color::Rgb(153, 153, 150),
            Type::PrimaryKeyword => color::Rgb(183, 65, 14),
            Type::SecondaryKeyword => color::Rgb(42, 161, 152),
            Type::KnownItem => color::Rgb(42, 201, 152),
            _ => color::Rgb(255, 255, 255),
        }
    }
}
