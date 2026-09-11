# Feature Specification: Fix shapes without filling closing their inner arms

**Feature Branch**: `049-bug-with-shapes-without-filling`

**Created**: 2026-09-11

**Status**: Draft

**Input**: User description: "Bug with shapes without filling. When the shapes does not have
filling, their inner arms should not be closed. For example a box without a fill whose border
overlaps another shape currently renders the overlap as if it were closed off (`├`, `┬`, ...)
instead of as a crossing (`┼`)."

## User Scenarios & Testing _(mandatory)_

### User Story 1 - An unfilled shape lets a crossing stroke show through (Priority: P1)

A person is composing a diagram out of more than one shape. One of the shapes has no fill. Another
shape's stroke happens to run into that first shape's interior — for instance, two boxes placed so
one box's border crosses the inside of the other. Today the unfilled shape's border still blocks the
second stroke, rendering the intersection as a closed corner or a T-junction. The person expects the
intersection to render as an open crossing instead, because there is nothing filling the first shape
to justify hiding what crosses into it.

**Why this priority**: This is the entire bug report. Nothing else in this spec has value without
it, and the example in the issue is exactly this scenario.

**Independent Test**: Render two overlapping boxes, neither filled, positioned so one box's border
enters the other's interior. Confirm the overlapping cells render as crossings rather than as closed
junctions.

**Acceptance Scenarios**:

1. **Given** a box drawn with no fill, **When** another shape's stroke crosses into that box's
   interior, **Then** the shared cell renders as an open crossing between the two strokes, not as a
   junction that treats the first box's interior side as closed.
2. **Given** the same layout with the first box drawn with a fill, **When** the diagram is rendered,
   **Then** the interior side stays closed exactly as it does today, so a crossing stroke is hidden
   by the fill rather than shown crossing over it.
3. **Given** the exact example from the bug report — an unfilled box whose bottom-right corner area
   overlaps a second unfilled box's top-left corner area — **When** the diagram is rendered,
   **Then** the output matches the corrected example in the report (crossing glyphs at both overlap
   points, in place of the closed-junction glyphs currently produced).

---

### Edge Cases

- A shape with no fill that no other shape ever touches renders exactly as it does today: this bug
  only shows up where strokes cross, so it must not change any output that has no overlap.
- Two unfilled shapes overlapping at more than one cell must resolve each overlapping cell to a
  crossing independently; fixing one occurrence must not miss another in the same render.
- A filled shape must keep hiding whatever crosses into it — this bug fix must not weaken that
  existing, intentional behavior.

## Requirements _(mandatory)_

### Functional Requirements

- **FR-001**: An unfilled shape MUST leave its interior side open to whatever stroke crosses into
  it, so the shared cell renders as a crossing rather than as a junction that treats the shape's
  interior as closed.
- **FR-002**: A filled shape MUST continue to close its interior side exactly as it does today, so a
  stroke crossing into it stays hidden behind the fill rather than shown crossing over it.
- **FR-003**: This fix MUST NOT change the rendered output of any diagram that has no overlapping
  shapes — only intersections that were rendered as closed junctions because of an unfilled shape's
  interior side are affected.

### Key Entities

- **Shape's fill**: whether a shape has a chosen glyph covering its interior, or none. Determines
  whether the shape's interior side should be closed (fill present) or open (no fill) to a crossing
  stroke.
- **Interior side**: the side of a shape's border that faces the shape's own inside, as opposed to
  the side that faces outward. This is the side the bug incorrectly closes regardless of fill.

## Success Criteria _(mandatory)_

### Measurable Outcomes

- **SC-001**: The example given in the bug report renders exactly as the report's corrected version,
  with no other change to that example's output.
- **SC-002**: Every existing diagram rendered by the project's demo and tests that involves no
  overlapping shapes produces byte-for-byte the same output as before this fix.
- **SC-003**: A diagram with an unfilled shape overlapped by another shape at more than one cell
  shows a crossing at every overlapping cell, not only the first one encountered.

## Assumptions

- "Shape" in scope for this fix is whatever shape kind already supports an optional fill in the
  current model (the box); the fix applies to that notion of fill wherever it exists, not to a
  redesign of which shapes may have one.
- The bug report's corrected example is taken as the authoritative expected output for that input;
  no alternative rendering was considered.
- No new user-facing input or option is introduced by this fix — it corrects existing rendering
  behavior for inputs that already produce a diagram today.
