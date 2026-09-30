<!-- The feature branch is named after the issue title, truncated by `cargo xtask spec`. That
     truncation landed inside a word, so the spell checker is told to expect it rather than left to
     guess: cspell:ignore carri -->

# Feature Specification: A description names its shapes, and carries the ordinal the next one takes

**Feature Branch**: `148-a-description-names-its-shapes-and-carri-deciding` | **Issue**:
[#148](https://github.com/andresmoschini/monospace/issues/148) | **Created**: 2026-09-30 |
**Status**: Draft

**Input**: Issue #148 — a description names its shapes by the place they are written, so
`into_diagram` issues `#1`, `#2`, … in array order and a reference written `"#5"` means the fifth
entry. This slice lets the file carry each shape's identity and the ordinal the next shape takes, so
a reference names the shape the file names and adding to a diagram read from a file continues the
file's numbering instead of restarting it. It answers §11's _Can a caller choose an identity?_ — the
trigger that section names, which 083 fired and declined. Part of
[#62](https://github.com/andresmoschini/monospace/issues/62), on top of
[#83](https://github.com/andresmoschini/monospace/issues/83).

## What this slice implements

Three sections of [`docs/diagram-model.md`](../../docs/diagram-model.md), **the first two amended**
— unlike 083's slice, which needed no amendment, because §3 already says a caller may write an
identity down and stops one sentence short of letting the diagram issue the one the caller wrote.

- [§3 _Identity_](../../docs/diagram-model.md#3-identity) — amended twice. A caller may choose the
  identity a shape is added under; and "unique within that diagram" becomes a sentence about the
  identities the diagram issues rather than a promise it keeps on a caller's behalf, because a
  caller-supplied identity is not checked (B3.1). Left: editing one after the fact, which §3's own
  second paragraph keeps out.
- [§9 _Changing a diagram_](../../docs/diagram-model.md#9-changing-a-diagram) — the `add` row says a
  shape is added and "gives it an identity"; this slice says which one, and that it is never one the
  diagram has handed out before.
- [§11 _Open questions_](../../docs/diagram-model.md#11-open-questions) — _Can a caller choose an
  identity?_ comes out, and its surviving half — editing one — is rewritten in place rather than
  left to name a trigger that has already fired.

One thing this adds that the model does not describe, because it is not the model's: the description
format grows the field carrying the identity, and every file in the repository grows it with it —
`assets/demo.json`'s 26 entries and the 63 shapes in the 24 markers that carry a real description.
That format belongs to the application
([ADR-0035](../../docs/decisions/0035-keep-the-cli-demo-format-out-of-the-model.md)), so this is a
consumer of the model and a change to the format. Letting a caller choose an identity is observable
from outside `monospace-diagram`, so the §3 amendment and an ADR in
[`docs/decisions/`](../../docs/decisions/README.md) are part of this slice and not a later one.
082's D3 declined a public way to add a shape under a chosen identity for being early; this is that
slice.

## Clarifications

### Session 2026-09-30

- Q: [P1] Is the field every entry has to carry, or does an entry without one go on being numbered
  where it sits? → A: **Required** — an entry without one is refused and named like any other
  missing required field. It costs 89 lines in ten tracked files and no picture moves; the
  alternative was refused because two ways of naming one shape is what 083's `at` tag exists to
  remove.
- Q: [P2] Two entries carrying the same identity — refused, or accepted? → A: **Accepted** — both
  are held and the first is what every change and every reference finds. Refusing it was preferred
  only if `serde` could refuse it in one line, and it cannot: measured on this crate's own
  `serde_json`, two entries carrying one identity read without an error, exactly as one key written
  twice inside one object does. Nobody hand-edits these files and an editor is coming. §3's "unique
  within that diagram" is amended rather than enforced — see **Edge cases**.
- Q: [P3] May an identity be free text, or is it an ordinal and nothing else? → A: **Free text, and
  every file in this repository keeps writing `#1`, `#2`, …** §11's trigger is "the first slice
  where a caller has a _name worth keeping_", and leaving the shipped files ordinal-spelled keeps
  the demonstrable change the one 083 made. Re-spelling them with names is a later commit on the
  same format, not a change to it.

## Behavior

### B1 — A reference names the shape the file names, not the place it is written

1. **Given** two descriptions differing only in the order of two box entries — the same two boxes,
   the same connector, and the reference naming the second entry — **When** each is read, **Then**
   they draw different pictures, because the second entry is a different box in each:

   ```text
   ┌──┐    ┌──┐
   │  │    │  │
   └──┘    └──┘

          ┌─┐▼
          ▲ └┘
   ```

   ```text
   ┌──┐    ┌──┐
   │  │    │  │
   └──┘    └──┘

     ▼ ┌──┐
     └─┘  ▲
   ```

   Measured, not generated — `cargo run -p monospace-cli` on both files, neither of which writes an
   identity. This is the behavior the slice replaces, and the pair is the whole of it: two entries
   swapped, one arrow moved.

2. **Given** those two files, each naming its boxes rather than pointing at their places, **When**
   each is read, **Then** both draw the first of the two pictures above, byte for byte. Writing the
   entries in the other order changes nothing the file did not say to change.
3. **Given** a shape added under an identity its caller chose, **When** it is asked for, replaced,
   removed, moved forward or moved backward by that identity, **Then** it is the shape those changes
   name, and by any other identity it is not a shape that diagram holds.

### B2 — The ordinal the next shape takes

1. **Given** a diagram whose shapes carry the identities `#1`, `#2` and `#3`, **When** a shape is
   added to it, **Then** the identity handed back is not one of those three, and adding again hands
   back a fourth, different one. Which number it is and where that number comes from is **the
   sheet's**; what is required here is only that the ordinal survives being read.
2. **Given** an identity a shape held and no longer holds, **When** a shape is added, **Then** the
   identity taken is not that one. This is the rule 081 pinned and this slice does not change it.
3. **Given** the shipped description, **When** the demonstration names `#1`, `#3` and `#10` by hand,
   **Then** they name the same three figures they name today.

### B3 — A file that does not agree with itself

1. **Given** a description in which two entries carry the same identity, **When** it is read,
   **Then** both shapes are held, the first of them is the one every change naming that identity
   acts on and the one every reference to it resolves through, and nothing errors, reports or
   panics. The second is reachable by no identity at all until the first is removed.
2. **Given** a description holding an entry that carries no identity, **When** it is read, **Then**
   the file is refused and the identity is named in the message, exactly as `canvas`, `shapes`,
   `leaving`, `terminal` and `at`'s `kind` are. No shape is added and nothing is drawn.
3. **Given** a description whose reference names an identity no entry carries, **When** it is read,
   **Then** the figure holding it is absent from the picture, every other shape is drawn exactly as
   it would have been, and the run succeeds. That is
   [ADR-0041](../../docs/decisions/0041-resolve-a-position-through-a-reference.md) arriving on a
   name rather than on a number, and it costs nothing extra.

### B4 — The demonstration keeps its five pictures

1. **Given** the shipped demonstration, **When** the application is run with no arguments, **Then**
   it prints the same five captioned pictures it prints today, character for character, and the file
   now carries an identity for each of its twenty-six entries.
2. **Given** a path, **When** the application is given one, **Then** it prints one picture and
   nothing else, exactly as today, which is what `cargo xtask render` embeds.

## Edge cases

- An identity carried by one entry and reached by a reference in a connector written **above** it in
  the array: it resolves. A file may name a shape it has not written yet, and the hole only appears
  if the name is never written at all.
- Two entries carrying one identity: the shape nobody can reach is not an error and not a report, it
  is a shape in the diagram. This is the cost B3.1 accepts on purpose, and it is the reason §3's
  "unique within that diagram" becomes a sentence about the identities the diagram issues rather
  than a promise the diagram keeps on a caller's behalf.
- An identity no entry carries and no reference names: nothing at all. It is a string in a file, not
  a slot the diagram keeps.
- Two identities differing only in case, or by a trailing space: two identities. Nothing in the
  format folds them together and nothing normalizes what a file wrote.
- A file naming identities with gaps — `#1`, `#7`, `#9` — and nothing filling them: the gaps are
  simply unused, and the ordinal the next shape takes is what B2 requires rather than the count of
  what was read.
- An identity written on a `box` and a reference naming it from a `connector` whose own identity the
  file also writes: the two never meet except through the name, and a `connector` still answers no
  anchor point of its own, so a chain of references stays one link long.

## What this slice does not decide

- **Whether the ordinal is a field of its own in the file or is read off the identities the file
  carries.** Issue #148 defers it to this stage, and B2 states only the observable rule — the
  ordinal survives, and it is never one the diagram has handed out. How it is kept is the sheet's,
  with the alternative and its cost.
- **Editing an identity after the fact.** §3's open question is half this and half that, and only
  this half fires. Renaming a shape is not something a file needs and a caller does not yet ask for.
  Settled by: the first consumer that edits rather than writes.
- **Asking a diagram what identities it holds.** B1.3 is observable through the changes that name an
  identity and through drawing, which is where every identity rule since 080 has been observable
  from. A listing is public surface added for a consumer that does not exist. Settled by: the first
  consumer that needs to read rather than name.
- **Reporting an identity that resolved to nothing.** ADR-0041 accepts the silent hole and
  [#88](https://github.com/andresmoschini/monospace/issues/88) is where it is written down; a name
  in a file is a second way to walk into the same hole and not a second kind of it.
- **Writing a description.** The format is read and never written — nothing in the repository
  derives `Serialize` — so the ordinal a file carries is one its author keeps by hand. B3.1 is the
  one disagreement this slice deliberately does not catch, and B3.2 is the one it does, which is the
  asymmetry P1's answer creates: a missing identity is refused, a repeated one is not. Settled by:
  the first consumer that has to save a diagram rather than build one, which is §11's last open
  question.

## Testing expectations

- **Contract** — a chosen identity is found by that identity and by nothing else: `get`, `remove`,
  `replace`, `forward` and `backward` each name the shape it was given, and an identity the file
  holds for a different shape changes nothing.
- **Contract** — a reference follows the name: two files differing only in the order of two
  non-overlapping entries draw the same buffer when the reference names the box, and B1.1's pair
  draws what it draws today when the reference names the place. Both directions, because a slice
  that made the name win in one and not the other would pass a single test.
- **Contract** — the ordinal, asked rather than drawn: after a diagram holds identities the caller
  chose, an addition hands back one no shape holds, a second addition hands back a different one,
  and an identity freed by a `remove` is not handed out again.
- **Contract** — a repeated identity, both ways: two entries carrying one are both held, and `get`
  returns the first — the cost B3.1 accepts, pinned so that a later slice which decides to report it
  has to say so rather than discover it.
- **Contract** — a missing identity is refused by name, with nothing added and nothing drawn, and
  free text reads: a file whose entries are named `right`, `left` and `arrow` draws the same picture
  as the ordinal-named file beside it.
- **Contract** — the demonstration: the five pictures are the ones it prints today, byte for byte,
  with an identity on each of the file's twenty-six entries, and a path prints one picture. The test
  pins the pictures and not the captions' wording.
- **Characterization** — none. Every rule this slice adds is a rule of the model stated in one
  sentence, which is what a contract test is for.

## Success criteria

- **SC-001**: Two files holding the same figures in a different order draw the same picture when a
  reference names a shape and a different one when it names a place, so writing a name into a file
  makes a later edit to that file's order stop being able to move an arrow by accident.
- **SC-002**: A shape is reachable by an identity the file wrote down, through every change the
  diagram offers, and by no other identity.
- **SC-003**: An identity added to a diagram read from a file is never one that file already used or
  that the diagram has handed out before.
- **SC-004**: A reference to an identity no entry carries still draws nothing, leaves every other
  shape untouched, and fails no run — and so does a shape held under an identity a second entry also
  carries, which is simply a shape nobody can name.
- **SC-005**: A bare run of the application prints the same five pictures it printed before, so the
  evidence for this slice is in the file and the pictures not moving is what proves it.
- **SC-006**: Every description in the repository names each of its shapes, and every picture drawn
  from one of them is byte for byte the picture it drew before this slice — the 63 shapes in the
  tracked markers and the 26 entries of `assets/demo.json` are all named, and 23 of the 24 markers
  have a gate step re-drawing to check it.
- **SC-007**: `cargo xtask check` is green, including `monospace-diagram` compiling for
  `wasm32-unknown-unknown`.
