use std::time::Duration;
use std::thread;
use crate::highlighting::Type;
use crate::utils::{find_grapheme_index, HighlightStreak, HighlightingOptions, Position};
use unicode_segmentation::UnicodeSegmentation;

pub struct Row {
    pub string: String,
    highlighting: Vec<Type>,
    pub is_highlighted: bool,
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

impl From<String> for Row {
    fn from(string: String) -> Self {
        let mut new_row = Row::default();
        new_row.string = string.clone();
        new_row.len = string[..].graphemes(true).count();

        new_row
    }
}

impl Row {
    pub fn highlight(
        &mut self,
        options: &HighlightingOptions,
        word: &Option<String>,
        hl_streak: &mut HighlightStreak,
    ) {
        if self.is_highlighted {
            return;
        }
        let ref mut index = 0;
        let curr_str_len = self.string.graphemes(true).count();

        while *index < curr_str_len {
            if (
                self.highlight_ml(options, word, hl_streak, index)
                || self.highlight_str(options, word, hl_streak, index)
                || self.highlight_comment(options, word, hl_streak, index)
                || self.highlight_char(options, word, hl_streak, index)
                || self.highlight_keyword(options, word, hl_streak, index)
            ) {
                continue;
            } else {
                self.highlighting.push(Type::None);
                *index += 1;
            }
        }
        self.is_highlighted = true;
    }

    pub fn render(&self, start: u16, end: u16) -> u16 {
        let mut res_string: String = String::new();
        if (self.string.is_empty()) {
            return 0_u16;
        }
        for (index, entry) in self
            .string
            .graphemes(true)
            .skip(start as usize)
            .take(end.saturating_sub(start) as usize)
            .enumerate()
        {
            res_string.push_str(
                format!("{}{}", termion::color::Fg(self.highlighting .get(index) .unwrap_or(&Type::None).to_color()), entry)
                .as_str(),
            );
        }
        res_string.push_str(format!("{}", termion::color::Fg(termion::color::Reset)).as_str());

        print!("{}", res_string);

        end.saturating_sub(start)
    }

    pub fn highlight_number(
        &mut self,
        options: &HighlightingOptions,
        word: &Option<String>,
        streak: &mut HighlightStreak,
        index: &mut usize,
    ) -> bool {
        true
    }

    pub fn highlight_keyword(
        &mut self,
        options: &HighlightingOptions,
        word: &Option<String>,
        streak: &mut HighlightStreak,
        index: &mut usize,
    ) -> bool {
        let curr_str_len = self.string.graphemes(true).count();
        let ref mut index_cpy  = (*index).clone();
        let mut graphemes = self.string.graphemes(true).skip(*index_cpy).collect::<Vec<&str>>();
        let graphemes_shift_left = self.string.graphemes(true).skip((*index_cpy).saturating_sub(1)).collect::<Vec<&str>>();

        if let Some(word_start) = graphemes.get(0) {
            if (*word_start).is_ascii() {
                let letter = (*word_start).chars().next();
                match letter {
                    Some(c) => {
                        if !((c as char).is_ascii_graphic() && ((c as char).is_alphabetic() || (c as char) == '_')) {
                            return false;
                        }
                        if let Some(prev) = graphemes_shift_left.get(0) {
                            if !prev.trim().is_empty() && *index != 0 {
                                return false;
                            }
                        }
                    },
                    _ => {
                        return false;
                    }
                }
            } else {
                return false;
            }
        } else {
            return false;
        }

        /*INVARIANT:
        * we are starting at the beginning of a valid word
        */

        let mut keyword: String = String::new();
        let mut k_count = 0;
        while k_count < graphemes.len() {
            if let Some(entry) = graphemes.get(k_count) {
                if (*entry).is_ascii() && ((*entry).chars().next().unwrap_or(' ').is_alphabetic() || (*entry).chars().next().unwrap_or(' ') == '_') {
                    keyword.push_str(*entry);
                } else {
                    break;
                }
            }

            k_count += 1;
        }

        if (keyword.is_empty()) {
            return false;
        }
        if options.primary_keywords.contains(&keyword)
            || options.secondary_keywords.contains(&keyword)
            || options.known_items.contains(&keyword){
            for _ in 0..keyword.len() {
                if options.primary_keywords.contains(&keyword) {
                    self.highlighting.push(Type::PrimaryKeyword);
                } else if options.secondary_keywords.contains(&keyword) {
                    self.highlighting.push(Type::SecondaryKeyword);
                } else {
                    self.highlighting.push(Type::KnownItem);
                }
            }
        } else {
            return false;
        }

        *index += keyword.len();

        true
    }

    pub fn highlight_pattern(
        &mut self,
        options: &HighlightingOptions,
        word: &Option<String>,
        streak: &mut HighlightStreak,
        index: &mut usize,
    ) -> bool {
        true
    }

    pub fn highlight_char(
        &mut self,
        options: &HighlightingOptions,
        word: &Option<String>,
        streak: &mut HighlightStreak,
        index: &mut usize,
    ) -> bool {

        let curr_str_len = self.string.graphemes(true).count();
        let quote_render_start = if let Some(needle) = find_grapheme_index(self.string.as_str(), *index, "'".as_ref()) {
            needle.saturating_add(*index)
        } else {
            curr_str_len
        };

        if quote_render_start != *index {
            return false;
        }

        /*INVARIANT:
        * '*index' is always at the start of a quote
        * no streak needed
        */

        let ref mut index_cpy  = (*index).clone();
        let mut quote_render_end: usize = 0;
        let mut graphemes = self.string.graphemes(true).skip(*index_cpy).collect::<Vec<&str>>();
        let mut terminated: bool = false;

        for cluster in &graphemes[..] {
            if *index_cpy == *index { // opening quote
                *index_cpy += 1;
                continue;
            }
            if *cluster == "'" {
                if let Some(prev) = graphemes.get((*index_cpy).saturating_sub(*index).saturating_sub(1)) {
                    if *prev != "\\" {
                        *index_cpy += 1;
                        terminated = true;
                        break;
                    }
                }
            }
            *index_cpy += 1
        }
        quote_render_end = *index_cpy;

        if quote_render_end >= curr_str_len {
            if terminated {
                quote_render_end = curr_str_len;
            } else {
                quote_render_end = quote_render_start.saturating_add(1);
            }
        }

        for _ in *index..quote_render_end {
            self.highlighting.push(Type::Character);
        }
        *index = quote_render_end;

        true
    }

    pub fn highlight_str(
        &mut self,
        options: &HighlightingOptions,
        word: &Option<String>,
        streak: &mut HighlightStreak,
        index: &mut usize,
    ) -> bool {
        let curr_str_len = self.string.graphemes(true).count();
        let quote_render_start = if let Some(needle) = find_grapheme_index(self.string.as_str(), *index, "\"".as_ref()) {
            needle.saturating_add(*index)
        } else {
            curr_str_len
        };

        if quote_render_start == *index && streak.comment == 0 {
            // check if we're starting a quote and not already in an ml comment
            streak.quote = true;
        }

        /*INVARIANT:
        * we're either:
         - at the start of a quote
         - in the middle of a quote
         - at the end of a quote
        * this means that we're in a quote streak
        */
        if !streak.quote {
            return false;
        };

        let ref mut index_cpy  = (*index).clone();
        let mut quote_render_end: usize = 0;
        let mut graphemes = self.string.graphemes(true).skip(*index_cpy).collect::<Vec<&str>>();

        // if you've approached the end of a line whilst still being in a quote streak and you didn't meet a \, you should stop the quote streak
        for cluster in &graphemes[..] {
           if *index_cpy == *index { // opening quote
               *index_cpy += 1;
               continue;
           }
           if *cluster == "\"" {
               if let Some(prev) = graphemes.get((*index_cpy).saturating_sub(*index).saturating_sub(1)) {
                   if *prev != "\\" {
                       *index_cpy += 1;
                       break;
                   }
               }
           }
           *index_cpy += 1
        }
        quote_render_end = (*index_cpy);

        if quote_render_end >= curr_str_len {
            quote_render_end = curr_str_len;
            match graphemes.get(graphemes.len().saturating_sub(1)) {
                Some(terminator) => {
                    match *terminator {
                        "\\" => {
                            streak.quote = true;
                        },
                        "\"" => {
                            streak.quote = false;
                        },
                        _ => {
                            streak.quote = false;
                        }
                    }
                },
                _ => {
                    streak.quote = false;
                }
            }

        } else {
            streak.quote = false;
        }

        for _ in *index..quote_render_end {
            self.highlighting.push(Type::String);
        }
        *index = quote_render_end;

        true
    }

    pub fn highlight_ml(
        &mut self,
        options: &HighlightingOptions,
        word: &Option<String>,
        streak: &mut HighlightStreak,
        index: &mut usize,
    ) -> bool {
        let curr_str_len = self.string.graphemes(true).count();

        let comment_render_start = if let Some(needle) = find_grapheme_index(self.string.as_str(), *index, "/*".as_ref()) {
            needle.saturating_add(*index)
        } else {
            curr_str_len
        };

        if comment_render_start == *index && !streak.quote {
            streak.comment = streak.comment.saturating_add(1);
        }
        /*INVARIANT:
        * we're either:
         - at the start of an ml_comment or
         - in an ml_comment or
         - at the end of an ml_comment or
        * this means that we're in a comment streak
        */
        if streak.comment == 0 {
            return false;
        };
        // while we're in a comment streak, sum up all the opening comments
        let ref mut index_cpy  = (*index).clone();
        let mut graphemes = self.string.graphemes(true).skip(*index_cpy).collect::<Vec<&str>>();

        for cluster in &graphemes[..] {
            if comment_render_start == *index { // opening comment
                *index_cpy += 1;
                continue;
            }
            if *cluster == "/" {
                if let Some(next) = graphemes.get((*index_cpy).saturating_sub(*index).saturating_add(1)) {
                    if *next == "*" {
                        streak.comment = streak.comment.saturating_add(1);
                    }
                }
            }
            *index_cpy += 1
        }

        let mut comment_render_end = if let Some(needle) = find_grapheme_index(self.string.as_str(), *index, "*/".as_ref()) {
            needle.saturating_add(2).saturating_add(*index)
        } else {
            curr_str_len.saturating_add(1)
        };

        for cluster in &graphemes[..] {
            if comment_render_start == *index { // opening comment
                *index_cpy += 1;
                continue;
            }
            if *cluster == "*" {
                if let Some(next) = graphemes.get((*index_cpy).saturating_sub(*index).saturating_add(1)) {
                    if *next == "/" {
                        streak.comment = streak.comment.saturating_sub(1);
                    }
                }
            }
            *index_cpy += 1
        }

        if comment_render_end > curr_str_len {
            comment_render_end = curr_str_len;
        } else {
            streak.comment = streak.comment.saturating_sub(1);
        }
        for _ in *index..comment_render_end {
            self.highlighting.push(Type::MultilineComment);
        }
        *index = comment_render_end;

        true
    }

    pub fn highlight_comment(
        &mut self,
        options: &HighlightingOptions,
        word: &Option<String>,
        streak: &mut HighlightStreak,
        index: &mut usize,
    ) -> bool {
        let curr_str_len = self.string.graphemes(true).count();

        let comment_render_start = if let Some(needle) = find_grapheme_index(self.string.as_str(), *index, "/".as_ref()) {
            needle.saturating_add(*index)
        } else {
            curr_str_len
        };

        /*INVARIANT:
        * we're either:
         - starting the index with a '/'
         - or we are not in a comment
        */
        if comment_render_start != *index {
            return false;
        }
        let mut graphemes = self.string.graphemes(true).skip(*index).collect::<Vec<&str>>();
        match graphemes.get(1) {
            Some(s) => {
                match *s {
                    "/" => {
                        for _ in &graphemes[..] {
                            self.highlighting.push(Type::Comment);
                        }
                        *index += graphemes.len();
                        return true;
                    },
                    _ => return false
                }
            },
            _ => return false
        }
    }
}
