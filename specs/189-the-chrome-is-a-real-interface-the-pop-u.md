---
status: draft
date: 2026-10-25
---

# The chrome is a real interface: the pop-up, the focus and the states of a control

## Why now

**The code and rule 4 disagree, and this slice makes the code hold the rule.** The bar's focused
label is drawn with the theme's `focus` background; rule 4 says a control the keyboard is on is
drawn in reverse video, and forbids it on the region's frame. `Style::FOCUS` is a filled background,
not the modifier the rule names.

**What this unlocks.** #179 is paused on its D1 — which control holds the keyboard — and #176 wants
the arrows in the region. The bar answers `Left` and `Right` even with no drop-down open, so a bare
arrow never reaches the region; this rule is what lets both be built. The About dialog is the first
pop-up and the first control that is not a menu entry: it is where the overlay rule and the states
become code. #179's D1 can be answered from here.

## Scope

### In

- The diagram region holds the keyboard at startup. The bar and the region are the two leaves, and a
  left press moves the keyboard to the rectangle it lands in, border included; the press selects a
  shape only inside the interior (D1).
- A focused control is drawn in reverse video over the theme's colors, not the theme's `focus`
  background, and a pressed control that is not focused is reversed too; the region has no cue (D2,
  D3).
- `Alt` with a label's navigation character (`Alt+F`, `Alt+E`, `Alt+H`) focuses the bar and opens
  that drop-down in one press, by translating the event into what the menu already does when it
  holds the keyboard (D4).
- `Help > About`, `Action::About`, and the dialog's state in `AppState` (D5).
- The overlay rule: while the dialog is up it takes every key and every control but the shortcuts of
  `Reach::Always`. `Ctrl+C` ends the application, `Ctrl+Q` and a click on the bar behind it do
  nothing, and `Esc` and `Ok` are the way out (D6).
- The drop-down is not an overlay, so the unclaimed keys still reach the session behind it (#178,
  #181). The theme is the dark BASE16 palette, read once (D7).

### Out

- **The Quit and Delete confirmations.** `docs/application-model.md` §4 assigns them to this issue;
  this change ships About only and amends the model so the removal stays unconfirmed. The change
  that adds one uses `DialogFrame`.
- Moving the selected shape — #179. Scrolling — #176. The selection's inversion — #177, not a focus
  cue here. Greying an entry — #231. The status text — #226. The shortcut table — #180. A hover
  state (rule 2); a light palette or the terminal's own colors (D7); a dialog that asks two answers
  or has a text field; `Tab`/`Shift+Tab` between the bar and the region (the spike).

## The decision

**D1 — the keyboard is in one of two places, and the diagram region is where it starts.**

**Answer:** the diagram region holds the keyboard at startup, and a left press moves it to the
rectangle it lands in — the bar, or the region's outer rectangle, border included; the press selects
a shape only inside the interior. The bar and the region are the two leaves; the status line is not
one. **Why not** the bar at startup: the diagram is what a person came to work on, and a bar that
starts focused promises a menu before a drawing. **Why not** a selection-derived rule: it answers
"which place does this key go to?" with a fact about the drawing, so tabbing to the bar would be
undone by the selection still there. **Answered by** the maintainer, in #189.

**D2 — the region has no focus cue of its own.**

**Answer:** the region is the body: it is drawn the same whether or not it holds the keyboard, and
its frame is never reversed. The controls that have a cue show it, so it says where the keyboard is
without the region claiming it. **Why not** a distinct border: the 2026-10-11 amendment withdrew the
border cue and spent reverse video on controls; a border is a hundred and fifty cells saying what
seven say, and the coarser one flickers. **Answered by** the maintainer, in #189.

**D3 — the keyboard's control is reversed, and a pressed control that is not focused is too.**

**Answer:** the theme gives each control its colors, and the application adds `Modifier::REVERSED`
to the control the keyboard is on — the menu's label, the drop-down's entry, the dialog's `Ok`; the
region is D2's exception. A pressed `Ok` that is not focused is reversed too, so the two cues never
compete. **The selection's inversion is #177's**, not a focus cue, and this slice does not change
it. **Why not** the theme's `focus` style: it is a background color, rule 4 names reverse video, and
the dialog's `Ok` would then say "focused" differently from the menu. **Answered by** rule 4,
amended on 2026-10-11; #177 owns the selection.

**D4 — the bar is reachable with `Alt` and a label's navigation character.**

**Answer:** `Alt` with a label's own navigation character (`Alt+F`, `Alt+E`, `Alt+H`) focuses the
bar and opens that drop-down in one press, from either leaf. It is not a second menu: the handler
sets the bar's focus and hands the same event to the menu, whose navigation character already opens
the label's drop-down when it holds the keyboard. The character comes from the labels, so the menu
stays the only table of names. **Why not** `F9` or a bare `Alt`: the framework has neither, and a
terminal reports a bare `Alt` as `Esc` if at all, which would close a drop-down. **Why not** `Tab`
alone: a menu is not a form, and the menu widget has no key of its own that asks for the bar.
**Answered by** the maintainer, in #189.

**D5 — the About dialog is the framework's message dialog, under `Help`.**

**Answer:** `Help > About`, no shortcut; the framework's `MsgDialog` with the theme's `MSG_DIALOG`
style, the title `About monospace`, the body `monospace <version>` from `status::NAME` and
`status::VERSION`, and the framework's `Ok`. **Why not** `DialogFrame`: it is for a question with
two answers, and About has one; `MsgDialog` is the framework's one-button dialog, and the spike's
About already works in it. **Answered by** the maintainer, in #189.

**D6 — the dialog is the overlay, the drop-down is not, and `Ctrl+C` wins over both.**

**Answer:** while the About dialog is up it takes every key and every control except the shortcuts
whose reach is `Reach::Always`. `Esc` and `Ok` are the way out, a click on the bar behind it reaches
nothing, and `Ctrl+Q` is unclaimed so it does nothing. `Ctrl+C` is `Reach::Always` and is asked
before the dialog, so it ends the application with the dialog up. The drop-down is not an overlay:
the unclaimed keys still reach the session behind it. **Why not** letting the dialog swallow
`Ctrl+C`: the forced exit would then depend on the state, which is what its reach exists to prevent.
**Why not** treating every pop-up as an overlay: it would reverse #178's and #181's behavior behind
an open menu, and rule 14's overlay is what holds the keyboard, which the drop-down never does.
**Answered by** rule 14's amendment, D3 of #178, and `Reach::Always`.

**D7 — the palette stays the dark BASE16 theme, read once.**

**Answer:** `Config` keeps `create_dark(BASE16)`; this slice adds no palette and no reason to change
the theme. **Why not** a light palette now: the framework's light theme is the dark one over a light
palette, so "looks right on a light terminal" is a palette to choose rather than a style set that
exists, and nothing here can tell which terminal is under it. **Answered by** D5 of #174, which
fixes the palette as the configuration's choice.

## Model slice

- `docs/application-model.md` §2: the draw order gains the dialog last of all, after the drop-down;
  a closed dialog leaves the region byte-identical.
- §3: the focus is one of the chrome's own, beside the selected label and the open drop-down, and
  not the selection; the region keeps its outer rectangle beside its interior.
- §4: `About` changes the chrome's own and no session. The Delete paragraph's "the dialog that asks
  is #189's" is amended: this slice ships About only, and the removal stays unconfirmed. The
  `Ctrl+C` paragraph names the dialog: the shortcut that reaches everything is answered before it.
- §5: `rat-widget`'s `MsgDialog` is the pop-up; the widgets' keys stay theirs; `Alt` reaches the
  menu's navigation character (D4).
- §6: the focus leaves "Deliberately unresolved"; the light palette and the confirmations stay.
- `docs/tui-conventions.md` §7.3: the survey puts the region's focus on the border; this one does
  not. The survey is not amended; the divergence is recorded here.

## Public surface

No crate gains or loses a public item, the binary's arguments do not change, and
`crates/monospace/Cargo.toml` gains no dependency.

- `Action` gains `About`, with no fields, so it stays `Copy`, `PartialEq` and `Eq`. `AppState` gains
  `about: MsgDialogState`; `AppState::new` starts it closed and the region focused.
- `AppState` gains `focus: Focus`, a two-variant value (`Bar`, `Region`) that starts `Region`; the
  menu is handed whether the bar holds it. `rat-focus` is not added: the slice has no `Tab`, the
  dialog owns its own focus, and the manifest's dependencies do not change. `DiagramState` keeps the
  region's outer rectangle beside `interior`.
- `menu::MENU` gains `("_Help", &["_About"])`, `HELP` and `ABOUT`, and `meaning` maps
  `(HELP, ABOUT)` to `Action::About`. `menu::handle` takes whether the bar holds the keyboard and
  translates `Alt` plus a navigation character into the focus and open the menu already answers.
- A new `about` module: `open(&mut MsgDialogState)`,
  `draw(Rect, &mut Buffer, &mut MsgDialogState, &SalsaTheme)` and
  `handle(&Event, &mut MsgDialogState) -> Outcome`; `frame::draw` draws it last, and `draw` reverses
  the button's `focus` and `armed` and clears `hover`.
- `callbacks::route`: `handle_always` first, before the dialog; if the dialog is open and did not
  answer, only the dialog is asked. Otherwise a left press first gives the keyboard to the leaf it
  lands in — the bar, its pop-up included, or the region — and a press outside both leaves it where
  it was. Then the menu, the region and `handle_unclaimed` follow. `callbacks::apply` answers
  `About` by opening the dialog and returning `Changed`, the session untouched.

## Behavior

1. Two focus leaves, the bar and the region; the status line is not one, and a click there moves
   nothing. The region holds the keyboard at startup.
2. A left press inside the region's rectangle moves the keyboard there, border included; inside the
   interior it also selects; outside both it changes neither. A press on the border focuses and
   selects no shape.
3. A press on the bar moves the keyboard there; the label under it opens its drop-down. A left press
   in the region while a drop-down is open moves the keyboard there, closes the drop-down on the
   same press, and selects inside the interior; the drop-down is not an overlay, so the press is not
   swallowed.
4. The control the keyboard is on is drawn in reverse video over the theme's colors; the theme's
   `focus` background is not used. The region looks the same with and without the keyboard, and its
   frame is never reversed. A pressed control that is not focused is reversed too. A control under
   the pointer is not a state, and the selection's inversion is #177's.
5. With the region holding the keyboard, the arrows open no drop-down and move no label; they are
   unclaimed until #179 binds them. With the bar holding it, the menu's keys hold: the navigation
   character opens a drop-down, the arrows move it, `Enter` activates, `Esc` closes.
6. `Alt` plus a label's navigation character focuses the bar and opens that drop-down in one press,
   from either leaf. A combination that names no label changes nothing and moves no keyboard; a bare
   `Alt` is not a key.
7. The drop-down closes when the bar loses the keyboard, and the region is byte-identical after. It
   is not an overlay: with it open, `Delete`, `Ctrl+Z`, `Ctrl+Shift+Z` and `Ctrl+Q` still reach the
   application, and the menu stays open.
8. `Help` is the third label and `About` its only entry, with no shortcut drawn. Taking it mints
   `Action::About`, which opens the dialog, changes no session and records no step.
9. The dialog is the framework's `MsgDialog`, with the theme's `MSG_DIALOG` style, drawn last over
   the region. Its title is `About monospace`, its body `monospace <version>`, its only button `Ok`.
10. While the dialog is open it takes every key and control except the shortcuts of `Reach::Always`:
    `Ctrl+C` still ends the application. `Esc` and `Ok` close it, and the closing event reaches
    nothing else. `Ctrl+Q` and a click on the bar behind it do nothing.
11. When the dialog opens the message holds the focus; `Tab` moves to `Ok`, `Shift+Tab` back, and
    the focused `Ok` is reversed. Closing leaves the keyboard where it was and the region
    byte-identical.
12. The palette is the dark BASE16 theme, read once at startup; this slice adds no light palette and
    reads no terminal color.

## Examples

Hypothetical: none of it has been observed. The diagram is one light 4×3 box at `(2, 1)`, `WIDTH` 60
and `HEIGHT` 20, so the interior starts at `(1, 2)`.

| Start          | Event          | Keyboard after | Action          | Screen                                    |
| -------------- | -------------- | -------------- | --------------- | ----------------------------------------- |
| startup        | none           | region         | none            | no label reversed                         |
| startup        | `Alt+F`        | bar            | `ScreenChanged` | `File` drop-down open                     |
| region         | `Right`        | region         | none            | no drop-down, no label moves              |
| bar            | `Right`        | bar            | `ScreenChanged` | `Edit` selected, its drop-down open       |
| bar            | press `(3, 3)` | region         | `ScreenChanged` | box selected, no label reversed           |
| bar, Edit open | press `(3, 3)` | region         | `ScreenChanged` | drop-down closed, box selected            |
| bar, Edit open | `Delete`       | bar            | `Delete`        | box gone behind the menu, menu still open |
| bar            | `Help > About` | bar            | `About`         | dialog open, no session change            |
| dialog         | `Ctrl+C`       | —              | `ForceQuit`     | the application ends                      |
| dialog         | `Ctrl+Q`       | bar            | none            | dialog still open                         |
| dialog         | `Tab`          | bar            | none            | `Ok` reversed                             |
| dialog         | `Esc` or `Ok`  | bar            | `ScreenChanged` | dialog gone, region byte-identical        |

The table carries the decisions: `Alt+F` is D4's one press; `Delete` is D6's drop-down, which stays
open; the `Ctrl+C` row is the forced exit reaching through the overlay.

## What proves it

The tests do not exist yet; the names bind the building stage, and `already` marks one that exists.

| Rule | Test                                                                                                                                                                                                                 |
| ---- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1    | `the_diagram_region_holds_the_keyboard_at_startup`, `a_click_on_the_status_bar_moves_nothing`                                                                                                                        |
| 2    | `a_press_inside_the_region_moves_the_keyboard_and_selects`, `a_press_on_the_border_moves_the_keyboard_and_selects_nothing`                                                                                           |
| 3    | `a_press_on_a_label_moves_the_keyboard_and_opens_its_drop_down`, `a_press_in_the_region_closes_the_drop_down_and_selects`                                                                                            |
| 4    | `a_control_the_keyboard_is_on_is_drawn_in_reverse_video`, `a_pressed_ok_that_is_not_focused_is_reversed`, `the_region_is_drawn_the_same_with_and_without_the_keyboard`, `a_control_under_the_pointer_is_not_a_state` |
| 5    | `the_arrows_are_not_the_bars_while_the_region_holds_the_keyboard`; `the_navigation_character_opens_the_drop_down_and_escape_closes_it` — already                                                                     |
| 6    | `alt_and_a_labels_character_opens_that_labels_drop_down`, `alt_with_a_character_that_names_no_label_changes_nothing`                                                                                                 |
| 7    | `a_drop_down_leaves_the_region_byte_identical_when_it_closes`; `the_delete_key_is_answered_with_the_drop_down_open`, `the_undo_and_redo_keys_are_answered_with_the_drop_down_open` — both already                    |
| 8    | `the_help_menu_holds_about_and_draws_no_shortcut`, `taking_the_about_entry_mints_the_about_action`, `the_about_action_changes_no_session`                                                                            |
| 9    | `the_about_dialog_carries_the_name_and_the_version`, `the_dialog_is_drawn_last_of_all`                                                                                                                               |
| 10   | `escape_closes_the_about_dialog_and_the_event_reaches_nothing_else`, `control_q_does_nothing_while_the_dialog_is_up`, `control_c_ends_the_application_whatever_the_state_is` — already                               |
| 11   | `tab_moves_from_the_message_to_the_ok_and_the_ok_is_reversed`, `the_region_is_byte_identical_after_the_dialog_closes`                                                                                                |
| 12   | `the_palette_is_the_dark_base16_theme` — a frame compared to the theme's own                                                                                                                                         |

Every bar snapshot moves: the labels gain `Help` and File and Edit are untouched; the focused
label's style moves because D3 makes it reverse video, and the region's border does not because D2
leaves it alone. `the_bar_and_the_status_start_inert` still holds: nothing is selected until the
first frame.

## Open questions

**The light palette.** D7 keeps the dark BASE16 theme, and the framework's light theme is the dark
one over a light palette; the change that adds a configuration chooses one or asks.

**The confirmation dialogs.** This slice ships About only, so `Delete` stays unconfirmed and `Quit`
does not ask; the change that adds either uses `DialogFrame` and decides whether the way out asks.
Joining the shortcut table and the menu is #180's.
