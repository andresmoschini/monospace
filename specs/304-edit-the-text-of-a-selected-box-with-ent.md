---
status: draft
date: 2026-11-05
---

# Edit the text of a selected box with `t`

## Why now

A box's `text: Option<Text>` is stored, drawn and reported, but no command sets it. The model has
had the text block since the initial set and the description format already reads and writes it;
what is missing is the way in. This is also the first modal that is neither the framework's message
box nor the file dialog, so it settles what a multi-line editor is made of and where its keys go
before the glyph palette needs the same answers. Issue
[#304](https://github.com/andresmoschini/monospace/issues/304) proposed `Enter` to open it, but
`Enter` is the accept key of every existing dialog, so this cut uses the free `t` and the `Ok`
button instead; D1 and D2 say why.

## Scope

### In

- `TEXT: [t] Edit` in the selection block, before `STROKE`, on a selected box only. `t` opens the
  editor; `Enter` stays unoffered and ignored on the diagram.
- A text editor overlay made of the framework's `TextArea` inside a `DialogFrame` with `Ok` and
  `Cancel`. It opens with all existing text selected; `Enter` inserts a line; `Tab` moves focus;
  `Ok` saves; `Cancel` and `Esc` cancel; the text area is fixed in size, soft-wraps, scrolls and
  answers the pointer.
- `Shape::with_text` on `monospace-diagram`, `Command::SetText` on `monospace-editing`, and the
  application's `Action::EditText` and `Action::SetText`.
- Saving normalizes CRLF/CR to `\n`, expands tabs to spaces and drops every other control character.
  A buffer with no visible character saves `None`; any other buffer is stored literally.
- `t` joins `ALL_POSSIBLE_CONTEXT_KEYS` in the guardian; `Enter` and `a` stay where they are.

### Out

- **`[a] Auto-fit`.** It needs a display-column measure and a model amendment; it follows in its own
  issue once text can exist.
- **Drawing wide glyphs in the box.** The editor may hold them and counts their columns; the diagram
  keeps the model's one glyph per cell. Also its own issue.
- **Where a modal's command row comes from.** The lower row stays the diagram's for every overlay
  today; the glyph palette will need the general mechanism, and this cut does not build it.
- **The OS clipboard.** The framework's local clipboard is what `Ctrl+X`, `Ctrl+C` and `Ctrl+V` use;
  no clipboard dependency is added.
- **`F2`, `Ctrl+Enter`, a dirty marker and a confirmation on cancel.** `Ok` is the one save path and
  `Esc` discards.
- **Text on lines or connectors, and wrapping in the drawing.** Only a box holds a text, and the
  diagram does not wrap it.

## The decision

**D1 — `t` opens the editor; `Enter` remains the accept key.**

**Answer:** the contextual row offers `TEXT: [t] Edit` on a selected box, and `t` opens the editor.
`Enter` stays in `NEVER_AN_ACTION` and is ignored on the diagram; its meaning in About, Open and
Save does not change. **Why not `Enter`** as the issue proposed: it is the one key every dialog here
already reads as "accept", and a diagram command that took it would make the same key mean two
things one layer apart. **Why not `e` or `F2`:** `t` is the mnemonic of the thing edited, it is
unclaimed in every table, and `F2` stays free for a later meaning. **Answered by** the maintainer,
in the conversation on #304.

**D2 — the editor is the framework's `TextArea` inside `DialogFrame`, and `Ok` saves.**

**Answer:** the overlay is a `DialogFrame` titled `Text of Box #id`, with a `TextArea` in its body
and `Ok` and `Cancel` in its button row. The text area starts focused and holds the keyboard; `Tab`
moves to `Ok`, then `Cancel`, then back; `Shift+Tab` reverses. `Enter` inserts a line in the text
area and presses a focused button; `Esc` and `Cancel` discard without confirmation. **Why not
`Enter` to save:** the text is multi-line, so the key that ends a line cannot also commit it. **Why
not `F2` or `Ctrl+Enter`:** `Ctrl+Enter` is indistinguishable from `Enter` on several terminals, and
the application already has a standard "take the focused button" mechanism that needs no second key.
**Why not a hand-written buffer:** `TextArea` already brings selection, word movement, local undo,
scrolling, the pointer and wide-glyph columns, all of which a minimal buffer would have to grow
into.

**D3 — while the editor is up, `Ctrl+C` is the editor's and not the forced exit.**

**Answer:** `callbacks::route` asks the active text editor before `shortcuts::handle_always`;
`Ctrl+C` copies the current selection to the application's local clipboard, and does nothing when
there is none. The forced exit is suspended while the editor is up: leaving means `Esc` or `Ok`, and
then `Ctrl+C`. **Why not keep the forced exit:** `TextArea` handles `Ctrl+C` as copy by default, and
a copy key that ends the process is a trap; the maintainer chose the editor's meaning. **Why not a
hybrid that exits when there is no selection:** it would make the same key mean two things according
to a selection the person cannot see, and it would be harder to explain and to test. The application
model's promise about `Ctrl+C` with an overlay up is amended for this overlay alone.

**D4 — the buffer opens with all its text selected, and empty means "no text".**

**Answer:** opening seeds the `TextArea` with the current lines joined by `\n` and selects all of
them; a box with no text opens empty with the cursor in the one empty line. Saving normalizes the
buffer: CRLF and a lone CR become `\n`, each tab becomes eight spaces, and every other character
`char::is_control` answers is dropped. If what remains is only whitespace the command saves `None`;
otherwise `Text::new` is given the literal buffer and stores it, surrounding spaces included. **Why
all selected:** the maintainer chose replace-first, which makes replacing a label one keystroke.
**Why not reject invalid text:** the core stays strict, but a paste must not trap a person in a
modal with no way to succeed; the boundary normalizes and the model keeps refusing. **Why not trim
what is stored:** lines are literal in the text block, and spaces a person typed are part of the
drawing even when invisible. **Why not `Some(Text::new(""))`:** an invisible text is not a thing a
person can see or take back, and `None` is the honest spelling of "no text".

**D5 — the status bar stays exactly as it was while the editor is up.**

**Answer:** the frame keeps painting both rows from `DiagramState`, exactly as it does behind About,
Open and Save; opening or closing the editor changes the region and nothing else, and a test
compares the status area before and after. **Why not rewrite the lower row with the editor's keys:**
the maintainer chose consistency with the overlays already there, and the general question of where
a modal's command row comes from is deferred to the glyph palette. **Why not hide or dim the bar:**
it would diverge from the other overlays and change the frame for one modal. **The price, stated
rather than hidden:** the row still offers `TEXT: [t] Edit`, `MOVE` and the rest while the overlay
owns the keyboard, so those keys are dead — the same dead offer About, Open and Save already show,
and it is not the Representation Rule's
([#288](https://github.com/andresmoschini/monospace/issues/288)) to settle here.

**D6 — the modal is fixed in size, centred and clamped, and the text area soft-wraps.**

**Answer:** the dialog is at most 56 columns by 14 rows, centred in the region and clamped to it the
way `open` and `save` clamp theirs; `TextArea` keeps its default soft wrap, scrolls with the cursor,
answers the pointer, and has no scrollbars of its own. **Why not grow with the text:** the
maintainer chose a fixed size; it is simpler, the modal does not jump while typing, and the text
area already scrolls. **Why not the whole region:** it would cover the diagram and depart from the
application's dialog shape. **Why soft wrap:** it is the framework's default and the simplest thing
that works; the drawing still does not wrap, and the difference is written where the editor is
described.

**D7 — Auto-fit and wide-glyph drawing are out of this cut; wide glyphs are accepted and
documented.**

**Answer:** the command row does not offer `[a] Auto-fit`, and nothing resizes a box;
`Text::width()` keeps counting glyphs. The editor accepts a wide cluster, shows it in two columns
and moves the cursor by columns; the box still draws it in one cell, as `docs/model.md` says. **Why
not ship Auto-fit:** it needs a display-column measure that does not exist, and measuring columns
amends the model's "one glyph is one cell"; that is a change of its own. **Why not refuse wide
glyphs in the editor:** the description format already accepts them, and refusing them only at the
keyboard would make one way into the document stricter than another. **Why not fix the drawing
here:** it reaches `TextShape`, the core's cell model and the glyph tables, which is more than this
cut can prove.

## Model slice

- `docs/application-model.md`
  - _§2 The frame_: the overlay layer holds the text editor beside the message, open and save
    dialogs; it is drawn last of all and sets the screen cursor when the text area holds the focus.
  - _§3 The state_: the chrome gains the text editor's state — its text area, its frame and whether
    it is up — and opening it selects all the box's text.
  - _§4 The action model_: `EditText` opens the overlay and records no step; `SetText` carries the
    new text and performs one command; `Ctrl+C` is the text editor's while it is up, and the
    forced-exit promise is amended for that overlay.
  - _§7 Properties worth testing_: the new bullets below.
- `docs/editing-model.md` — _§2 The initial set_: `SetText { id, text }` joins the table, with
  `replace` and `Shape::with_text` as the change it becomes.
- `docs/diagram-model.md` — _§9 Changing a diagram_: `Shape::with_text` joins the three figure
  mutations; it changes only a box's `text`, and giving a box the text it already holds is no
  change.
- `docs/model.md` and `docs/tui-conventions.md` are **not** amended: the text block, its lines and
  its one-glyph-one-cell rule already exist, normalization happens at the editor's boundary, and the
  second is a survey.

## Public surface

```rust
// monospace-diagram
impl Shape {
    #[must_use]
    pub fn with_text(&self, text: Option<Text>) -> Self;
}

// monospace-editing
pub enum Command {
    SetText { id: ShapeId, text: Option<Text> },
}
```

In `crates/monospace`, all crate-private: `Action::EditText`, `Action::SetText(Option<Text>)`,
`text::TextEditorState` with `open`, `handle` and `draw`, `DiagramState::set_text`, and the `TEXT`
block of `context_table`.

## Behavior

1. A selected box offers `TEXT: [t] Edit` before `STROKE`; a selected line or connector does not.
2. `t` on a selected box opens the editor. `t` on a line, a connector, nothing selected, or a stroke
   sub-mode is ignored and changes nothing.
3. `Enter` on a selected box is still ignored; the guardian keeps it in `NEVER_AN_ACTION`.
4. The editor is titled `Text of Box #id`, has `Ok` and `Cancel`, and opens with the text area
   focused and all existing text selected; a box with no text opens empty.
5. The editor is at most 56×14, centred in the region and clamped to it. The status area is
   byte-identical to the frame before the editor opened.
6. Typing replaces the selection. `Enter` inserts a line; `Backspace`, `Delete`, the arrows, `Home`,
   `End`, `PageUp`, `PageDown`, `Shift` with an arrow, and the framework's text-editing keys behave
   as the `TextArea` does. `Tab` moves focus and never inserts a tab.
7. `Ok` saves; `Enter` on a focused `Ok` saves; `Cancel` cancels; `Enter` on a focused `Cancel`
   cancels; a click on either does the same.
8. `Esc` cancels from any focus, discards the buffer, records no step, leaves the diagram and the
   history untouched, keeps the box selected, and closes the editor.
9. On save, CRLF and a lone CR become `\n`, each tab becomes eight spaces, and every other control
   character is dropped.
10. If the normalized buffer has no character other than whitespace, the command saves `None`;
    otherwise the buffer is stored literally, surrounding spaces included.
11. Saving changed text records exactly one step; saving the text the box already holds records
    none; saving `None` over `None` records none. Saving closes the editor, leaves the box selected,
    and `Ctrl+Z` gives the change back as one step while `Ctrl+Shift+Z` redoes it.
12. `t` typed inside the editor inserts `t`; the overlay consumes it, and it neither reopens the
    editor nor reaches the region.
13. While the editor is up, `Ctrl+C` copies the selection to the local clipboard and does not end
    the application; with no selection it does nothing. `Ctrl+Q`, `Ctrl+S`, `Ctrl+O` and `Ctrl+N` do
    nothing. `Ctrl+X`, `Ctrl+V`, `Ctrl+A`, `Ctrl+Z` and `Ctrl+Shift+Z` are the text area's, and its
    undo walks the text rather than the diagram.
14. The screen cursor is shown while the text area holds the focus and is absent while a button
    does.
15. A click inside the editor moves the caret or the selection; a click outside it reaches nothing.
    The wheel scrolls the text.
16. A wide cluster is accepted, edited in its two columns and drawn by the box in one cell.
17. The command row does not offer `[a] Auto-fit`, and no box is resized by this cut.

## Examples

None of it has been observed: it is what the acceptance list checks, not the record of a run.

- A box with no text, `t`, type `hi`, `Tab`, `Enter`: the box's `text` becomes `Some("hi")` and the
  session records one step.
- A box with text `one\ntwo`, `t` (all selected), type `x`, `Tab`, `Enter`: the text becomes
  `Some("x")` and the session records one step.
- A box with text `one`, `t`, `Esc`: the text is still `Some("one")` and the history is where it
  was.
- A box with text `one`, `t`, `Tab`, `Enter`: nothing changed, no step is recorded, the editor
  closes.
- A box with text `one`, `t`, `Ctrl+A`, `Delete`, `Tab`, `Enter`: the text becomes `None` and the
  session records one step.
- Pasting `a\r\n\tb` into the editor and saving stores `Some("a\n        b")`.
- A box with only spaces or only line breaks, saved, stores `None`.
- Typing `漢` stores `Some("漢")`; the editor shows it in two columns and the box in one.

## What proves it

- `monospace-diagram`: `with_text_changes_only_a_box`,
  `with_text_on_a_line_or_connector_changes_nothing`, `setting_the_same_text_is_no_change`.
- `monospace-editing`: `a_set_text_command_changes_only_a_box`,
  `setting_the_same_text_records_no_step`, `clearing_a_text_is_one_step`.
- `monospace`: `t_opens_the_text_editor_on_a_box`, `t_is_ignored_off_a_box`,
  `enter_remains_ignored_on_a_box`, `the_status_bar_is_unchanged_while_editing`,
  `the_editor_opens_with_all_the_text_selected`, `ok_saves_and_escape_discards`,
  `saving_the_same_text_records_no_step`, `saving_normalizes_control_characters`,
  `saving_whitespace_only_stores_none`, `ctrl_c_copies_while_the_text_editor_is_up`,
  `the_editor_consumes_every_key`, and the snapshots of a box with the editor over it and of the
  same box after it closes.
- The guardian test walks `ALL_POSSIBLE_CONTEXT_KEYS` and `NEVER_AN_ACTION` and holds `t` offered on
  a box only and `Enter` ignored everywhere in the region.

## Open questions

- **The forced exit's reach.** `Ctrl+C` is suspended for this overlay alone; whether the other
  overlays should follow is not decided here.
