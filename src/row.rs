use std::collections::HashSet;
use std::time::Duration;
use std::thread;
use crate::highlighting::Type;
use crate::utils::{find_grapheme_index, HighlightStreak, HighlightingOptions, Position, NumberMode};
use unicode_segmentation::UnicodeSegmentation;

pub struct Row {
    pub string:         String,
    highlighting:       Vec<Type>,
    pub is_highlighted: bool,
    pub len:            usize,
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
                || self.highlight_number(options, word, hl_streak, index)
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
                format!("{}{}", termion::color::Fg(self.highlighting.get(index.saturating_add(start as usize)) .unwrap_or(&Type::None).to_color()), entry)
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
        self.highlight_float(options, word, streak, index) // order matters here since a float is valid decimal to some extent
        || self.highlight_decimal(options, word, streak, index, NumberMode::None)
        || self.highlight_octal(options, word, streak, index)
        || self.highlight_binary(options, word, streak, index)
        || self.highlight_hex(options, word, streak, index)
        // false
    }

    pub fn highlight_decimal(
        &mut self,
        options: &HighlightingOptions,
        word: &Option<String>,
        streak: &mut HighlightStreak,
        index: &mut usize,
        mode: NumberMode
    ) -> bool {
        let curr_str_len = self.string.graphemes(true).count();
        let ref mut index_cpy  = (*index).clone();
        let mut graphemes = self.string.graphemes(true).skip(*index_cpy).collect::<Vec<&str>>();
        let graphemes_shift_left = self.string.graphemes(true).skip((*index_cpy).saturating_sub(1)).collect::<Vec<&str>>();

        if let Some(dec_start) = graphemes.get(0) {
            if !(*dec_start).chars().next().unwrap_or(' ').is_ascii_digit() {
                return false;
            }
            match (*dec_start).chars().next() {
                Some(c) => {
                    if let Some(prev) = graphemes_shift_left.get(0) {
                        if (!prev.trim().is_empty() && prev.is_ascii()) && *index != 0 {
                            match prev.chars().next() {
                                Some(c) => {
                                    match mode {
                                        NumberMode::Octal => {
                                            if c != '_' && c != 'o'  { return false; }
                                        },
                                        NumberMode::Binary => {
                                            if c != '_' && c != 'b'  { return false; }
                                        },
                                        NumberMode::Hexadecimal => {
                                            if c != '_' && c != 'x'  { return false; }
                                        },
                                        _ => {
                                            if (c.is_ascii_alphanumeric() || c == '_') { return false; }
                                        }
                                    }
                                },
                                _ => {
                                    return false;
                                }
                            }
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

        /*INVARIANT:
        * '*index' is always at the start of a valid decimal
        */

        let mut k_count = 0; while k_count < graphemes.len() {
            if let Some(entry) = graphemes.get(k_count) {
                if !(*entry).chars().next().unwrap_or(' ').is_numeric() && *entry != "_" {
                    match (*entry) {
                        "f"|"i"|"u" => {
                            let mut annotation: String = String::new();

                            let mut usize_capture_str = String::new();
                            if *entry == "u" {
                                let mut usize_capture = 0;
                                while (k_count.saturating_add(usize_capture) < graphemes.len() && usize_capture < 5) {
                                    if let Some(candidate) = graphemes.get(k_count.saturating_add(usize_capture)) {
                                        usize_capture_str.push_str(*candidate);
                                    }
                                    usize_capture += 1;
                                }
                            }
                            if usize_capture_str == "usize" {
                                k_count += usize_capture_str.len();

                                for _ in 0..k_count {
                                    self.highlighting.push(Type::Number);
                                }
                                *index = (*index).saturating_add(k_count);

                                return true;
                            }
                             else {
                                let mut annotation_capture = 1;
                                while (k_count.saturating_add(annotation_capture) < graphemes.len()) {
                                    if let Some(entry) = graphemes.get(k_count.saturating_add(annotation_capture)) {
                                        match (*entry).chars().next() {
                                            Some(c) => {
                                                if !c.is_numeric() {
                                                    if annotation_capture <= 1 || c.is_alphabetic() || c == '_' { //handling terminating character: if at least you moved once
                                                        return false;
                                                    }
                                                    break;
                                                }
                                            },
                                            _ => {
                                                return false;
                                            }
                                        }
                                    }
                                    annotation_capture += 1;
                                }
                                k_count = k_count.saturating_add(annotation_capture);

                                for _ in 0..k_count {
                                 self.highlighting.push(Type::Number);
                                }
                                *index = (*index).saturating_add(k_count);

                                return true;
                            }
                        },
                        "e" => {
                            if mode != NumberMode::Float && mode != NumberMode::Hexadecimal {return false;}
                            if k_count + 1 == graphemes.len() { return false; }

                            k_count += 1;
                            // INVARIANT: k_count is a valid index

                            let graphemes_shift_right = &graphemes[k_count..];
                            match graphemes_shift_right.get(0) {
                                Some(entry) => {
                                    match *entry {
                                        "-" => {
                                            let fill_width = k_count.saturating_add(1);

                                            for _ in 0..fill_width {
                                                self.highlighting.push(Type::Number);
                                            }

                                            *index = (*index).saturating_add(fill_width);

                                            if !self.highlight_decimal(options, word, streak, index, NumberMode::None) {
                                                let mut x = 0;
                                                while x < fill_width {
                                                    self.highlighting.pop();
                                                    x += 1;
                                                }

                                                return false;
                                            } else {
                                                return true;
                                            }

                                        },
                                        digit => {
                                            if !digit.chars().next().unwrap_or(' ').is_ascii_digit() {
                                                return false;
                                            }
                                        }
                                    }
                                },
                                _ => {
                                    return false;
                                }
                            }
                        },
                        "." => {
                            break;
                        }
                        _ => {
                            if (*entry).chars().next().unwrap_or(' ').is_ascii_alphabetic() {
                                return false;
                            } else {
                                break;
                            }
                        }
                    }
                } else {
                    k_count += 1;
                    continue;
                }
            }
        }

        for _ in 0..k_count {
            self.highlighting.push(Type::Number);
        }
        *index = (*index).saturating_add(k_count);

        true
    }

    pub fn highlight_float(
        &mut self,
        options: &HighlightingOptions,
        word: &Option<String>,
        streak: &mut HighlightStreak,
        index: &mut usize,
    ) -> bool {
        let mut last_index = (*index).clone();
        if !self.highlight_decimal(options, word, streak, index, NumberMode::Float) {
            return false;
        }

        let curr_str_len = self.string.graphemes(true).count();
        let ref mut index_cpy  = (*index).clone();
        let cloned_string = self.string.clone();
        let mut graphemes = cloned_string.graphemes(true).skip(*index_cpy).collect::<Vec<&str>>();

        if let Some(entry) = graphemes.get(0) {
            match *entry {
                "." => {
                    self.highlighting.push(Type::Number);
                    *index += 1;
                    if !self.highlight_decimal(options, word, streak, index, NumberMode::Float) {
                        let delta = (*index).saturating_sub(last_index);
                        let mut x = delta;
                        for _ in 0..x {
                            self.highlighting.pop();
                        }
                        *index -= delta;
                        return false;
                    }
                },
                non_number_tok => {
                    return false;
                },
            }
        }

        true
    }

    pub fn highlight_octal(
        &mut self,
        options: &HighlightingOptions,
        word: &Option<String>,
        streak: &mut HighlightStreak,
        index: &mut usize,
    ) -> bool {
        self.highlight_o_or_b_number(options, word, streak, index, NumberMode::Octal)
    }

    pub fn highlight_o_or_b_number(
        &mut self,
        options: &HighlightingOptions,
        word: &Option<String>,
        streak: &mut HighlightStreak,
        index: &mut usize,
        mode: NumberMode
    ) -> bool {

        let curr_str_len = self.string.graphemes(true).count();
        let ref mut index_cpy  = (*index).clone();
        let cloned_string = self.string.clone();
        let mut graphemes = cloned_string.graphemes(true).skip(*index_cpy).collect::<Vec<&str>>();
        let graphemes_shift_left = cloned_string.graphemes(true).skip((*index_cpy).saturating_sub(1)).collect::<Vec<&str>>();

        if let Some(dec_start) = graphemes.get(0) {
            if !(*dec_start).chars().next().unwrap_or(' ').is_ascii_digit() {
                return false;
            }
            match (*dec_start).chars().next() {
                Some(c) => {
                    if let Some(prev) = graphemes_shift_left.get(0) {
                        if (!(*prev).trim().is_empty() && (*prev).is_ascii()) && *index != 0 {
                            match (*prev).chars().next() {
                                Some(c) => {
                                    if (c.is_ascii_alphanumeric() || c == '_') { return false; }
                                },
                                _ => {
                                    return false;
                                }
                            }
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

        /*INVARIANT:
        * '*index' is always at the start of a valid decimal
        */

        let mut last_index = (*index).clone();

        if let Some(specifier) = graphemes.get(1) {
            if (mode == NumberMode::Binary && (*specifier) != "b")
                || (mode == NumberMode::Octal && (*specifier) != "o")
                || (mode == NumberMode::Hexadecimal && (*specifier) != "x")
            {
                return false;
            }
            let mut underscore_index = 2;

            if let Some(underscore) = graphemes.get(underscore_index) {
                if *underscore == "_" {
                    while underscore_index < graphemes.len() {
                        underscore_index += 1;

                        if let Some(s) = graphemes.get(underscore_index) {
                            if *s != "_" {
                                break;
                            }
                        }
                    }

                    for _ in 0..underscore_index {
                        self.highlighting.push(Type::Number);
                    }
                    *index = (*index).saturating_add(underscore_index);
                } else {
                    if !(*underscore).chars().next().unwrap_or(' ').is_ascii_digit() {
                        return false;
                    }
                    for _ in 0..2 {
                        self.highlighting.push(Type::Number);
                    }
                    *index = (*index).saturating_add(2);

                }

                if !self.highlight_decimal(options, word, streak, index, mode) {
                    let mut x = 0;
                    while (*index).saturating_sub(last_index.saturating_add(x)) > 0 {
                        self.highlighting.pop();
                        x += 1;
                    }
                    return false;
                }
            } else {
                return false;
            }

        } else {
            self.highlighting.push(Type::Number);
            *index = (*index).saturating_add(1);
            return true;
        }

        true
    }

    pub fn highlight_binary(
        &mut self,
        options: &HighlightingOptions,
        word: &Option<String>,
        streak: &mut HighlightStreak,
        index: &mut usize,
    ) -> bool {
        self.highlight_o_or_b_number(options, word, streak, index, NumberMode::Binary)
    }

    pub fn highlight_hex(
        &mut self,
        options: &HighlightingOptions,
        word: &Option<String>,
        streak: &mut HighlightStreak,
        index: &mut usize,
    ) -> bool {
        self.highlight_o_or_b_number(options, word, streak, index, NumberMode::Hexadecimal)
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
                            if (!prev.trim().is_empty() && prev.is_ascii()) && *index != 0 {
                                match prev.chars().next() {
                                    Some(c) => {
                                        if (c.is_ascii_alphanumeric() || c == '_') {
                                            return false;
                                        }
                                    },
                                    _ => {}
                                }
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
                if (*entry).chars().next().unwrap_or(' ').is_ascii_alphanumeric() || (*entry).chars().next().unwrap_or(' ') == '_' {
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
        // let mut og_graphemes : Vec<&str> =  self.string.graphemes(true).collect();
        let mut terminated: bool = false;

        for cluster in &graphemes[..] {
            if *index_cpy == *index { // opening quote
                *index_cpy += 1;
                continue;
            }
            if *cluster == "'" {
                // TODO: thinking of backtracking

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
        let og_graphemes: Vec<&str> = self.string.graphemes(true).collect();

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
                   } else {
                       let mut back_track = 0;
                       let mut escape_pos = (*index_cpy).saturating_sub(1);
                       loop {
                           if let Some(slash) = og_graphemes.get(escape_pos) {
                               match *slash {
                                   "\\" => {
                                       back_track += 1;
                                   },
                                   _ => {
                                       break;
                                   }
                               }
                           } else {
                               break;
                           }
                           if escape_pos == 0 { break; }
                           escape_pos -= 1;
                       }

                       if back_track % 2 == 0 {
                           *index_cpy += 1;
                           break;
                       }
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
                        *index_cpy += 2;
                        streak.comment = streak.comment.saturating_sub(1);
                        continue;
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_decimal_parsing() {
        let mut new_row = Row::default();
        new_row.string = "32___f64".to_owned();
        let ref mut index = 0;
        assert_eq!(new_row.highlight_decimal(&HighlightingOptions::default(), &None, &mut HighlightStreak::default(), index, NumberMode::None), true);
        assert!(new_row.highlighting.len() == 8);

        // new_row.string = "2_.5e-32___f64".to_owned();
        // new_row.highlighting = Vec::new();
        // let ref mut index = 0;
        // assert_eq!(new_row.highlight_decimal(&HighlightingOptions::default(), &None, &mut HighlightStreak::default(), index, NumberMode::None), true);
        // assert!(new_row.highlighting.len() == 14);

        new_row.string = "02.0e2__f64".to_owned();
        new_row.highlighting = Vec::new();
        let ref mut index = 0;
        assert_eq!(new_row.highlight_decimal(&HighlightingOptions::default(), &None, &mut HighlightStreak::default(), index, NumberMode::Float), true);
        assert!(new_row.highlighting.len() == 2);

        new_row.string = "5e-32___f64".to_owned();
        new_row.highlighting = Vec::new();
        let ref mut index = 0;
        assert_eq!(new_row.highlight_decimal(&HighlightingOptions::default(), &None, &mut HighlightStreak::default(), index, NumberMode::Float), true);
        assert!(new_row.highlighting.len() == 11);

        new_row.string = "5e32___f64".to_owned();
        new_row.highlighting = Vec::new();
        let ref mut index = 0;
        assert_eq!(new_row.highlight_decimal(&HighlightingOptions::default(), &None, &mut HighlightStreak::default(), index, NumberMode::Float), true);
        assert!(new_row.highlighting.len() == 10);
    }

    #[test]
    fn test_float_parsing() {
        let mut new_row = Row::default();
        new_row.string = "2_.5e-32___x64".to_owned();
        new_row.highlighting = Vec::new();
        let ref mut index = 0;
        assert_eq!(new_row.highlight_float(&HighlightingOptions::default(), &None, &mut HighlightStreak::default(), index), false);
        assert!(new_row.highlighting.len() == 0);
    }

    #[test]
    fn test_octal_parsing() {
        let mut new_row = Row::default();
        new_row.string = "0".to_owned();
        new_row.highlighting = Vec::new();
        let ref mut index = 0;
        assert_eq!(new_row.highlight_octal(&HighlightingOptions::default(), &None, &mut HighlightStreak::default(), index), true);
        assert!(new_row.highlighting.len() == 1);

        new_row.string = "0o_101010_x32".to_owned();
        new_row.highlighting = Vec::new();
        let ref mut index = 0;
        assert_eq!(new_row.highlight_octal(&HighlightingOptions::default(), &None, &mut HighlightStreak::default(), index), false);
        assert!(new_row.highlighting.len() == 0);

        new_row.string = "0o_201010_i32;".to_owned();
        new_row.highlighting = Vec::new();
        let ref mut index = 0;
        assert_eq!(new_row.highlight_octal(&HighlightingOptions::default(), &None, &mut HighlightStreak::default(), index), true);
        assert!(new_row.highlighting.len() == 13);

        new_row.string = "0o701010_i32;".to_owned();
        new_row.highlighting = Vec::new();
        let ref mut index = 0;
        assert_eq!(new_row.highlight_octal(&HighlightingOptions::default(), &None, &mut HighlightStreak::default(), index), true);
        assert!(new_row.highlighting.len() == 12);

        new_row.string = " 0o_801010_i32;".to_owned();
        new_row.highlighting = Vec::new();
        let ref mut index = 1;
        assert_eq!(new_row.highlight_octal(&HighlightingOptions::default(), &None, &mut HighlightStreak::default(), index), true);
        assert!(new_row.highlighting.len() == 13);

        new_row.string = "0o___101010_i32".to_owned();
        new_row.highlighting = Vec::new();
        let ref mut index = 0;
        assert_eq!(new_row.highlight_octal(&HighlightingOptions::default(), &None, &mut HighlightStreak::default(), index), true);
        assert!(new_row.highlighting.len() == 15);
    }

    #[test]
    fn test_binary_parsing() {
        let mut new_row = Row::default();
        new_row.string = "0".to_owned();
        new_row.highlighting = Vec::new();
        let ref mut index = 0;
        assert_eq!(new_row.highlight_binary(&HighlightingOptions::default(), &None, &mut HighlightStreak::default(), index), true);
        assert!(new_row.highlighting.len() == 1);

        new_row.string = "0b_101010_x32".to_owned();
        new_row.highlighting = Vec::new();
        let ref mut index = 0;
        assert_eq!(new_row.highlight_binary(&HighlightingOptions::default(), &None, &mut HighlightStreak::default(), index), false);
        assert!(new_row.highlighting.len() == 0);

        new_row.string = "0b_201010_i32;".to_owned();
        new_row.highlighting = Vec::new();
        let ref mut index = 0;
        assert_eq!(new_row.highlight_binary(&HighlightingOptions::default(), &None, &mut HighlightStreak::default(), index), true);
        assert!(new_row.highlighting.len() == 13);

        new_row.string = "0b701010_i32;".to_owned();
        new_row.highlighting = Vec::new();
        let ref mut index = 0;
        assert_eq!(new_row.highlight_binary(&HighlightingOptions::default(), &None, &mut HighlightStreak::default(), index), true);
        assert!(new_row.highlighting.len() == 12);

        new_row.string = " 0b_801010_i32;".to_owned();
        new_row.highlighting = Vec::new();
        let ref mut index = 1;
        assert_eq!(new_row.highlight_binary(&HighlightingOptions::default(), &None, &mut HighlightStreak::default(), index), true);
        assert!(new_row.highlighting.len() == 13);

        new_row.string = "0b___101010_i32".to_owned();
        new_row.highlighting = Vec::new();
        let ref mut index = 0;
        assert_eq!(new_row.highlight_binary(&HighlightingOptions::default(), &None, &mut HighlightStreak::default(), index), true);
        assert!(new_row.highlighting.len() == 15);
    }

    #[test]
    fn test_hex_parsing() {
        let mut new_row = Row::default();
        new_row.string = "0".to_owned();
        new_row.highlighting = Vec::new();
        let ref mut index = 0;
        assert_eq!(new_row.highlight_hex(&HighlightingOptions::default(), &None, &mut HighlightStreak::default(), index), true);
        assert!(new_row.highlighting.len() == 1);

        new_row.string = "0x_101010_x32".to_owned();
        new_row.highlighting = Vec::new();
        let ref mut index = 0;
        assert_eq!(new_row.highlight_hex(&HighlightingOptions::default(), &None, &mut HighlightStreak::default(), index), false);
        assert!(new_row.highlighting.len() == 0);

        new_row.string = "0x2;".to_owned();
        new_row.highlighting = Vec::new();
        let ref mut index = 0;
        assert_eq!(new_row.highlight_hex(&HighlightingOptions::default(), &None, &mut HighlightStreak::default(), index), true);
        assert!(new_row.highlighting.len() == 3);

        new_row.string = "0x701010_i32;".to_owned();
        new_row.highlighting = Vec::new();
        let ref mut index = 0;
        assert_eq!(new_row.highlight_hex(&HighlightingOptions::default(), &None, &mut HighlightStreak::default(), index), true);
        assert!(new_row.highlighting.len() == 12);

        new_row.string = " 0x_801010_i32;".to_owned();
        new_row.highlighting = Vec::new();
        let ref mut index = 1;
        assert_eq!(new_row.highlight_hex(&HighlightingOptions::default(), &None, &mut HighlightStreak::default(), index), true);
        assert!(new_row.highlighting.len() == 13);

        new_row.string = "0x___10e1010_i32".to_owned();
        new_row.highlighting = Vec::new();
        let ref mut index = 0;
        assert_eq!(new_row.highlight_hex(&HighlightingOptions::default(), &None, &mut HighlightStreak::default(), index), true);
        assert!(new_row.highlighting.len() == 16);
    }


    // let hx = 0x_82323_i32;
    // let hx = 0x82323e3_i32;
    // let hx = 0x____82323e3_i32;

}