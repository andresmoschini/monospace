---
status: draft
date: 2026-10-25
---

# The chrome is a real interface: the pop-up, the focus and the states of a control

## Why now

**The code and rule 4 disagree, and this slice makes the code hold the rule.** The bar's focused
label is drawn with the theme's `focus` background; rule 4 says a control the keyboard is on is
drawn in reverse video, and the same rule forbids reverse video on the region's frame. The theme
does not settle it: `Style::FOCUS` is a filled background, so leaving the style alone makes the cue
a color rather than the modifier the rule names.

**What this unlocks.** #179 is paused on the focus model — its D1 is unanswered on its deciding
branch — and #176 wants the arrows in the region. The bar answers `Left` and `Right` even with no
drop-down open, so a bare arrow never reaches the region, and the two issues cannot both be built
without a rule saying which control holds the keyboard. This slice is that rule, and #179's D1 can
be answered from it. The About dialog is the first pop-up and the first control that is not a menu
entry: it is where the overlay rule and the states become code rather than prose.

## Scope

### In

- `rat-focus`'s explicit `Focus` over the bar and the region, with `Tab` and `Shift+Tab` and a left
  press; the bar holds the keyboard at startup (D1).
- The region holds the keyboard after a press inside its rectangle, border included; the click still
  selects only inside the interior, and the arrows stay unclaimed for #179.
- A focused control is drawn in reverse video over the theme's colors, not the theme's `focus`
  background, and a pressed control that is not focused is reversed too; the region has no cue and
  its frame is never reversed (D2, D3).
- `Alt` with a label's navigation character (`Alt+F`, `Alt+E`, `Alt+H`) focuses the bar and opens
  that drop-down in one press (D4).
- `Help > About`, `Action::About`, and the dialog's state in `AppState` (D5).
- The overlay rule: while the dialog is up it takes every key and every control; `Ctrl+Q`, `Ctrl+C`
  and a click on the bar behind it do nothing, and `Esc` and `Ok` are the way out (D6).
- The drop-down is not an overlay, so the unclaimed keys still reach the session behind it (#178,
  #181). The theme is the dark BASE16 palette, read once (D7).

### Out

- **The Quit and Delete confirmations.** `docs/application-model.md` §4 assigns them to this issue;
  this change ships About only and amends the model so the removal stays unconfirmed. The change
  that adds one uses `DialogFrame`.
- Moving the selected shape with the arrows — #179. Scrolling — #176. The selection highlight —
  #177. Greying an entry — #231. The status text — #226. The shortcut table — #180.
- A hover state (rule 2); a light palette or the terminal's own colors (D7); a dialog that asks a
  question with two answers; a dialog with a text field.

## The decision

**D1 — the keyboard is in one of two places, and `rat-focus` says which.**

**Answer:** an explicit `Focus` over the bar and the region, built from the state's flags on every
event; `Tab` and `Shift+Tab` move between them, a left press moves the keyboard to the rectangle it
lands in, and the bar holds it at startup. **Why not** the selection-derived rule of the #189
comment: it answers "which place does this key go to?" with a fact about the drawing, so tabbing to
the bar would be undone by the selection that is still there. **Why not** click-only: a keyboard
that cannot leave the bar cannot reach the diagram. **Answered by** the maintainer, in #189.

**D2 — the region has no focus cue of its own.**

**Answer:** the region is the body: it looks the same whether or not it holds the keyboard. The
control that has a cue shows it — the bar's label and entries, the dialog's `Ok` — and the bar is
plain when the keyboard is elsewhere. **Why not** a distinct border: the 2026-10-11 amendment
withdrew the border cue and spent reverse video on controls; a border is a hundred and fifty cells
saying what seven say, and the coarser one flickers. **Answered by** the maintainer, in #189.

**D3 — the keyboard's control is reversed, and a pressed control that is not focused is too.**

**Answer:** the theme gives each control its colors, and the application adds `Modifier::REVERSED`
to the control the keyboard is on — the menu's label, the drop-down's entry, the dialog's `Ok`; the
region is D2's exception. A pressed `Ok` that does not hold the keyboard is reversed as well, so the
two cues never compete. **Why not** the theme's `focus` style: it is a background color, rule 4
names reverse video, and the dialog's `Ok` would then say "focused" differently from the menu.
**Answered by** rule 4, amended on 2026-10-11.

**D4 — the bar is reachable with `Alt` and a label's character.**

**Answer:** `Alt` with `F`, `E` or `H` focuses the bar and opens that drop-down in one press, from
either area; the character is the label's own navigation character, read from the menu, so the menu
is still the only table of names. **Why not** `Tab` alone: a menu is not a form, and the menu widget
has no key of its own that asks for the bar. **Why not** a bare `Alt`: a terminal reports it as
`Esc` if at all, and `Esc` closes a drop-down. **Answered by** the maintainer, in #189.

**D5 — the About dialog is the framework's message dialog, under `Help`.**

**Answer:** `Help > About`, no shortcut; the framework's `MsgDialog` with the theme's `MSG_DIALOG`
style, the title `About monospace`, the body `monospace <version>` from `status::NAME` and
`status::VERSION`, and the framework's `Ok`; drawn last, over the region, which is byte-identical
after it closes. **Why not** `DialogFrame`: it is for a question with two answers, and About has
one; `MsgDialog` is the framework's one-button dialog, and the spike's About already works in it.
**Answered by** the maintainer, in #189.

**D6 — the dialog is the overlay, and the drop-down is not.**

**Answer:** while the About dialog is up it takes every key and every control: `Esc` and `Ok` are
the way out, `Ctrl+Q` and `Ctrl+C` do nothing, and a click on the bar behind it reaches nothing. The
drop-down is not an overlay: it belongs to the bar, and the bar holds the keyboard, so the unclaimed
keys still reach the session behind it. **Why not** treating every pop-up as an overlay: it would
reverse #178's and #181's agreed behavior behind an open menu, and rule 14's overlay is the thing
that holds the keyboard, which the drop-down never does. **Answered by** rule 14's amendment and D3
of #178, where the drop-down's reach was agreed.

**D7 — the palette stays the dark BASE16 theme, read once.**

**Answer:** `Config` keeps `create_dark(BASE16)` and this slice adds no palette. **Why not** a light
palette now: the framework's light theme is the dark one over a light palette, so "looks right on a
light terminal" is a palette to choose rather than a style set that exists, and nothing here can
tell which terminal is under it. **Answered by** D5 of #174, which fixes the palette as the
configuration's choice.

## Model slice

- `docs/application-model.md` §2: the draw order gains the dialog last of all, after the drop-down;
  a closed dialog leaves the region byte-identical.
- §3: the state gains the dialog's own state and the two focus flags among the chrome's own. The
  region's outer rectangle is kept beside its interior; the focus is not the selection.
- §4 and §7: `About` is an action that changes the chrome's own and no session. The Delete
  paragraph's "the dialog that asks is #189's" is amended: this slice ships About only, and the
  removal stays unconfirmed. The `Ctrl+C` paragraph and its property gain the overlay's exception:
  the dialog is asked before the shortcut table, so `Ctrl+C` does nothing while it is up.
- §5: `rat-focus` owns which leaf holds the keyboard; `rat-widget`'s `MsgDialog` is the pop-up; the
  widgets' keys stay theirs. The application answers `Alt` and the label's character, which the menu
  does not route.
- §6 and the paragraph above §1 that sends the controls, their states, the focus, the colors and the
  pop-up to this issue: the focus resolves to D1 and D2, the pop-up to D5 and D6, the states to D3,
  and the colors to D7. §8's light terminal and the confirmation stay open.
- `docs/tui-conventions.md` §7.3: the surveyed applications put the region's focus on the border;
  this one deliberately does not. The survey decides nothing and is not amended; the divergence is
  recorded here.

## Public surface

No crate gains or loses a public item, the binary's arguments do not change, and
`crates/monospace/Cargo.toml` gains no dependency: the focus types come through `rat_widget::focus`.

- `Action` gains `About`, with no fields, so it stays `Copy`, `PartialEq` and `Eq`. `AppState` gains
  `about: MsgDialogState`; `AppState::new` starts it closed and the bar focused.
- `DiagramState` gains a `FocusFlag`, the region's `Rect`, and an implementation of
  `rat_widget::focus::HasFocus`. `interior` stays what a click is answered from.
- `menu::MENU` gains `("_Help", &["_About"])`, `HELP` and `ABOUT`, and `meaning` maps
  `(HELP, ABOUT)` to `Action::About`. `menu::handle` takes whether the bar holds the keyboard, and
  `menu::widgets` replaces the theme's `MenuStyle.focus` and `popup_focus` with reverse video.
- A new `about` module: `open(&mut MsgDialogState)`,
  `draw(Rect, &mut Buffer, &mut MsgDialogState, &SalsaTheme)`, and
  `handle(&Event, &mut MsgDialogState) -> Outcome`. `frame::draw` draws it last; `draw` makes the
  button's `focus` and `armed` reverse video and clears `hover`.
- `callbacks::route` gains the dialog first while it is open, then `handle_always`, then the focus
  pass, then the `Alt` handling, then the menu with the bar's focus, then the region, then the
  unclaimed shortcuts. A focus change makes the frame owed when nothing else answers.
- `callbacks::apply` answers `Action::About` by opening the dialog and returning `Changed`; the
  session is untouched.

## Behavior

1. Two focus leaves, the bar and the region; the status line is not one, and clicking it moves
   nothing. The bar starts focused, and `Tab`/`Shift+Tab` move and wrap between the two.
2. A left press inside the region's rectangle moves the keyboard there, border included; a press
   inside the interior also selects; a press outside both rectangles changes neither.
3. A press on the bar moves the keyboard there; the label under it opens its drop-down.
4. The control the keyboard is on is drawn in reverse video over the theme's colors; the theme's
   `focus` background is not used. The region is drawn the same with and without the keyboard, its
   frame is never reversed, and a control under the pointer is not a state. A pressed control that
   is not focused is reversed too, so the two cues never compete.
5. With the region focused, `Left`, `Right`, `Up` and `Down` open no drop-down and move no label;
   they are unclaimed until #179 binds them. With the bar focused, the framework's keys hold: the
   navigation character opens a drop-down, the arrows move it, `Enter` activates, `Esc` closes.
6. `Alt` with a label's navigation character, from either area, focuses the bar and opens that
   label's drop-down in one press. `Alt` with a character that names no label changes nothing and
   does not move the keyboard; a bare `Alt` is not a key.
7. The drop-down closes when the bar loses the keyboard, and the region is byte-identical
   afterwards. It is not an overlay: with it open, `Delete`, `Ctrl+Z`, `Ctrl+Shift+Z` and `Ctrl+Q`
   still reach the application, and the menu stays open.
8. `Help` is the third label and `About` its only entry, with no shortcut drawn. Taking it mints
   `Action::About`, which opens the dialog, changes no session and records no step.
9. The dialog is the framework's `MsgDialog`, drawn with the theme's `MSG_DIALOG` style, last of
   all, over the region. Its title is `About monospace`, its body is `monospace <version>`, and its
   only button is `Ok`.
10. While the dialog is open it takes every key and every control. `Esc` and `Ok` close it, and the
    event that closes it reaches nothing else. `Ctrl+Q`, `Ctrl+C`, `F9` and a click on the bar
    behind it do nothing; `Ctrl+C` is the forced exit in every other state.
11. When the dialog opens the message holds its focus; `Tab` moves to `Ok`, `Shift+Tab` back, and
    the focused `Ok` is reversed. Closing leaves the keyboard where it was and the region
    byte-identical.
12. The palette is the dark BASE16 theme, read once at startup; this slice adds no light palette and
    reads no terminal color.

## Examples

Hypothetical: none of it has been observed. The diagram is one light 4×3 box at `(2, 1)`, `WIDTH` 60
and `HEIGHT` 20, so the interior starts at `(1, 2)`.

| Start          | Event                   | Keyboard after | Action          | Screen                                    |
| -------------- | ----------------------- | -------------- | --------------- | ----------------------------------------- |
| startup        | none                    | bar            | none            | `File` reversed, nothing else             |
| startup        | `Tab`                   | region         | `ScreenChanged` | no label reversed                         |
| region         | `Right`                 | region         | none            | no drop-down, no label moves              |
| bar            | `Right`                 | bar            | `ScreenChanged` | `Edit` selected, its drop-down open       |
| bar            | press `(3, 3)`          | region         | `ScreenChanged` | box selected, no label reversed           |
| region         | press on the status bar | region         | none            | nothing changes                           |
| region         | `Alt+E`                 | bar            | `ScreenChanged` | `Edit` drop-down open                     |
| bar, Edit open | `Delete`                | bar            | `Delete`        | box gone behind the menu, menu still open |
| bar            | `Help > About`          | bar            | `About`         | dialog open, no session change            |
| dialog         | `Ctrl+C`                | bar            | none            | dialog still open                         |
| dialog         | `Tab`                   | bar            | none            | `Ok` reversed                             |
| dialog         | `Esc` or `Ok`           | bar            | `ScreenChanged` | dialog gone, region byte-identical        |

The table carries the decisions: `Alt+E` is D4's one press; `Delete` is D6's drop-down, which stays
open; the `Ctrl+C` row is D6's other half, and the one test that changes meaning.

## What proves it

The tests do not exist yet; the names bind the building stage, and `already` marks one that exists.

| Rule | Test                                                                                                                                             |
| ---- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| 1    | `a_click_on_the_status_bar_moves_nothing`, `tab_and_shift_tab_move_between_two_areas`                                                            |
| 2    | `a_press_inside_the_region_moves_the_keyboard_and_selects`, `a_press_on_the_border_moves_the_keyboard_and_selects_nothing`                       |
| 3    | `a_press_on_a_label_moves_the_keyboard_and_opens_its_drop_down`                                                                                  |
| 4    | `a_control_the_keyboard_is_on_is_drawn_in_reverse_video`, `a_pressed_ok_that_is_not_focused_is_reversed`                                         |
| 4    | `the_region_is_drawn_the_same_with_and_without_the_keyboard`, `a_control_under_the_pointer_is_not_a_state`                                       |
| 5    | `the_arrows_are_not_the_bars_while_the_region_holds_the_keyboard`; `the_navigation_character_opens_the_drop_down_and_escape_closes_it` — already |
| 6    | `alt_and_a_labels_character_opens_that_labels_drop_down`, `alt_with_a_character_that_names_no_label_changes_nothing`                             |
| 7    | `a_drop_down_leaves_the_region_byte_identical_when_it_closes`                                                                                    |
| 7    | `the_delete_key_is_answered_with_the_drop_down_open`, `the_undo_and_redo_keys_are_answered_with_the_drop_down_open` — both already               |
| 8    | `the_help_menu_holds_about_and_draws_no_shortcut`, `taking_the_about_entry_mints_the_about_action`, `the_about_action_changes_no_session`        |
| 9    | `the_about_dialog_carries_the_name_and_the_version`, `the_dialog_is_drawn_last_of_all`                                                           |
| 10   | `escape_closes_the_about_dialog_and_the_event_reaches_nothing_else`, `control_q_and_control_c_do_nothing_while_the_dialog_is_up`                 |
| 10   | `control_c_ends_the_application_whatever_the_state_is_except_under_a_dialog` — changes                                                           |
| 11   | `tab_moves_from_the_message_to_the_ok_and_the_ok_is_reversed`, `the_region_is_byte_identical_after_the_dialog_closes`                            |
| 12   | `the_palette_is_the_dark_base16_theme` — a frame compared to the theme's own                                                                     |

Every bar snapshot moves: the labels gain `Help` and File and Edit are untouched. The focused
label's style moves because D3 makes it reverse video; the region's border does not, because D2
leaves it alone. `the_bar_and_the_status_start_inert` still holds: nothing is selected until the
first frame.

## Open questions

**The light palette.** D7 keeps the dark BASE16 palette, and the framework's light theme is the dark
one over a light palette; the change that adds a configuration chooses one or asks.

**The confirmation dialogs.** This slice ships About only, so `Delete` stays unconfirmed and `Quit`
does not ask; the change that adds either uses `DialogFrame` and decides whether the way out asks.

**The `Alt` letters and the shortcut table.** The letters are the menu's navigation characters, so
the menu is still the one table of names; joining the rest of the table and the menu is #180's.
