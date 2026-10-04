# Glyph sets

A glyph set maps a combination of four arms to the character that draws it. The four key columns are
the top, right, bottom and left sides of a cell; each holds the name of the stroke that runs to that
side, or nothing when no stroke does. The fifth column is the character.

These are the sets the project starts from. Light is carried as data by `monospace-core`, and ASCII,
Double, Heavy, Light Round and the four mixing sets below by `monospace-glyph-sets`. Users will be
able to add their own, and a set may hold any combination it has a character for.

The five single-stroke sets are complete. Each holds 15 rows, which is every combination of four
sides except the empty one. That completeness is what the degradation rule in [`model.md`](model.md)
leans on: it degrades a cell to its base stroke, and needs the answer to be there.

The mixing sets cannot be complete, because they hold what Unicode provides. Light with heavy
happens to cover all 50 mixed combinations; light with double covers 18 of them, and the other 32
degrade.

`light-round` is identical to `light` except for its four corners, and its two mixing sets are
copies of `light`'s with the name changed. Keeping them as data rather than deriving them is a cost,
and a general one: a stroke that is a variant of another pays for the whole table, so four differing
corners cost fifteen rows of their own and sixty-eight copied from `light`. A declared fallback to
`light` would have made it four rows and inherited every mixed combination `light` has, and it was
declined together with the per-stroke fallback chains of [`model.md`](model.md) §10.

## ASCII

| Top   | Right | Bottom | Left  | Character |
| ----- | ----- | ------ | ----- | --------- |
|       |       | ascii  |       | \|        |
|       |       | ascii  | ascii | +         |
|       | ascii | ascii  | ascii | +         |
| ascii | ascii | ascii  | ascii | +         |
| ascii |       | ascii  | ascii | +         |
|       | ascii | ascii  |       | +         |
| ascii | ascii | ascii  |       | +         |
| ascii |       | ascii  |       | \|        |
|       |       |        | ascii | -         |
|       | ascii |        | ascii | -         |
| ascii | ascii |        | ascii | +         |
| ascii |       |        | ascii | +         |
|       | ascii |        |       | -         |
| ascii | ascii |        |       | +         |
| ascii |       |        |       | \|        |

## Light

| Top   | Right | Bottom | Left  | Character |
| ----- | ----- | ------ | ----- | --------- |
|       |       | light  |       | │         |
|       |       | light  | light | ┐         |
|       | light | light  | light | ┬         |
| light | light | light  | light | ┼         |
| light |       | light  | light | ┤         |
|       | light | light  |       | ┌         |
| light | light | light  |       | ├         |
| light |       | light  |       | │         |
|       |       |        | light | ─         |
|       | light |        | light | ─         |
| light | light |        | light | ┴         |
| light |       |        | light | ┘         |
|       | light |        |       | ─         |
| light | light |        |       | └         |
| light |       |        |       | │         |

## Double

| Top    | Right  | Bottom | Left   | Character |
| ------ | ------ | ------ | ------ | --------- |
|        |        | double |        | ║         |
|        |        | double | double | ╗         |
|        | double | double | double | ╦         |
| double | double | double | double | ╬         |
| double |        | double | double | ╣         |
|        | double | double |        | ╔         |
| double | double | double |        | ╠         |
| double |        | double |        | ║         |
|        |        |        | double | ═         |
|        | double |        | double | ═         |
| double | double |        | double | ╩         |
| double |        |        | double | ╝         |
|        | double |        |        | ═         |
| double | double |        |        | ╚         |
| double |        |        |        | ║         |

## Heavy

| Top   | Right | Bottom | Left  | Character |
| ----- | ----- | ------ | ----- | --------- |
|       |       | heavy  |       | ┃         |
|       |       | heavy  | heavy | ┓         |
|       | heavy | heavy  | heavy | ┳         |
| heavy | heavy | heavy  | heavy | ╋         |
| heavy |       | heavy  | heavy | ┫         |
|       | heavy | heavy  |       | ┏         |
| heavy | heavy | heavy  |       | ┣         |
| heavy |       | heavy  |       | ┃         |
|       |       |        | heavy | ━         |
|       | heavy |        | heavy | ━         |
| heavy | heavy |        | heavy | ┻         |
| heavy |       |        | heavy | ┛         |
|       | heavy |        |       | ━         |
| heavy | heavy |        |       | ┗         |
| heavy |       |        |       | ┃         |

## Light Round

| Top         | Right       | Bottom      | Left        | Character |
| ----------- | ----------- | ----------- | ----------- | --------- |
|             |             | light-round |             | │         |
|             |             | light-round | light-round | ╮         |
|             | light-round | light-round | light-round | ┬         |
| light-round | light-round | light-round | light-round | ┼         |
| light-round |             | light-round | light-round | ┤         |
|             | light-round | light-round |             | ╭         |
| light-round | light-round | light-round |             | ├         |
| light-round |             | light-round |             | │         |
|             |             |             | light-round | ─         |
|             | light-round |             | light-round | ─         |
| light-round | light-round |             | light-round | ┴         |
| light-round |             |             | light-round | ╯         |
|             | light-round |             |             | ─         |
| light-round | light-round |             |             | ╰         |
| light-round |             |             |             | │         |

## Mixing Light and Double

| Top    | Right  | Bottom | Left   | Character |
| ------ | ------ | ------ | ------ | --------- |
|        |        | double | light  | ╖         |
|        | light  | double | light  | ╥         |
| double | light  | double | light  | ╫         |
| double |        | double | light  | ╢         |
|        | light  | double |        | ╓         |
| double | light  | double |        | ╟         |
|        |        | light  | double | ╕         |
|        | double | light  | double | ╤         |
| light  | double | light  | double | ╪         |
| light  |        | light  | double | ╡         |
|        | double | light  |        | ╒         |
| light  | double | light  |        | ╞         |
| light  | double |        | double | ╧         |
| light  |        |        | double | ╛         |
| double | light  |        | light  | ╨         |
| double |        |        | light  | ╜         |
| double | light  |        |        | ╙         |
| light  | double |        |        | ╘         |

## Mixing Light and Heavy

| Top   | Right | Bottom | Left  | Character |
| ----- | ----- | ------ | ----- | --------- |
|       |       | heavy  | light | ┒         |
|       | light | heavy  |       | ┎         |
| light |       | heavy  |       | ╽         |
|       | light |        | heavy | ╾         |
| light |       |        | heavy | ┙         |
| light | heavy |        |       | ┕         |
|       |       | light  | heavy | ┑         |
|       | heavy | light  |       | ┍         |
| heavy |       | light  |       | ╿         |
|       | heavy |        | light | ╼         |
| heavy |       |        | light | ┚         |
| heavy | light |        |       | ┖         |
|       | light | heavy  | heavy | ┱         |
| light |       | heavy  | heavy | ┪         |
|       | heavy | heavy  | light | ┲         |
| heavy |       | heavy  | light | ┨         |
|       | light | heavy  | light | ┰         |
| light |       | heavy  | light | ┧         |
| heavy | light | heavy  |       | ┠         |
| light | light | heavy  |       | ┟         |
| light | heavy | heavy  |       | ┢         |
| light | heavy |        | heavy | ┷         |
| heavy | light |        | heavy | ┹         |
| light | light |        | heavy | ┵         |
|       | heavy | light  | heavy | ┯         |
| heavy |       | light  | heavy | ┩         |
|       | light | light  | heavy | ┭         |
| light |       | light  | heavy | ┥         |
| heavy | heavy | light  |       | ┡         |
| light | heavy | light  |       | ┝         |
|       | heavy | light  | light | ┮         |
| heavy |       | light  | light | ┦         |
| heavy | light | light  |       | ┞         |
| heavy | heavy |        | light | ┺         |
| light | heavy |        | light | ┶         |
| heavy | light |        | light | ┸         |
| light | heavy | heavy  | heavy | ╈         |
| heavy | light | heavy  | heavy | ╉         |
| light | light | heavy  | heavy | ╅         |
| heavy | heavy | heavy  | light | ╊         |
| light | heavy | heavy  | light | ╆         |
| heavy | light | heavy  | light | ╂         |
| light | light | heavy  | light | ╁         |
| heavy | heavy | light  | heavy | ╇         |
| light | heavy | light  | heavy | ┿         |
| heavy | light | light  | heavy | ╃         |
| light | light | light  | heavy | ┽         |
| heavy | heavy | light  | light | ╄         |
| light | heavy | light  | light | ┾         |
| heavy | light | light  | light | ╀         |

## Mixing Light Round and Double

The same combinations as _Mixing Light and Double_, with `light` replaced by `light-round`.

| Top         | Right       | Bottom      | Left        | Character |
| ----------- | ----------- | ----------- | ----------- | --------- |
|             |             | double      | light-round | ╖         |
|             | light-round | double      | light-round | ╥         |
| double      | light-round | double      | light-round | ╫         |
| double      |             | double      | light-round | ╢         |
|             | light-round | double      |             | ╓         |
| double      | light-round | double      |             | ╟         |
|             |             | light-round | double      | ╕         |
|             | double      | light-round | double      | ╤         |
| light-round | double      | light-round | double      | ╪         |
| light-round |             | light-round | double      | ╡         |
|             | double      | light-round |             | ╒         |
| light-round | double      | light-round |             | ╞         |
| light-round | double      |             | double      | ╧         |
| light-round |             |             | double      | ╛         |
| double      | light-round |             | light-round | ╨         |
| double      |             |             | light-round | ╜         |
| double      | light-round |             |             | ╙         |
| light-round | double      |             |             | ╘         |

## Mixing Light Round and Heavy

The same combinations as _Mixing Light and Heavy_, with `light` replaced by `light-round`.

| Top         | Right       | Bottom      | Left        | Character |
| ----------- | ----------- | ----------- | ----------- | --------- |
|             |             | heavy       | light-round | ┒         |
|             | light-round | heavy       |             | ┎         |
| light-round |             | heavy       |             | ╽         |
|             | light-round |             | heavy       | ╾         |
| light-round |             |             | heavy       | ┙         |
| light-round | heavy       |             |             | ┕         |
|             |             | light-round | heavy       | ┑         |
|             | heavy       | light-round |             | ┍         |
| heavy       |             | light-round |             | ╿         |
|             | heavy       |             | light-round | ╼         |
| heavy       |             |             | light-round | ┚         |
| heavy       | light-round |             |             | ┖         |
|             | light-round | heavy       | heavy       | ┱         |
| light-round |             | heavy       | heavy       | ┪         |
|             | heavy       | heavy       | light-round | ┲         |
| heavy       |             | heavy       | light-round | ┨         |
|             | light-round | heavy       | light-round | ┰         |
| light-round |             | heavy       | light-round | ┧         |
| heavy       | light-round | heavy       |             | ┠         |
| light-round | light-round | heavy       |             | ┟         |
| light-round | heavy       | heavy       |             | ┢         |
| light-round | heavy       |             | heavy       | ┷         |
| heavy       | light-round |             | heavy       | ┹         |
| light-round | light-round |             | heavy       | ┵         |
|             | heavy       | light-round | heavy       | ┯         |
| heavy       |             | light-round | heavy       | ┩         |
|             | light-round | light-round | heavy       | ┭         |
| light-round |             | light-round | heavy       | ┥         |
| heavy       | heavy       | light-round |             | ┡         |
| light-round | heavy       | light-round |             | ┝         |
|             | heavy       | light-round | light-round | ┮         |
| heavy       |             | light-round | light-round | ┦         |
| heavy       | light-round | light-round |             | ┞         |
| heavy       | heavy       |             | light-round | ┺         |
| light-round | heavy       |             | light-round | ┶         |
| heavy       | light-round |             | light-round | ┸         |
| light-round | heavy       | heavy       | heavy       | ╈         |
| heavy       | light-round | heavy       | heavy       | ╉         |
| light-round | light-round | heavy       | heavy       | ╅         |
| heavy       | heavy       | heavy       | light-round | ╊         |
| light-round | heavy       | heavy       | light-round | ╆         |
| heavy       | light-round | heavy       | light-round | ╂         |
| light-round | light-round | heavy       | light-round | ╁         |
| heavy       | heavy       | light-round | heavy       | ╇         |
| light-round | heavy       | light-round | heavy       | ┿         |
| heavy       | light-round | light-round | heavy       | ╃         |
| light-round | light-round | light-round | heavy       | ┽         |
| heavy       | heavy       | light-round | light-round | ╄         |
| light-round | heavy       | light-round | light-round | ┾         |
| heavy       | light-round | light-round | light-round | ╀         |
