---
source: https://www.chessprogramming.org/Principal_Variation_Search
retrieved: 2026-11-02
---

# Principal Variation Search (PVS)

> based on null- or zero window searches of none PV-nodes, to prove a
> move is worse or not than an already safe score from the principal
> variation [...] finding a series of moves whose value is greater than
> alpha but lower than beta throughout the entire branch, the chances
> are that deviating from it will do us no good.

Search the first move (expected best, per move ordering) with the full
`[alpha, beta]` window to get its true value. Search every other move
with a cheap null (zero-width) window `[-alpha-1, -alpha]` that only
proves "worse than alpha" or "not worse" — if it comes back "not worse"
(`score > alpha`) *and* still inside the real window (`score < beta`),
that's surprising (move ordering predicted it would be worse) and a full
`[alpha, beta]` re-search is needed to get its true value.

> With a good move ordering we expect to save about 10% of a search
> effort [...] fail-highs on the first move approximately 90% of the
> time.

## Pseudocode adopted (fail-soft, matches Zeus's existing convention)

```
Call from root:
   rootscore = PVS(-infinite, infinite, depthleft);

int PVS(alfa, beta, depthleft) {
   if (depthleft <= 0) return qsearch(alfa, beta);

   // using fail soft with negamax:
   make first move
   bestscore = -PVS(-beta, -alfa, depthleft-1);
   unmake first move
   if (bestscore > alfa) {
      if (bestscore >= beta)
         return bestscore;
      alfa = bestscore;
   }

   for (all remaining moves) {
      make move
      score = -PVS(-alfa-1, -alfa, depthleft-1); // alphaBeta or zwSearch
      if (score > alfa && score < beta) {
         // research with window [alfa;beta]
         score = -PVS(-beta, -alfa, depthleft-1);
         if (score > alfa)
           alfa = score;
      }
      unmake move
      if (score > bestscore) {
         if (score >= beta)
            return score;
         bestscore = score;
      }
   }
   return bestscore;
}
```

## Caveats

- **Move-ordering dependent**: the whole technique only saves work if the
  first move really is usually best — Zeus already has hash-move / MVV-LVA
  / killer ordering in place (`Reference/search/Move_Ordering.md` and
  friends), the prerequisite this page assumes.
- **Aspiration windows**: "one must be aware that in this case also a
  normal window search might fail, leaving the program with no move and
  no PV" — already handled in Zeus by `search_root_once`'s existing
  fallback-to-full-window-after-retries logic
  (`Reference/search/Aspiration_Windows.md`), unchanged by this stage.
- Shallow-depth disabling of PVS is mentioned as something "programs
  respond differently to" — no clear recommendation, so Zeus applies PVS
  uniformly at every depth rather than guessing at a cutoff.

## Zeus usage

Both `crates/zeus-engine/src/search.rs::alpha_beta` and
`::search_root_pass` restructured to this exact pattern: first move full
window, remaining moves null-window with a conditional full re-search.
Not applied inside `quiesce` (out of scope for this stage, same footing
as the transposition table and killer heuristic before it — quiescence
already narrows its own search via stand-pat and MVV-LVA ordering, and
this page gives no specific guidance on qsearch integration). Null move
pruning (`Reference/search/Null_Move_Pruning.md`) is unaffected — it runs
before the move loop begins, orthogonal to how the loop itself searches
moves once reached.
