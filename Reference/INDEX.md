# Reference Index

Every page/spec here was fetched from chessprogramming.org (CPW) or a primary
paper/spec it points to, and is the **only** permitted source of algorithmic
ideas for Zeus. No other codebase, and no unsourced "common knowledge," may
be used to justify an implementation choice — see `/CITATIONS.md` at the repo
root for the feature -> reference mapping, and `/ROADMAP.md` for what each
future stage will need to fetch before it is implemented.

| File | Source | Retrieved |
|---|---|---|
| `search/Negamax.md` | https://www.chessprogramming.org/Negamax | 2026-10-23 |
| `search/Alpha-Beta.md` | https://www.chessprogramming.org/Alpha-Beta | 2026-10-23 |
| `evaluation/Simplified_Evaluation_Function.md` | https://www.chessprogramming.org/Simplified_Evaluation_Function | 2026-10-23 |
| `evaluation/Point_Value.md` | https://www.chessprogramming.org/Point_Value | 2026-10-23 |
| `testing/Match_Statistics.md` | https://www.chessprogramming.org/Match_Statistics | 2026-10-23 |
| `protocols/UCI_Protocol.md` | https://www.chessprogramming.org/UCI (pointer) -> official spec, April 2006, Huber/Meyer-Kahlen | 2026-10-23 |

## Adding a new reference

1. Fetch the CPW page (or the primary paper it cites) and write a faithful
   markdown note under the appropriate subfolder, with a `source:` /
   `retrieved:` header — pseudocode blocks are kept verbatim, prose is
   extracted faithfully but need not be a byte-for-byte copy.
2. Add a row to the table above.
3. Add/extend the corresponding entry in `/CITATIONS.md` once code lands that
   uses it.
