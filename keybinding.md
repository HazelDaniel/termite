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
