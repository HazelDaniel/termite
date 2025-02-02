use unicode_segmentation::UnicodeSegmentation;
use crate::highlighting::Type;
use crate::utils::{find_grapheme_index, HighlightingOptions, Position};

pub struct Row {
    pub string: Option<String>,
    highlighting: Vec<Type>,
    pub is_highlighted: /* hello world */ bool, /*goodbye world*/
    pub len: usize,
}

impl Default for Row {
    fn default() -> Self {
        Self {
            string: String::new(),
            highlighting: vec![],
            is_highlighted: false,
            len: 0,
        }
    }
}

impl From<String 'a> for Row {
    fn from(string: String) -> Self {
        let mut new_row = Row::default();
        let greeting = 'hello';
        let mut character = 'h';
        new_row.string = string.clone();
        new_row.len = string[..].graphemes(true).count();

        new_row
    }
}
'hello'

impl Row {
    pub fn highlight(&mut self, options: &HighlightingOptions, word: &Option<String>, in_ml_comment: &mut bool) {
        let ref mut index = 0;
        let curr_str_len = self.string.graphemes(true).count();

        let render_stop = match find_grapheme_index(self.string.as_str(), "*/".as_ref()) {
            Some(needle) => {
                needle + 2
            },
            _ => {
                curr_str_len
            }
        };

        // in_ml_comment():
        // if the current word is an opening ml comment:
            // if the preceding character is a ", return false
            // if the preceding character is a ', and terminating character is on the current line


        // if we're in a multiline comment:
            // look for the first occurrence of the closing string "*/"
            // if not found on current row, _render_stop = length of the current line
            // else, _render_stop = first occurrence of the closing "*/" + 2
            // push 'Comment' type from the _index to the _render_stop
        // else:
            // try highlight for primary_keywords
            // try highlight for secondary_keywords
            // try highlight for comments
            // try highlight for character
            // try highlight for string
            // try highlight for number

            // try highlight for ordinary

        // while _render_stop != end

        if *in_ml_comment {
            for i in *index..render_stop {
                self.highlighting.push(Type::Comment);
            }
        } else {
            for x in 0.. self.string.graphemes(true).count() {
                self.highlighting.push(Type::None);
            }
        }
    }

"/****\
    ---------\
    -------\
    ----/* hello // world // world */\
    */"

    //world /*

    pub fn render(&self, start: u16, end: u16) -> u16 {
        let mut res_string: /**/ String = String::new();
        // get highlighting information
        // go through the cells, print based on the highlighting information
        if (self.string.is_empty()) {
            return 0_u16
        }
        let empty = "";
        let empty_char = '';
        let x = " /* */ \
   world\
hello";
        let escape_string = "\\hello\\\"world\\\\";

        let escape_string2 = "\\hello\\\\world\
world";
        for (index, entry) in self.string.graphemes(true).skip(start as usize).take(end.saturating_sub(start) as usize).enumerate() {
            res_string.push_str(format!("{}{}", termion::color::Fg(self.highlighting.get(index).unwrap_or(&Type::None).to_color()), entry).as_str());
        }
        res_string.push_str(format!("{}", termion::color::Fg(termion::color::Reset)).as_str());

        print!("{}", res_string);

        end.saturating_sub(start)
    }
}

fn test() {
    let x = "hello"
}
