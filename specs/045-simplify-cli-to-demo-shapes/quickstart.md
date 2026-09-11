# Quickstart: Simplify CLI to demo shapes

Validation scenarios for this feature. Run from the repository root.

## Prerequisites

```sh
cargo build -p monospace-cli
```

## 1. No arguments: the shipped demonstration (User Story 2)

```sh
cargo run -p monospace-cli
```

Expected: one diagram, no labels or headings (FR-021), showing all three shapes, a filled box
interior, overlap, and both stamp modes side by side on one canvas (FR-010, SC-004). Exits
successfully.

## 2. An explicit path renders the same file (User Story 2, scenario 3)

```sh
cargo run -p monospace-cli -- crates/monospace-cli/assets/demo.json
```

Expected: byte-identical output to scenario 1.

## 3. A hand-written file (User Story 1)

```sh
cat > /tmp/one-box.json <<'JSON'
{
  "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 } },
  "shapes": [
    { "kind": "box", "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
      "stroke": "light", "fill": "░", "mode": "above" }
  ]
}
JSON
cargo run -p monospace-cli -- /tmp/one-box.json
```

Expected:

```text
┌──┐
│░░│
└──┘
```

Run twice; the two outputs must be identical (FR-017, acceptance scenario 4).

## 4. A missing path (User Story 3, scenario 1)

```sh
cargo run -p monospace-cli -- /tmp/does-not-exist.json
```

Expected: nothing on stdout, a message naming `/tmp/does-not-exist.json` on stderr, failure exit
status.

## 5. Malformed JSON (User Story 3, scenario 2)

```sh
printf '{ "canvas": ' > /tmp/broken.json
cargo run -p monospace-cli -- /tmp/broken.json
```

Expected: nothing on stdout, a message locating the problem on stderr, failure exit status.

## 6. An unrecognized shape kind (User Story 3, scenario 3)

```sh
cat > /tmp/bad-kind.json <<'JSON'
{
  "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 } },
  "shapes": [ { "kind": "triangle", "mode": "above" } ]
}
JSON
cargo run -p monospace-cli -- /tmp/bad-kind.json
```

Expected: nothing on stdout, a message naming `triangle` on stderr, failure exit status.

## 7. More than one argument (Edge Cases)

```sh
cargo run -p monospace-cli -- one.json two.json
```

Expected: a usage message, failure exit status.

## Full check

```sh
cargo xtask check
```

Must stay green throughout implementation (constitution, principle III). See
[`contracts/description-format.md`](contracts/description-format.md) for the full file format and
[`data-model.md`](data-model.md) for the field-by-field mapping onto `monospace-core`.
