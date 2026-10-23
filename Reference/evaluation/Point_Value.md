---
source: https://www.chessprogramming.org/Point_Value
retrieved: 2026-10-23
---

# Point Value

Historical piece point-value sets, pawn/knight/bishop/rook/queen (centipawn
equivalents):

| Source | Year | P / N / B / R / Q |
|---|---|---|
| H. S. M. Coxeter | 1940 | – / 300 / 350 / 550 / 1000 |
| Max Euwe & Hans Kramer | 1944 | 100 / 350 / 350 / 550 / 1000 |
| Claude Shannon | 1949 | 100 / 300 / 300 / 500 / 900 |
| Alan Turing | 1953 | 100 / 300 / 350 / 500 / 1000 |
| Mac Hack | 1967 | 100 / 325 / 350 / 500 / 975 |
| Chess 4.5 | 1977 | 100 / 325 / 350 / 500 / 900 |
| Tomasz Michniewski | 1995 | 100 / 320 / 330 / 500 / 900 |
| Hans Berliner | 1999 | 100 / 320 / 333 / 510 / 880 |
| Larry Kaufman | 1999 | 100 / 325 / 325 / 500 / 975 |
| Fruit | 2005 | 100 / 400 / 400 / 600 / 1200 |
| Larry Kaufman | 2012 | 100 / 350 / 350 / 525 / 1000 |

The common rule-of-thumb values `{1, 3, 3, 5, 9}` are attributed to Shannon
(1949).

## Zeus usage

Confirms the Michniewski 1995 values (100/320/330/500/900) used in
`Reference/evaluation/Simplified_Evaluation_Function.md` are a documented,
named historical set, not an invented number — this is the citation for the
exact piece-value constants in `zeus-engine::eval::PIECE_VALUE`.
