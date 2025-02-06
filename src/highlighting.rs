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
            Type::Number => color::Rgb(220, 203, 203),
            Type::Pattern => color::Rgb(38, 139, 210),
            Type::String => color::Rgb(204, 95, 104),
            Type::Character => color::Rgb(204, 95, 104),
            Type::Comment | Type::MultilineComment => color::Rgb(153, 153, 150),
            Type::PrimaryKeyword => color::Rgb(183, 65, 14),
            Type::SecondaryKeyword => color::Rgb(212, 220, 160),
            Type::KnownItem => color::Rgb(42, 161, 192),
            _ => color::Rgb(240, 240, 250),
        }
    }
}
