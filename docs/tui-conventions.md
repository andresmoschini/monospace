# Terminal interface conventions

<!-- The words below are identifiers quoted out of ncurses, vim, Emacs, dialog and Midnight
     Commander, and nothing else in this repository spells them. They are ignored here rather than
     added to project-words.txt because a global dictionary entry makes the rest of the repository
     silently accept a misspelling of it forever, and none of these is a word this project writes.
     cspell:ignore ALNUM affordances BUTTONBAR buttonbar checkflag cmdheight cmdline codepoint
     cspell:ignore ctlseqs defaultitem Elisp elisp ESCDELAY fboundp filemanager fselect geolocation
     cspell:ignore halfdelay hline inputbox keymap laststatus ltlines LEFTMIDDLE lipgloss menuhot menuhotsel menusel
     cspell:ignore Minibuffer minibuffer mysession nodelay NILP passwordbox RIGHTMIDDLE reselects
     cspell:ignore shortmess showmode SIGWINCH stdscr terminfo textinput tolower treeview vcur
     cspell:ignore Vinhibit waddstr wcwidth xdisp -->

**Status:** Survey, not decisions. This document records what the established terminal applications
do and which source fixed each thing. **It does not decide anything.** Where this repository follows
a convention, the specification that follows it says so; where this repository departs from one, the
specification that departs says so and says why; and where nothing has been decided yet, that is an
open question in the specification, not a gap here. Nothing in this document may be cited as though
it answered anything.

**Researched:** 2026-10-10, from manual pages and from source. **Adapted into `docs/`:** 2026-10-11,
in English, with the framing changed from a set of rules to follow into a record of what the sources
say. The conclusions are the researcher's; where a claim could not be verified it is in §13 and it
is asserted nowhere else.

**Why it exists.** There is no TUI style guide. What there is, is four different things that are
routinely confused, and only the second of them is what this document is about:

| Layer                      | What it is                                    | Where the rule lives               |
| -------------------------- | --------------------------------------------- | ---------------------------------- |
| Command line interface     | arguments, `--help`, exit codes               | POSIX, GNU §5, Fedora              |
| **On-screen presentation** | boxes, focus, bars, shortcuts                 | **this document**                  |
| Terminal capabilities      | which sequences exist, which colors there are | `terminfo`, xterm ctlseqs, wcwidth |
| Frameworks                 | the concrete widgets of one library           | its own documentation              |

The second is the one written down nowhere and inherited by imitation. Four sources fixed it, in
order of historical weight:

1. **ncurses** — the reference library, and its `menu(3x)` fixes the semantics of focus and of the
   mark.
2. **Midnight Commander** — the ancestor of Turbo Vision, and the most complete example of a screen
   composed of regions.
3. **dialog / whiptail** — the exact shape of the box and of the button.
4. **Emacs, less, tmux, vim** — each fixed one piece: the status line, the command line, arrow-key
   navigation, the `%t` of truncation.

The modern frameworks added almost nothing to this layer, and that is a finding rather than an
absence: §4.6.

## 1. The three layers of a line

Before deciding what goes on each row of the screen, three distinct mechanisms have to be decided,
which almost every tool implements separately and which are constantly confused:

| Layer            | Lifetime                | Owner            | What it must **not** carry            | Examples                                                                   |
| ---------------- | ----------------------- | ---------------- | ------------------------------------- | -------------------------------------------------------------------------- |
| **Chrome**       | the whole application   | the widget tree  | nothing the user can type             | MC's button bar, vim's `statusline`, tmux's `status`                       |
| **Message**      | seconds, expires alone  | a timer          | nothing needing an ack                | tmux's `display-message` (750 ms), Textual's `Toast`, vim's `-- INSERT --` |
| **Command line** | while it is being typed | the input widget | nothing but `text + flags + position` | MC's shell line, `less`'s `:` prompt, vim's `cmdline`                      |

Two concrete rules come out of this and both are documented:

- **tmux** puts the message and the prompt on **separate lines of the same bar**, and makes it
  configurable: `message-line` (`0` by default) is _"Position (line) of messages and the command
  prompt"_.
- **vim** makes the status line conditional rather than fixed: `'laststatus'` is `0` never, `1`
  _"only if there are at least two windows"_, `2` always. And `'cmdheight'` is `1` by default, with
  the note _"A larger value helps avoiding hit-enter prompts"_.

> **Design consequence:** if one row carries both an error message and an interactive menu, one of
> the two overwrites the other. `tig` resolves it by separating them: each view's **title line** is
> persistent and carries identity and position, and the **status line** is a single transient row
> carrying messages and, while it is needed, the options menu.

## 2. The anatomy of a full screen

The canonical example — MC, 80 columns — with the regions labelled:

```text
  Left   File   Command   Options   Right      ← row 0, the menu
┌─<─ ~/src ────────────────────────── .[^]>──┐
│> README.md       │ 4.0K │ Oct20 12:00│< /etc ──── .[^]>─┐
│  Makefile        │ 1.1K │ Oct 20 12:00│> ../             │
│  src             │ 512B │ Oct 19 09:31│> Makefile        │
│                  12.0K in 3 files            │               │
│   README.md │ 4.0K │ -rw-r--r--    UP--DIR  1.2G/16G (7%) │
└────────────────────────────────────────────┴───────────────┘
  ~/src$ git status --short                   ← command line
  1Help   2Menu   3View   4Edit   5Copy  6RenMov 7Mkdir 8Delete 9PullDn 10Quit
```

`mc(1)` describes it as four parts: _"The screen of Midnight Commander is divided into four parts"_,
with the command line _"the second line from the bottom"_ and the key labels _"the bottom line"_,
and _"The topmost line is the menu bar line"_, which _"may not be visible, but appears if you click
the topmost line with the mouse or press the F9 key"_.

The four rows at the bottom are, from the inside out:

- **mini-status**: one line of information about the item under the cursor.
- **command line**: the shell input, with its prompt.
- **hint bar**: help text that rotates; MC picks one at random and refreshes it at most every 60 s.
- **button bar**: `F1`–`F10`, at `LINES-1`.

## 3. The drop-down menu

### 3.1 The title row

MC draws five titles, each as **one space + title + one space**, starting at column 1, and the gap
between them is **clamped**:

```text
  Left   File   Command   Options   Right
 ^   ^ ^   ^ ^ ^     ^   ^     ^   ^     ^
 │   │ │   │ │     │     │   │     │   │     │   └── the title's trailing space
 │   │ │   │ │     │     │   │     │   │     └────── gap of 3 columns (the clamp's top)
 └─ col 1       col 10        col 19       col 31       col 43
```

The arithmetic is in `lib/widget/menu.c`, `menubar_arrange`:
`gap = (COLS - 2 - Σ(title+2)) / (n-1)`, and if `gap <= 0` it is `1`, if `gap >= 3` it is `3`. **The
gap is never less than 1 and never more than 3.**

Emacs uses the opposite and simpler criterion: one space at the end of each title, none at the start
(`display_menu_bar()`: _"Display the item, pad with one space"_).

```text
File Edit Buffers Tools Help          ← Emacs on a TTY, one trailing space
 File Edit  Buffers  Tools  Help     ← the toolkit rendering, for contrast
```

### 3.2 The drop-down box

MC places the box with exact geometry, in `lib/widget/menu.c`:

- The left border is at `start_x - 1`: **one column to the left of the title's first letter**.
- Width = `max_entry_len + 4`; height = the number of entries plus two.
- **There is no padding between `│` and the first letter**: the label starts in the first interior
  column.
- The shortcut column is at interior offset `max_label_width + 3`, that is, **exactly three blank
  columns** between the longest label and the shortcuts, and **one column** before the right border.
- The hardware cursor rests at `start_x + 1`, inside the item.
- The box is **clipped against the edge of the screen** if it does not fit.
- The shadow is optional, and is an appearance option.

```text
  Left   File   Command   Options   Right
         ┌──────────────────────────┐
         │View                     F3│
         │View file...              │
         │Filtered view        M-!  │
         │Edit                     F4│
         │Chmod                C-x c│
         │Rename/Move               F6│
         │Mkdir                     F7│
         │Delete                    F8│
         ├──────────────────────────┤
         │Select group             + │
         │Invert selection         * │
         ├──────────────────────────┤
         │Exit                    F10│
         └──────────────────────────┘ ▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒
```

The separator is `├───…───┤`: `LEFTMIDDLE` + `max_entry_len + 2` horizontals + `RIGHTMIDDLE`, on the
same columns as the borders.

**What is underneath is not composed, it is regenerated.** `menubar_finish()` calls a full
`do_refresh()`. Emacs does it properly and is the model to copy (`src/term.c`):

> _"the previous contents of the screen […] is stashed away, and used to restore screen contents
> when the menu selection changes or when the final selection is made"_

and additionally `tty_hide_cursor()` while the menu is open, `Vinhibit_redisplay` so that no timer
repaints underneath, and `discard_mouse_events()` on close.

### 3.3 Mnemonics

**The marker is `&` before the letter**, and the letter is drawn in a different color. Never
brackets, never underline.

MC parses `&` in `hotkey_new()`, storing `start` / `hotkey` / `end`, and `hotkey_draw()` paints the
`hotkey` with the shortcut color. The defaults in `misc/skins/default.ini`:

```ini
[menu]         menuhot      = yellow;cyan     ; mnemonic, unselected row
               menuhotsel  = yellow;black    ; mnemonic, selected row
               menusel     = white;black     ; selected row
```

**The bare letter fires, without `Alt`.** `menubar_try_exec_menu()` compares
`hotkey == g_ascii_tolower(text.hotkey[0])`. The documented exception, from the ArchWiki: inside a
text field the bare letter is typed, so there it is `Alt`+letter.

Turbo Vision, MC's ancestor, used `~X~` instead of `&`. Same language, different paint.

### 3.4 Separators, submenus and Emacs's two glyphs

**MC has no submenus**: each drop-down is a flat list and a `NULL` item is a separator.

Emacs has them, and its two glyphs are the reference for the genre:

- **`>`** — two characters, a space and the angle. It is the submenu cue, and only on a TTY
  (`src/menu.c`: `if (is_tty_frame(...) && !NILP(map)) item_string = concat2(item_string, " >")`).
- **`...`** — _"A `'...'` at the end of a menu item means that the command will prompt you for
  further input before it actually does anything."_
- The separator is **a run of hyphens**, not a box-drawing line: `display_tty_menu_separator()`
  writes `width-1` copies of `-` (or `=` for `--:double-line`).
- Every item carries **one column of left padding**, and the drop-down **has no frame**.

```text
File Edit Buffers Tools Help
 Edit Edit Buffers Tools Help
       ▸ Fund Editor       ▸ Compare
       ▸ Tags
     Buffers
       foo.txt        *
       # scratch#
```

The corresponding Elisp, which is the most precise specification of a menu that exists:

```elisp
'(menu-item "Quit" save-buffers-kill-terminal :help "Save unsaved buffers, then exit")
'(menu-item "Print" ,menu-bar-print-menu)          ; the 2nd element is a keymap ⇒ submenu
'(menu-item "Undo Last External Command" vc-undo-command
           :enable (fboundp 'vc-undo-command))     ; :enable ⇒ enabled/disabled
'(menu-item separator-print)                        ; "a name that starts with two hyphens"
```

> The order of the items is the order of the keymap, and `define-key` **pushes to the front** —
> which is why Emacs defines its menus from the bottom up.

### 3.5 The menu's keyboard

The most transferable fact in this document, because MC and Emacs agree exactly:

| Action                    | MC                    | Emacs (TTY)           |
| ------------------------- | --------------------- | --------------------- |
| Open                      | `F9`                  | `F10`                 |
| Change menu               | `←` `→` (`C-b` `C-f`) | `←` `→` (`C-b` `C-f`) |
| Move inside the drop-down | `↑` `↓` (`C-p` `C-n`) | `↑` `↓` (`C-p` `C-n`) |
| Activate                  | `Enter`               | `RET`                 |
| Cancel                    | `Esc`, `F10`, `C-g`   | `C-g`, `ESC ESC ESC`  |

**`←`/`→` change menu; `↑`/`↓` move inside.** It is what the user wants and it is what both tools
do.

ncurses `menu(3x)` **has no menubar**: the title row is another `MENU` posted over `stdscr`, and
`E_REQUEST_DENIED` at the end is the wrap signal. Its own demo says so, `test/demo_menus.c`:

```c
set_menu_mark(mpBanner, ">");            /* the title row uses ">" as the mark */
code = menu_driver(mpBanner, cmd);
if (code == E_REQUEST_DENIED) {          /* reached the end ⇒ wrap */
    if (menu_number() > 0) code = menu_driver(mpBanner, REQ_FIRST_ITEM);
    else                   code = menu_driver(mpBanner, REQ_LAST_ITEM);
}
```

`menu(3x)` **also does not handle mnemonics**: that belongs to the application, always.

### 3.6 What the modern frameworks do instead

Verified against each one's widget registry, not against blog posts:

| Framework        | Has a menu?     | What it puts there                                                                                                                                                                                                                                                            |
| ---------------- | --------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **ratatui**      | no              | `Block::bordered()` plus `Clear` over a `Rect` from `area.centered(...)`, drawn **last**. The draw order is the z-order, and its own popup example says so                                                                                                                    |
| **Bubble Tea**   | no              | nothing: it is a runtime. The components live in the sibling module `bubbles` (Spinner, TextInput, TextArea, Table, Progress, Paginator, Viewport, List, FilePicker, Timer, Stopwatch, Help, Key) — no menu                                                                   |
| **Textual**      | no              | `Header` (icon, centred title, clock) plus a shortcut `Footer`, and `HelpPanel` / `KeyPanel`                                                                                                                                                                                  |
| **tview**        | no, by decision | the maintainer in issue #86: _"the `List` primitive was implemented as a kind of menu. That's why it has shortcuts […] I don't see the difference between this and a `List`"_. It uses `List` inside `Pages`/`Grid` and `SetInputCapture()` for global `Alt`+letter shortcuts |
| **Turbo Vision** | yes, complete   | the ancestor: `Alt+F` and `F10`, `~X~`, cascading with `→`, the shortcut column on the right                                                                                                                                                                                  |

**The honest conclusion:** since 2020 the menubar was given up in favour of lists and shortcuts. The
exception is that **a contextual action needs a name on screen**, and what there is for that is the
help overlay, not the drop-down menu.

## 4. Key bars and toolbars

### 4.1 Persistent, transient or absent

The evidence splits cleanly along whether the set of keys is global or contextual:

|                                      | Tool                                                                | When                                                                                                                                                                               |
| ------------------------------------ | ------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Persistent row at the bottom**     | MC                                                                  | the set is small (≤10), global, and **changes with the focused widget**: MC reassigns the same ten labels per context, and that is only possible because the row is always visible |
| **Transient row in the status line** | tig's options menu, Emacs's minibuffer, fzf's `--header`/`--footer` | the content is a message, a prompt, or a short list that exists while something is happening                                                                                       |
| **Absent**                           | dialog/whiptail, and fzf's `--expect`                               | modal dialogs put the actions in buttons                                                                                                                                           |
| **Its own region**                   | tig's help (`h`)                                                    | when the list of shortcuts is long enough to need scrolling                                                                                                                        |

**The rule:** a key bar is justified when the same ≤10 keys serve the whole application. If it is
contextual, the row has to be **reassigned and redrawn**, not decorated.

### 4.2 The shape

MC draws `%2d` — the key number right-aligned in two columns — followed by the label. **It does not
use brackets.** The `[F1]Help [F2]Menu` shape that circulates in third-party tutorials is not what
the code draws.

```text
 1Help  2Menu  3View  4Edit  5Copy  6RenMov  7Mkdir  8Delete  9PullDn 10Quit
```

At 64 columns:

```text
 1Help 2Menu 3View 4Edi 5Cop 6RenM 7Kid 8Dele 9Pull 10Qui
```

At 50 columns — `COLS < BUTTONBAR_LABELS_NUM * 7`, that is, under 70:

```text
 1Help 2Menu 3View 4Edit 5Copy 6RenMo 7Mkdir
```

### 4.3 The rules, with their line numbers

From `lib/widget/buttonbar.c` and `buttonbar.h`:

1. **Minimum 7 columns per button.** The label is **truncated**, not wrapped:
   `str_fit_to_term(text, width - 2, J_LEFT_FIT)`.
2. **If it does not fit, it falls off the right.** The loop is `if (pos + 7 <= COLS) pos += 7;` and
   the drawing stops with `break` as soon as the width is ≤ 0. At 50 columns `Quit` is simply not
   there.
3. **Never a second row.** There is no code for it.
4. **With columns to spare, they are distributed**, in a deliberate order so that the central seam
   of the bar lands on the divider between the two panels. The code's comment says so: _"Distribute
   the extra width in a way that the middle vertical line (between F5 and F6) aligns with the two
   panels. The extra width is distributed in this order: F10, F5, F9, F4, …, F6, F1."_
5. **The key has its own attribute.** `[buttonbar] hotkey = white;black` / `button = black;cyan`.
6. **It is clickable.** `mc(1)`: _"it is possible to execute the commands assigned to the function
   key labels by clicking on them."_
7. **Every function key has a fallback without a function key.** `mc(1)`: _"On terminals without
   function key support, you can achieve the same functionality by pressing the Esc key and then a
   number in the range 1 through 9 and 0."_

The contrast with tig is instructive: tig **has no persistent key bar**. The shortcuts live in a
help view with collapsible sections, and the options menu is drawn in the status line:

```text
Toggle option date display [d]                                                (3 of 16)
```

Navigating that menu, from the code: `←`/`↑` previous (with wrap), `→`/`↓` next (with wrap), the
shortcut's letter selects immediately, `RET` accepts, `ESC`/`C-c` cancels, and `report_clear()`
erases the line when it ends.

### 4.4 Does the button show its key?

**Yes, always.** There is no exception in any surveyed tool. What changes is **where**:

- **In front, in a fixed-width gutter, right-aligned** — MC (`%2d`), tig's help.
- **Behind** — tig's options menu (`[d]`), Emacs (`Label ==> m`).

For a bar, **in front**: the key column aligns into a legible vertical stripe. Behind only makes
sense in a menu item, where the right-hand column is already taken by label alignment.

## 5. The status line

### 5.1 What it carries

The most coded reference is vim's `'statusline'`, which is a small language:

| Item                | What it shows                                        |
| ------------------- | ---------------------------------------------------- |
| `%f`                | the path as it was written                           |
| `%l`                | line number                                          |
| `%c`                | column in **bytes**                                  |
| `%V`                | visual column (suppressed when equal to `%c`)        |
| `%P`                | percentage of the **window**, _"Always 3 in length"_ |
| `%m` `%r` `%h` `%w` | flags: modified, read-only, help, preview            |

And three layout operators:

- **`%=`** — _"Separation point between alignment sections. Each section will be separated by an
  equal number of spaces."_
- **`%<`** — where to truncate if it is long.
- **`%(` … `%)`** — grouping, so that the group **disappears entirely** if all its items come out
  empty.

That last behavior is the one worth copying:

```text
 file.c [+][RO]      ← the bracketed form
 file.c +RO          ← the comma form, if the group follows plain text
 file.c              ← the group disappears entirely if no flag is set
```

tmux does the same with one character per flag, and with one color per state:

```text
[mysession] 0:bash* 1:vim- 2:nano! 3:vim~ 4:ssh* 5:vimM 6:vimZ     22:14 08-10-26
                       │      │   │   │   │   │
                       │      │   │   │   │   └─ Z: pane active and zoomed
                       │      │   │   │   └───── M: contains the marked pane
                       │      │   │   └───────── ~: silence, by monitor-silence
                       │      │   └───────────── !: there is a bell
                       │      └───────────────── #: activity detected
                       └──────────────────────── -: previous window
                                                *: current window
```

_"The window name is printed in inverted colors if an alert (bell, activity or silence) is
present."_

### 5.2 The scroll position

The most replicated convention and the cheapest one:

- **vim**: `"lines 3-3/42"` and, on the right, `Bot`, `Top`, `All` or `45%`.
- **dialog**: an `NN%` right-aligned **on the bottom border** of the box, at column `right - 7`,
  plus `(-)` above and `(+)` below as affordances.
- **fzf**: `3/42` in the prompt line.
- **MC**: `1.2G / 16G (7%)` in the panel's bottom right corner.

```text
int main(void)                                                     All
{
    printf("hello");                          lines 3-3/42        7%
    return 0;
}
```

## 6. The command line

`less` is the cleanest case because its command line is a **conditional formatter**, and it
documents the exact result:

```text
default (-Ps) :  :

default (-Pm) :  ?n?f%f .?m(%T %i of %m) ..?e(END) ?x- Next\: %x..%t
                 :?pB%pB\%:byte %bB?s/%s..%t

"=" message   :  ?f%f .?m(%T %i of %m) .?ltlines %lt-%lb?L/%L..
                 byte %bB?s/%s. ?e(END) :?pB%pB\%..%t
```

The syntax is pseudo-code: `?` plus a condition, the true branch, `:` introducing the false branch,
`.` closing it. And `%t` is _"Causes any trailing spaces to be removed"_ — **every status line ends
with `%t`**, because right-padding is noise.

Rendered:

```text
 header.c (END)
 #define MAIN  int                            (file 2 of 3)
 int main(void) {                              lines 40-58/310
                                                  byte 1043-1062
     return 0;
 }
```

Three details worth copying:

1. **The default prompt is a bare `:`.** Not `>`, not `:` with text.
2. **`(END)`** marks the end, and combines with the next file.
3. **An unknown value prints `?`** — _"If any item is unknown (for example, the file size if the
   file does not exist), it is replaced with `?`"_.

## 7. Focus and selection

### 7.1 The four attributes, with their defaults

`menu_attributes(3x)` fixes all four and leaves no room for interpretation:

| Role                             | Attribute   | Default       |
| -------------------------------- | ----------- | ------------- |
| selected item                    | `menu_fore` | `A_REVERSE`   |
| selectable, unselected item      | `menu_back` | `A_NORMAL`    |
| **non-selectable** item          | `menu_grey` | `A_UNDERLINE` |
| gap between name and description | `menu_pad`  | one space     |

**Reverse video, not bold.** Bold fails on many terminals and already means emphasis elsewhere in
the interface.

### 7.2 The mark: a glyph in a one-column gutter

ncurses puts a text prefix on the selected item, and the default is `"-"`, _"In order to make menu
selections visible on older terminals without highlighting or color capability"_.

fzf separates the two markers explicitly, and that is the cleanest statement of the convention:

| Option      | Default | Role                                   |
| ----------- | ------- | -------------------------------------- |
| `--pointer` | `▌`     | the item under the cursor              |
| `--marker`  | `┃`     | a **selected** item in multi-selection |
| `--gutter`  | `▌`     | the column                             |

```text
▌ Item 2          ← pointer
┃ Item 4          ← selected (multi-selection)
  Item 5
```

**A half-height glyph for the cursor, a full-height one for the selection, in the same column.**

### 7.3 Three levels, not one

What no individual control says and what nonetheless has to be decided:

1. **Region focus** — which panel is active. In MC, in vim splits and in tmux, **the border** says
   it: the active panel has the bright border and the other a dim one.
2. **Control focus** — inside a region, which widget has the focus.
3. **Selection** — inside the control, which item is chosen.

`bubbles` separates them in the cleanest form found: focus is **a border**, not a color.

```go
s.SelectedTitle = lipgloss.NewStyle().
    Border(lipgloss.NormalBorder(), false, false, false, true).   // left border only
    Padding(0, 0, 0, 1)                                            // and padding from 2 to 1,
                                                                   // so the text does not move
```

```text
  Normal item
│ Selected item
│ selected description
```

### 7.4 Disabled

ncurses' default for a non-selectable item is `A_UNDERLINE` — and **underline is the opposite of
what "enabled" means**. Any other pattern is better than that one.

What is well attested:

- **MC**: a disabled menu item is marked `[-]`.
- **dialog**: a disabled item is not drawn with a different glyph but with a different **attribute**
  — the hardware cursor is parked on the first character of the focused button's label.
- **Textual** (`_footer.py`): `.-disabled { text-style: dim; }`.

**The pattern:** disabled is marked with a **dimmed attribute**, not with a different glyph. A
different glyph is left for a different meaning.

### 7.5 The hardware cursor has the last word

- **`curs_set(0)`** in menus and lists. Emacs does it explicitly: `tty_hide_cursor(tty)` while the
  menu is open.
- **`curs_set(1)`** in a text field, parked at the cursor position. It is _"the only reliable focus
  indicator in a text terminal"_ — the position is the only focus signal that cannot be switched
  off.
- **The blink is not the application's.** Every library delegates to the emulator's cursor; none
  prescribes a frequency. See §13.

## 8. Common controls

### 8.1 Padding

The most citable rule in this document, from `ratatui-widgets/src/block/padding.rs`:

> _"Terminal cells are often taller than they are wide, so to make horizontal and vertical padding
> seem equal, doubling the horizontal padding is usually pretty good."_

`Padding::proportional(n)` is literally `left = right = 2n`, `top = bottom = n`:

```text
padding for a control:  2 columns at the sides, 1 above and below
```

Corroborated by dialog's constants: `MARGIN 1` (each box's frame), `INDENT 3` (one level of tree),
`ARROWS_COL 5` (left margin of the scroll arrows), and a checklist's description column starting at
`check_x + 4`.

### 8.2 Text field

- It is a **window of exactly the field's width**; the text scrolls inside. ncurses: _"The buffer
  contents always have the same length, and are padded with trailing spaces as needed"_.
- The cursor goes inside and is the only thing that shows the focus.
- **Placeholder = dimmed, not another tone.** fzf calls it `--ghost`, with a `ghost` style that
  applies `dim` by default; `bubbles` swaps it for a `Placeholder` style and **puts the cursor on
  the placeholder's first character**.
- **Mask = `*`, reveal = plain text, and the toggle is a first-class key.** Emacs: _"every character
  you type is displayed as an asterisk ('_') instead of its usual form"_, and _"TAB toggles the
  visibility of the password."_ dialog does the opposite by default (`--passwordbox` shows nothing)
  and `--insecure` is what adds the asterisks, explicitly _"friendlier but less secure"_.
- **Validation**: in ncurses it is a type constraint (`TYPE_INTEGER`, `TYPE_ALNUM`, `TYPE_ENUM`,
  `TYPE_IPV4`…) applied **on trying to leave the field**, not while typing. The focus does not move
  and the error code is `E_INVALID_FIELD`. **There is no red-border convention** (§13).

The editing keys MC applies to all of its input widgets, which are the canonical set:

| Key                                    | Action                                                                                      |
| -------------------------------------- | ------------------------------------------------------------------------------------------- |
| `C-a` / `Home` · `C-e` / `End`         | start / end of line                                                                         |
| `C-b` `C-f` / `←` `→`                  | move one character                                                                          |
| `M-b` `M-f`                            | move by word                                                                                |
| `C-h` / `Backspace` · `C-d` / `Delete` | delete before / at the cursor                                                               |
| `C-k` · `M-d` `M-DEL` · `C-w`          | kill to end · word forward · word backward                                                  |
| `C-y`                                  | paste the kill ring                                                                         |
| `M-p` `M-n`                            | previous / next in history                                                                  |
| `M-Tab`                                | complete; _"Press M-Tab again to get a listbox if there are multiple possible completions"_ |

### 8.3 Checkbox and radio

dialog's distinction is not folklore, it is in `checklist.c`:

```c
data->checkflag == FLAG_CHECK ? "[%c]" : "(%c)"
```

with the default alphabet `states = " *"`. That is:

```text
[X]  checked        (*)  selected
[ ]  unchecked      ( )  not selected
```

**Brackets for multi-selection, parentheses for radio.** Four columns.

Focus is an **axis separate** from state: dialog applies one attribute to the bracket glyph and a
different one to the label, so a checked but unfocused row looks different from a checked and
focused one. And exclusivity is enforced by the render rather than by input: _"If this is a
radiobutton list, ensure that no more than one item is selected initially"_.

**The key is the same in both: `SPACE`.** In dialog there is no rule of "the arrows change a radio's
value": the arrows move the _cursor_ and `SPACE` confirms. That is the correct model — in a vertical
group, moving the cursor already changes what is seen.

### 8.4 Lists

- **Single selection**: a gutter glyph plus a highlight over the whole row. ratatui exposes it as
  `highlight_style` + `highlight_symbol`, with `repeat_highlight_symbol` so the symbol repeats on
  long rows and the block stays rectangular.
- **Multi-selection**: a second, taller glyph (§7.2).
- **Type-ahead**: ncurses' `menu_driver` treats any printable character as another character of the
  _pattern buffer_ and jumps to the next matching item, returning `E_NO_MATCH` and erasing the
  character when there is none. It is free and it is what makes `fzf` implausibly fast to use.
- **Scrolling within the list, never paging the bar.** dialog: _"There are `menu-height` entries
  displayed in the menu at one time, but the menu will be scrolled if there are more"_.
- **Truncation with an explicit ellipsis.** fzf: `--ellipsis` default `··`; `bubbles` uses `…`; tig
  truncates and appends `…` or `~`.

### 8.5 Tree

dialog draws one `│` per level of depth, at a step of **3 columns**, with the label starting at
`item_x + 3 × depth`:

```text
[ ] src
│   [ ] core
│   │   [ ] main.rs
│   [ ] tests
[*] docs
```

**But dialog's tree is always expanded**: the depth comes per row from the input data
(`tag item status depth`), and there is no collapse glyph.

`rich` is the other dominant convention and uses **guides**, not only indentation: `TREE_GUIDES`
with `┬ ├ └ ─ │`, and an `expanded` boolean per node.

The collapsible-section marker, where one exists, is `[-]` / `[+]` — tig's,
`keymap->hidden ? '+' : '-'`.

Keys for MC's directory tree: `Enter` descend, `←` parent, `→` child, `↑` `↓` siblings, `F4` toggle
static/dynamic navigation, `C-s` search (which has to be activated first, because otherwise the
character is eaten by quick search), `Esc` exit.

### 8.6 Tabs

ratatui states it in `Tabs`' own doc comment:

> _"The divider defaults to `|`, and padding defaults to a singular space on each side."_ _"The
> default widget has: No tabs; No selected tab; The highlight style is set to reversed."_

```rust
const DEFAULT_HIGHLIGHT_STYLE: Style = Style::new().reversed();
```

```text
 Files │ Diagrams │  Settings
                        ↑ reversed
```

Reverse on the active tab, a vertical bar between tabs, one space of padding on each side. And MC's
menubar is the degenerate case of "one tab per top-level menu".

### 8.7 Progress and spinner

Three renderings of the bar, all agreeing on what matters: **the full glyph, a half-step glyph for
the edge, and the label drawn inverted over the bar so it reads.**

- **rich**: full `━`, right half-step `╸`, left half-step `╺`, the rest `━` in the background color.
- **ratatui**: `█` full, and on the edge cell an octant block (`▏▎▍▌▋▊▉█`) chosen by
  `get_unicode_block(frac)`. The label is drawn by **swapping foreground and background**, not by
  overprinting.
- **bubbles**: full `▌` with fg/bg blending, or `█` without, empty `░`. Default
  `PercentFormat: " %3.0f%%"`, width 40.

Indeterminate: rich calls it _"a pulsing animation"_ (`PULSE_SIZE = 20`); dialog does the opposite
and uses **numeric-only** widgets, no animation.

**The spinner.** The ten-frame braille set at 80 ms is the de facto default:

```text
⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏        10 frames, 80 ms  — rich "dots" and bubbles "MiniDot" (12 fps)
| / - \                 ~10 fps           — bubbles "Line", and rich "line"
```

And **every frame has to be the same width**, or the rest of the line trembles: rich's `clock`
frames carry a trailing space on purpose, and bubbles' `Dot` frames are `"⣾ "` with a space. That is
the kind of detail that separates an application that looks finished from one that does not.

### 8.8 Scrollbar

ratatui's symbol module documents the anatomy with ASCII art that is directly reusable:

```text
<--▮------->
^  ^   ^   ^
│  │   │   └ end
│  │   └──── track
│  └──────── thumb
└─────────── begin
```

Four independent slots: `track` `│`, `thumb` `█`, `begin` `↑`, `end` `↓`.

Two opposite decisions about when to show it, both well attested:

- **dialog: hidden by default.** `--scrollbar` is opt-in. The percentage, on the other hand, is
  drawn whenever either of the two arrows is needed.
- **fzf: visible by default, with `--no-scrollbar` to turn it off.** And it adds `--scroll-off=3`:
  the list does not let the last item reach the edge, which is the affordance that replaces the bar.

### 8.9 Combobox and dropdown

There is no combobox in any surveyed library. There are three patterns, and **none of them puts the
list over the field**:

1. **Dialog style**: a text field plus one or two lists **beside or above** it. dialog's `--fselect`
   is _"a text-entry window […] and above that two windows with directory names and filenames"_, and
   _"typing any printable characters switches focus to the text-entry window, entering that
   character as well as scrolling the directory and filename windows to the closest match"_.
2. **Minibuffer style**: the list is a **separate buffer** that appears on request (Emacs's
   `*Completions*`).
3. **Fuzzy finder style**: its own view, full screen or with `--height`.

In all three, one rule repeats: **typing in the field moves the highlight live, and the list
reselects the closest match rather than jumping to the first.**

For a non-editable select, the canonical terminal dropdown is ncurses' `TYPE_ENUM` with
`REQ_NEXT_CHOICE` / `REQ_PREV_CHOICE`: cycling forward and backward through the options, inside the
same field.

### 8.10 Buttons

Two namings with distinct histories, and it is worth knowing where each comes from:

| Form          | Where                                                                    | Focus glyph            |
| ------------- | ------------------------------------------------------------------------ | ---------------------- |
| `[ Label ]`   | MC, and `midnight-commander.org` writes it that way in the documentation | the brackets           |
| `<Label>`     | dialog, `buttons.c`: `waddstr(win, "<")` … `waddstr(win, ">")`           | the angles             |
| `[< Label >]` | MC, the default button                                                   | angles inside brackets |

All three agree on what matters: **the focused button is the only one with a mark**, `Tab` rotates,
`Enter` accepts, and the key is **shown inside the button** in a different attribute.

dialog's geometry, measured from the code: minimum inner width 6 (or 5), every button in a row
equalized to the longest, the gap computed as `unused / (count + 3)` plus `count*2` for the two
angle characters per button, and the hardware cursor parked on the first character of the focused
button's label.

And `--default-button` exists for a concrete reason: _"By preselecting a button, a script makes it
possible for the user to simply press Enter to proceed through a dialog with minimum interaction."_

### 8.11 The box

```text
╭─ Confirm ──────────────────────────────────────────╮
│                                                   │
│   Delete 3 figures?                                │
│                                                   │
│        <Help>  <OK>  <Cancel>                      │
╰───────────────────────[ F1 for help ]─────── 25% ──╯
```

From the bottom up, all verified in dialog's code:

- The top border carries the **title inside it**.
- The button row goes at `height - 2`, that is, the second from the bottom.
- The help line `[ text ]` goes **centred over the bottom border**.
- The scroll percentage goes **right-aligned over the bottom border**, with `(-)` and `(+)` to the
  left of the list.
- The shadow, if there is one, is a separate option (MC: an appearance option).

### 8.12 Hint, tooltip and footnote

There is no tooltip in the hover sense, because there is no hover. There are three distinct surfaces
with their own names:

- **Per-item hint, on the last line of the screen.** dialog's `--item-help`: _"adding a column which
  is displayed in the bottom line of the screen, for the currently selected item"_, redrawn on every
  cursor move.
- **Static footnote, centred inside the widget.** `--hline`: _"Display the given string centered at
  the bottom of the widget"_.
- **Toast.** Textual's `Toast`: _"short-lived notifications"_, with `border-left` by severity. tmux
  does the same in its bar, with `display-time` at 750 ms.

## 9. Glyphs, palette and degradation

### 9.1 What can be assumed

- **8 colors, degrading to 16; 256 only if `$TERM` says so, truecolor only if
  `$COLORTERM=truecolor`.**
- **Use the terminal's default color** wherever possible, so the same binary looks right on light
  and on dark.
- **`NO_COLOR`** is the de facto standard for turning all of it off.

### 9.2 Glyph sets

| Set            | Default                                       |
| -------------- | --------------------------------------------- |
| spinner        | `⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏` at 80 ms; ASCII `\| / - \`       |
| progress bar   | `━` full, `╸`/`╺` half-step, `░` or `─` empty |
| blocks         | `█▉▊▋▌▍▎▏` from full to an eighth             |
| scrollbar      | `│` track, `█` thumb, `↑` `↓` at the ends     |
| pointer/marker | `▌` cursor, `┃` selected                      |
| shadow         | `░` or ACS                                    |

Every one has an ASCII fallback, and every animated one needs **frames of constant width**.

### 9.3 What this repository already knows about this layer

[`glyph-sets.md`](glyph-sets.md) holds the two risks of this layer, and they are the same two:

- **The Windows Terminal font.** The catalogs use single-codepoint glyphs, but box-drawing, `░` and
  `▲` need a font that carries them. With Windows' legacy raster font, or a code page that is not
  UTF-8, it renders as garbage.
- **Cell width.** `monospace-core` assumes a glyph occupies one cell and computes no width. That
  holds for the palette above and stops holding for any double-width glyph.

What this survey adds: **the animated and block glyphs need a font that carries those ranges**, and
every tool surveyed has an ASCII fallback where this repository does not. That is a difference, not
a gap: the fallbacks above are what `mc`, `dialog` and `fzf` do, and this repository draws what the
terminal can show and does not detect what it cannot. The two lines above are what it is aware of,
and neither is answered by a fallback chain — `glyph-sets.md` §_ASCII_ holds the ASCII set as a
stroke a caller can ask for, and `model.md` §10 declined the per-stroke fallback chains that would
substitute it automatically.

## 10. The keyboard: the vocabulary

| Key              | Meaning                                               | Who fixed it            |
| ---------------- | ----------------------------------------------------- | ----------------------- |
| `↑ ↓ ← →`, `j k` | navigate                                              | everyone                |
| `Enter` / `RET`  | accept, descend                                       | everyone                |
| `Esc`            | **back, one level**                                   | the modern TUI standard |
| `?`              | help overlay                                          | Bubble Tea, Textual     |
| `q`              | quit                                                  | —                       |
| `Ctrl+C`         | always quit, and `SIGINT` has to be caught            | POSIX                   |
| `F1`–`F10`       | named actions                                         | MC                      |
| `Ctrl-` + letter | one-key commands                                      | MC, Emacs               |
| `Alt` + letter   | global command, or the alternative to the bare letter | MC, tview               |
| `M-Tab`          | complete                                              | MC                      |

And the mechanical part:

- The arrows, `Home`/`End`, `PageUp`/`Down` and the function keys arrive as **escape sequences**:
  only `terminfo` says which exist on this terminal.
- `ESC` is ambiguous (the start of a sequence), so `ESCDELAY` or `halfdelay` + `nodelay` is needed.
- `SIGWINCH` / `KEY_RESIZE`, and redraw everything.
- `Esc` + digit as a fallback for `F1`–`F10` — MC's convention, and Emacs's with `ESC ESC ESC`.
- The prefix as a modal layer (vi, tmux) is the best way to add commands without colliding with the
  editing keys.

## 11. Who fixed what

So that nobody cites the wrong source:

| Convention                 | Fixed by             | Concrete                                                            |
| -------------------------- | -------------------- | ------------------------------------------------------------------- |
| the drop-down menu         | MC + Emacs           | `F9`/`F10`, `←→` between menus, `↑↓` inside                         |
| the mnemonic               | MC                   | `&` in the data, the letter in another color, the bare letter fires |
| the box and the button     | dialog               | `<OK>`, `Tab` rotates, `Enter` accepts                              |
| the library's own menu     | ncurses              | `menu_fore = A_REVERSE`, mark `"-"`                                 |
| the key bar                | MC                   | `%2d` + label, minimum 7, falls off the right                       |
| the status line with flags | vim                  | `[+]` `[RO]`, `%=`, the group disappears                            |
| color per state            | tmux                 | one flag per character, reversed when there is an alert             |
| the one-line prompt        | `less`               | bare `:`, `(END)`, `%t` at the end                                  |
| the help overlay           | Bubble Tea / Textual | `?`                                                                 |
| the spinner                | rich / bubbles       | ten braille frames, four ASCII                                      |

## 12. What does not exist, or could not be verified

These things are **not** in any source, and should therefore not appear in code as though they were
convention:

1. **A "More" item in a toolbar.** It appears in no surveyed source. The closest is `less`'s
   `--More--` half-prompt, which is a prompt and not a toolbar.
2. **A tri-state / indeterminate checkbox.** dialog's `states` alphabet is extensible and the
   library mentions _"more than two states"_, but **no third glyph is specified**. `[-]` is invented
   unless someone invents it on purpose.
3. **A disabled-control rendering** in ncurses' `form`/`menu`, dialog, whiptail, rich, bubbles or
   ratatui. The closest is Textual's `Footer`'s `.-disabled { text-style: dim }`.
4. **The application drawing the cursor's blink.** Every library delegates to the emulator's cursor.
   None prescribes a frequency.
5. **A visual "validation failed" marker** on a text field. ncurses only refuses to move the focus
   (`E_INVALID_FIELD`). No red border and no warning glyph is specified.
6. **A "low level" subtree in `menu(3x)` via `*` in the format string.** `curs_vcur(3x)` could not
   be verified from any reachable source.
7. **Numeric shortcuts over a tab bar.** Confirmed in GUIs, not in terminals.
8. **`?` as "show shortcuts" inside an Emacs menu.** It does not exist: it is not in
   `tty-menu-navigation-map`, nor in `tmm.el`, nor in the manual. The documented form is that the
   shortcut is already shown beside the item.
9. **The `ncurses-ext` man pages `menu_bar.3x` and `popup_menu.3x` do not exist**: two source trees
   of ncurses have 144 pages each, 16 of them `menu*.3x`, and neither of those is one of them.

## 13. Sources

### Manual pages

- [`mc(1)`, Arch](https://man.archlinux.org/man/mc.1.en.txt) — the four-part layout, `F1`–`F10`, the
  `Esc`+digit fallback, quick search, the input line's keys, the tree's keys, the display formats.
- [mc(1), source.midnight-commander.org](https://source.midnight-commander.org/man/mc.html)
- [ncurses `menu(3x)`, Arch](https://man.archlinux.org/man/core/ncurses/menu.3x.en) and its siblings
  `menu_attributes`, `menu_driver`, `menu_format`, `menu_mark`, `menu_options`, `menu_pattern`,
  `menu_post`, `menu_spacing`, `menu_win`.
- [ncurses `form(3form)`, Debian](https://manpages.debian.org/trixie/ncurses-doc/form.3form.en.html),
  [`field_opts`](https://manpages.debian.org/trixie/ncurses-doc/field_opts.3form.en.html),
  [`field_validation`](https://manpages.debian.org/trixie/ncurses-doc/field_validation.3form.en.html),
  [`driver`](https://manpages.debian.org/trixie/ncurses-doc/driver.3form.en.html).
- [dialog(1), Debian](https://manpages.debian.org/trixie/dialog/dialog.1.en.html) and
  [dialog(3)](https://manpages.debian.org/trixie/libdialog-dev/dialog.3.en.html).
- [tig manual](https://jonas.github.io/tig/doc/manual.html) — screen regions, title line, status
  window, all default shortcuts.
- [Emacs §1 Organization of the Screen](https://www.gnu.org/software/emacs/manual/html_node/emacs/Screen.html),
  [§1.2 Echo Area](https://www.gnu.org/software/emacs/manual/html_node/emacs/Echo-Area.html),
  [§1.3 Mode Line](https://www.gnu.org/software/emacs/manual/html_node/emacs/Mode-Line.html),
  [§1.4 The Menu Bar](https://www.gnu.org/software/emacs/manual/html_node/emacs/Menu-Bar.html),
  [§10.4 Completion](https://www.gnu.org/software/emacs/manual/html_node/emacs/Completion.html),
  [§10.7 Passwords](https://www.gnu.org/software/emacs/manual/html_node/emacs/Passwords.html).
- [Elisp: Defining Menus](https://www.gnu.org/software/emacs/manual/html_node/elisp/Defining-Menus.html),
  [Simple Menu Items](https://www.gnu.org/software/emacs/manual/html_node/elisp/Simple-Menu-Items.html).
- [less(1), Debian](https://manpages.debian.org/trixie/less/less.1.en.html) — the three default
  prompts and the complete conditional syntax.
- [vim `options.txt`](https://raw.githubusercontent.com/vim/vim/master/runtime/doc/options.txt) —
  `'statusline'`, `'ruler'`, `'laststatus'`, `'cmdheight'`, `'shortmess'`, `'showmode'`; and
  [`windows.txt`](https://raw.githubusercontent.com/vim/vim/master/runtime/doc/windows.txt) §
  `*status-line*`.
- [fzf(1), Arch](https://man.archlinux.org/man/fzf.1.en) — `--pointer`, `--marker`, `--gutter`,
  `--scrollbar`, `--ellipsis`, `--ghost`, `--scroll-off`, `--info`.

### Source code, which is where the exact glyphs are

- Midnight Commander:
  [`lib/widget/menu.c`](https://raw.githubusercontent.com/MidnightCommander/mc/master/lib/widget/menu.c),
  [`lib/widget/buttonbar.c`](https://raw.githubusercontent.com/MidnightCommander/mc/master/lib/widget/buttonbar.c),
  [`lib/widget/widget-common.c`](https://raw.githubusercontent.com/MidnightCommander/mc/master/lib/widget/widget-common.c),
  [`src/filemanager/filemanager.c`](https://raw.githubusercontent.com/MidnightCommander/mc/master/src/filemanager/filemanager.c),
  [`src/filemanager/panel.c`](https://raw.githubusercontent.com/MidnightCommander/mc/master/src/filemanager/panel.c),
  [`src/keymap.c`](https://raw.githubusercontent.com/MidnightCommander/mc/master/src/keymap.c),
  [`misc/skins/default.ini`](https://raw.githubusercontent.com/MidnightCommander/mc/master/misc/skins/default.ini).
- Emacs: [`src/menu.c`](https://raw.githubusercontent.com/emacs-mirror/emacs/master/src/menu.c),
  [`src/xdisp.c`](https://raw.githubusercontent.com/emacs-mirror/emacs/master/src/xdisp.c),
  [`src/term.c`](https://raw.githubusercontent.com/emacs-mirror/emacs/master/src/term.c),
  [`lisp/menu-bar.el`](https://raw.githubusercontent.com/emacs-mirror/emacs/master/lisp/menu-bar.el).
- dialog 1.3: [`buttons.c`](https://sources.debian.org/data/main/d/dialog/1.3-20250116-1/buttons.c),
  [`checklist.c`](https://sources.debian.org/data/main/d/dialog/1.3-20250116-1/checklist.c),
  [`inputbox.c`](https://sources.debian.org/data/main/d/dialog/1.3-20250116-1/inputbox.c),
  [`treeview.c`](https://sources.debian.org/data/main/d/dialog/1.3-20250116-1/treeview.c),
  [`arrows.c`](https://sources.debian.org/data/main/d/dialog/1.3-20250116-1/arrows.c),
  [`dialog.h`](https://sources.debian.org/data/main/d/dialog/1.3-20250116-1/dialog.h).
- ratatui:
  [`gauge.rs`](https://raw.githubusercontent.com/ratatui/ratatui/main/ratatui-widgets/src/gauge.rs),
  [`list.rs`](https://raw.githubusercontent.com/ratatui/ratatui/main/ratatui-widgets/src/list.rs),
  [`tabs.rs`](https://raw.githubusercontent.com/ratatui/ratatui/main/ratatui-widgets/src/tabs.rs),
  [`borders.rs`](https://raw.githubusercontent.com/ratatui/ratatui/main/ratatui-widgets/src/borders.rs),
  [`block/padding.rs`](https://raw.githubusercontent.com/ratatui/ratatui/main/ratatui-widgets/src/block/padding.rs),
  [`symbols/scrollbar.rs`](https://raw.githubusercontent.com/ratatui/ratatui/main/ratatui-core/src/symbols/scrollbar.rs).
- rich: [`box.py`](https://raw.githubusercontent.com/Textualize/rich/master/rich/box.py),
  [`_spinners.py`](https://raw.githubusercontent.com/Textualize/rich/master/rich/_spinners.py),
  [`progress_bar.py`](https://raw.githubusercontent.com/Textualize/rich/master/rich/progress_bar.py).
- bubbles:
  [`list/defaultitem.go`](https://raw.githubusercontent.com/charmbracelet/bubbles/master/list/defaultitem.go),
  [`spinner/spinner.go`](https://raw.githubusercontent.com/charmbracelet/bubbles/master/spinner/spinner.go),
  [`progress/progress.go`](https://raw.githubusercontent.com/charmbracelet/bubbles/master/progress/progress.go),
  [`textinput/textinput.go`](https://raw.githubusercontent.com/charmbracelet/bubbles/master/textinput/textinput.go).
- tig: [`src/display.c`](https://raw.githubusercontent.com/jonas/tig/master/src/display.c),
  [`src/help.c`](https://raw.githubusercontent.com/jonas/tig/master/src/help.c),
  [`src/prompt.c`](https://raw.githubusercontent.com/jonas/tig/master/src/prompt.c).
- tmux: [`tmux.1`](https://raw.githubusercontent.com/tmux/tmux/master/tmux.1) and
  [`options-table.c`](https://raw.githubusercontent.com/tmux/tmux/master/options-table.c).
- ncurses: [`test/demo_menus.c`](https://github.com/grgbr/ncurses/blob/4115bfaa/test/demo_menus.c).
- ratatui and Textual: the widget registries,
  [`ratatui-widgets/src/lib.rs`](https://raw.githubusercontent.com/ratatui/ratatui/main/ratatui-widgets/src/lib.rs)
  and
  [`textual/widgets/__init__.py`](https://raw.githubusercontent.com/Textualize/textual/main/src/textual/widgets/__init__.py),
  plus
  [`_footer.py`](https://raw.githubusercontent.com/Textualize/textual/main/src/textual/widgets/_footer.py),
  [`_header.py`](https://raw.githubusercontent.com/Textualize/textual/main/src/textual/widgets/_header.py)
  and
  [`_toast.py`](https://raw.githubusercontent.com/Textualize/textual/main/src/textual/widgets/_toast.py).
- Turbo Vision, as the ancestor:
  [`list_components.rs`](https://github.com/aovestdipaperino/turbo-vision-4-rust/blob/main/examples/list_components.rs).

### A note on the manual-page sources

`invisible-island.net`, which is ncurses' canonical host, blocks by geolocation from the network the
research was done on. Those pages were read through **Arch Linux**, which serves the same ncurses
6.6 mdoc documents. The same criterion was applied to everything else: if a claim could not be
opened, it is not here.
