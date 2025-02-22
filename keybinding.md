# Vim Keybinding Reference Guide

## 1. Basic Movement Commands
(Accept single numeric prefix to repeat action)

| Command | Description                                    | Example                               |
|---------|------------------------------------------------|---------------------------------------|
| j       | Move down one line                             | 5j moves down 5 lines                 |
| k       | Move up one line                               | 10k moves up 10 lines                 |
| h       | Move left one character                        | 3h moves left 3 characters            |
| l       | Move right one character                       | 4l moves right 4 characters           |
| w       | Move forward to start of next graph            | 2w moves forward two words            |
| e       | Move forward to end of current/next graph      | 3e moves to end of three words        |
| b       | Move backward to start of previous graph       | 5b moves back five words              |
| ge      | Move backward to end of previous graph         | 2ge moves back two word endings       |
| {       | Move up to start of previous paragraph         | 3{ moves up three paragraphs          |
| }       | Move down to start of next paragraph           | 2} moves down two paragraphs          |
| (       | Move back to start of current/previous sentence| 2( moves back two sentences           |
| )       | Move forward to start of next sentence         | 3) moves forward three sentences      |
| n       | Repeat last search forward                     | 4n repeats search 4 times             |
| N       | Repeat last search backward                    | 2N repeats search 2 times             |
| f[char] | Jump to the next occurrence of [char] on d line| 2f[char] Jumps to the next 2          |


## 1b. Special movement commands
(doesn't accept single numeric prefix)
|  M      | Moves to the middle of the screen                             |                                       |
|  L      | Moves to the bottom of the screen                             |                                       |
|  -      | Jumps to the first non blank character on the previous line   |                                       |
|  _      | Jumps to the first non blank character on the line            |                                       |
|  g_     | Jumps to the last non blank character on the line             |                                       |
| gm      | Move to the middle of the screen or end of current line       |                                       |
| zz      | Scroll current line to the middle of the screen               |                                       |
| zt      | Scroll current line to the top    of the screen               |                                       |
| zb      | Scroll current line to the bottom of the screen               |                                       |
| Ctrl f  | move screen down one page (cursor on the first line)          |                                       |
| Ctrl b  | move screen up   one page (cursor on the last  line)          |                                       |
| Ctrl u  | move screen and cursor up 1/2 of the page                     |                                       |
| Ctrl d  | move screen and cursor down 1/2 of the page                   |                                       |

## 1c. Insert commands
|  key    |             features                                          |        is quantifiable?               |

|  I      | insert at the beginning of the line                           |            yes                        |
|  i      | insert before cursor                                          |            yes                        |
|  a      | insert after cursor                                           |            yes                        |
|  A      | insert at the end of the line                                 |            yes                        |
|  o      | insert a new line after  the current line                     |            yes                        |
|  O      | insert a new line before the current line                     |            yes                        |

## 1d. Edit commands
|  key    |             features                                          |        is quantifiable?               |

|  r      | replace a single character                                    |             yes                       |
|  R      | enters replace mode until esc is pressed                      |             yes                       |
|  J      | joins with the line below                                     |             yes                       |
|  gJ     | joins with the line below without space in between            |             yes                       |
|  C      | change the rest of the line                                   |             yes                       |
|  S      | delete the entire line and enter insert mode                  |             yes                       |
|  s      | delete selection and enter insert mode                        |             yes                       |
|  c[nav] | change up to navigation  (e.g) cl changes up to the next char |             yes                       |
|  u      | undo                                                          |             yes                       |
|  Ctrl+r | redo                                                          |             yes                       |
|  .      | repeat the last edit command                                  |             yes                       |
|  ~      | toggle case                                                   |             yes                       |


## 2. Visual Mode Commands
(Do not accept numeric prefixes)


| Command | Description                                 |
|---------|---------------------------------------------|
| v       | Enter visual mode (character-wise)          |
| V       | Enter visual mode (line-wise)               |
| Ctrl-v  | Enter visual block mode (column selection)  |
| gv      | Reselect last visual selection              |
|  o      | move to other end of the marked area        |
| vi[]    | select in between a text/character class    |
| va[]    | select around  a text/character class       |

## 3. Search Commands

| Command | Description                            |
|---------|----------------------------------------|
| *       | Search forward for word under cursor   |
| #       | Search backward for word under cursor  |

## 4. Single-Quantifiable Commands

| Command | Description                        | Example                           |
|---------|------------------------------------|-----------------------------------|
| v3w     | Select 3 words forward             | v5e selects to end of 5 words     |
| V5j     | Select 5 lines down                |                                   |
| >       | Indent selection right             | 3>> indents 3 lines               |
| <       | Indent selection left              | 2<< un-indents 2 lines            |

## 5. Text Object Commands

### Basic Structure
- **v + text-object** → Select (Visual mode)
- **c + text-object** → Change (Delete + Insert mode)
- **d + text-object** → Delete
- **y + text-object** → Yank (Copy)

### (A) Non-Quantifiable Text Object Commands

| Command | Target               |  Operation                               |
|---------|----------------------|------------------------------------------|
| viw     | Inner Word           | Select content without spaces            |
| ciw     | Inner Word           | Change content without spaces            |
| vi(     | Inside Parentheses   | Select content inside ()                 |
| ci[     | Inside Brackets      | Change content inside []                 |
| vi<     | Inside Angles        | Select content inside <>                 |
| vip     | Inner Paragraph      | Select paragraph content                 |
| di[]    | Inner Paragraph      | deletes contents inside capture          |
| da[]    | Inner Paragraph      | deletes contents around capture          |
| yi[]    | Inner Paragraph      | yanks contents inside capture            |
| ya[]    | Inner Paragraph      | yanks contents around capture            |

### (B) Single-Quantifiable Text Object Commands

| Command | Description                              | Example                    |
|---------|------------------------------------------|----------------------------|
| 3diw    | Delete 3 words as separate text objects  |                            |
| 2ci(    | Change inside 2 sets of nested parens    |                            |


### (C) "Inner" vs "Around" Text Objects

| Modifier | Description                              |
|----------|------------------------------------------|
| i (inner)| Affects only content inside delimiters   |
| a (around)| Includes delimiters and whitespace      |

### Common Text Object Patterns

| Pattern | Example Commands                         |
|---------|------------------------------------------|
| Word    | viw, ciw, diw, yiw                       |
| Sentence| vis, cis, dis, yis                       |
| Paragraph| vip, cip, dip, yip                      |
| Quotes  | vi", ci", di", yi"                       |
| Brackets| vi], ci], di], yi]                       |

## 6. Line Folding
| Command | Description                              | Example                    |
|---------|------------------------------------------|----------------------------|
| zf      | fold up to navigation                    |                            |
| zo      | open fold on current line                |                            |

#### Key Takeaways
- Text objects enable smart, context-aware editing
- Inner (i) modifier works on content only
- Around (a) modifier includes delimiters
- Most commands support numeric prefixes
- Mastering these commands significantly improves editing efficiency




# Vim Cheat Sheet

## Global

- `:h[elp] keyword` - open help for keyword
- `:sav[eas] file` - save file as
- `:clo[se]` - close current pane
- `:ter[minal]` - open a terminal window
- `K` - open man page for word under the cursor

*Tip*: Run `vimtutor` in a terminal to learn the first Vim commands.

## Cursor movement

- `h` - move cursor left ------<Done>
- `j` - move cursor down ------<Done>
- `k` - move cursor up ------<Done>
- `l` - move cursor right ------<Done>

[//]: # (- `gj` - move cursor down &#40;multi-line text &#41;)
[//]: # (WE ARE NOT SUPPORTING MULTILINE NAVIGATION)
[//]: # (- `gk` - move cursor up &#40;multi-line text&#41;)
- `H` - move to top of screen ------<Done>
- `M` - move to middle of screen ------<Done>
- `L` - move to bottom of screen ------<Done>
- `w` - jump forwards to the start of a word ------<Done>

[//]: # ( merged - `W` with `w` - )

- `e` - jump forwards to the end of a word

[//]: # ( merged - `E` with `e` - )

- `b` - jump backwards to the start of a word

[//]: # ( merged - `B` with `b` - )

- `ge` - jump backwards to the end of a word ------<Done>

[//]: # ( merged - `gE` with `ge` - )

- `%` - move cursor to matching character (default supported pairs: '()', '{}', '[]' - use `:h matchpairs` in vim for more info)
- `0` - jump to the start of the line ------<Done>
- `^` - jump to the first non-blank character of the line
- `$` - jump to the end of the line ------<Done>
- `g_` - jump to the last non-blank character of the line
- `gg` - go to the first line of the document ------<Done>
- `G` - go to the last line of the document ------<Done>
- `5gg` or `5G` - go to line 5 ------<Done>
- 
- `gd` - move to local declaration
- `gD` - move to global declaration
- `fx` - jump to next occurrence of character x
- `tx` - jump to before next occurrence of character x
- `Fx` - jump to the previous occurrence of character x
- `Tx` - jump to after previous occurrence of character x
- `;` - repeat previous f, t, F or T movement
- `,` - repeat previous f, t, F or T movement, backwards
- `}` - jump to next paragraph (or function/block, when editing code)
- `{` - jump to previous paragraph (or function/block, when editing code)
- `zz` - center cursor on screen
- `zt` - position cursor on top of the screen
- `zb` - position cursor on bottom of the screen
- `Ctrl` + `e` - move screen down one line (without moving cursor)
- `Ctrl` + `y` - move screen up one line (without moving cursor)
- `Ctrl` + `b` - move screen up one page (cursor to last line)
- `Ctrl` + `f` - move screen down one page (cursor to first line)
- `Ctrl` + `d` - move cursor and screen down 1/2 page
- `Ctrl` + `u` - move cursor and screen up 1/2 page

*Tip*: Prefix a cursor movement command with a number to repeat it. For example, `4j` moves down 4 lines.

## Insert mode - inserting/appending text

- `i` - insert before the cursor
- `I` - insert at the beginning of the line
- `a` - insert (append) after the cursor
- `A` - insert (append) at the end of the line
- `o` - append (open) a new line below the current line
- `O` - append (open) a new line above the current line
- `ea` - insert (append) at the end of the word
- `Ctrl` + `h` - delete the character before the cursor during insert mode
- `Ctrl` + `w` - delete word before the cursor during insert mode
- `Ctrl` + `j` - add a line break at the cursor position during insert mode
- `Ctrl` + `t` - indent (move right) line one shiftwidth during insert mode
- `Ctrl` + `d` - de-indent (move left) line one shiftwidth during insert mode
- `Ctrl` + `n` - insert (auto-complete) next match before the cursor during insert mode
- `Ctrl` + `p` - insert (auto-complete) previous match before the cursor during insert mode
- `Ctrl` + `rx` - insert the contents of register x
- `Ctrl` + `ox` - Temporarily enter normal mode to issue one normal-mode command x.
- `Esc` or `Ctrl` + `c` - exit insert mode

## Editing

- `r` - replace a single character.
- `R` - replace more than one character, until `ESC` is pressed.
- `J` - join line below to the current one with one space in between
- `gJ` - join line below to the current one without space in between
- `gwip` - reflow paragraph
- `g~` - switch case up to motion
- `gu` - change to lowercase up to motion
- `gU` - change to uppercase up to motion
- `cc` - change (replace) entire line
- `c$` or `C` - change (replace) to the end of the line
- `ciw` - change (replace) entire word
- `cw` or `ce` - change (replace) to the end of the word
- `s` - delete character and substitute text (same as cl)
- `S` - delete line and substitute text (same as cc)
- `xp` - transpose two letters (delete and paste)
- `u` - undo
- `U` - restore (undo) last changed line
- `Ctrl` + `r` - redo
- `.` - repeat last command

