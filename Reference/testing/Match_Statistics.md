---
source: https://www.chessprogramming.org/Match_Statistics
retrieved: 2026-10-23
---

# Match Statistics

## Elo <-> expected score

```
E = 1 / ( 1 + 10^(-Δ/400) )
```

`E` = expected score (win probability against a draw-less opponent, or
expected fractional score more generally) for the player who is `Δ` Elo
points stronger. Inverting it gives the Elo difference implied by an
observed score:

```
Δ = 400 * log10( E / (1 - E) )
```

## Basic match counting

```
N      = wins + draws + losses
score  = wins + draws / 2
win_ratio  = score / N
draw_ratio = draws / N
```

`score / N` is exactly the `E` used in the inversion formula above, so a
single finished match directly yields a performance-rating estimate relative
to the (known) opponent strength: `Δ = 400 * log10(E/(1-E))`, and
`engine_elo = opponent_elo + Δ`.

## Likelihood of Superiority (LOS)

For a two-outcome (win/loss, draws excluded) significance test against the
null hypothesis of equal strength:

```
LOS = Φ( (wins - losses) / sqrt(wins + losses) )
```

equivalently `LOS = 1/2 * [ 1 + erf( (wins - losses) / sqrt(2*wins + 2*losses) ) ]`.
Draws are excluded from this computation and it does not matter which color
each game was played with.

## SPRT

Sequential Probability Ratio Test: a log-likelihood-ratio test between a null
hypothesis H0 (Elo difference = elo0) and alternative H1 (Elo difference =
elo1), with error bounds alpha (false-positive rate) and beta
(false-negative rate), that lets a test stop early once enough evidence has
accumulated either way, rather than requiring a fixed, large game count.

## Zeus usage

`zeus-bench` computes, for every calibration run against a fixed-Elo
Stockfish opponent: `N`, `score`, `E = score/N`, `Δ = 400*log10(E/(1-E))`, and
reports `estimated_elo = opponent_elo + Δ` together with the 95% Wald
confidence interval on `E` (standard normal approximation,
`E ± 1.96*sqrt(E(1-E)/N)`, propagated through the same Δ formula) so every
printed Elo number carries an explicit uncertainty band rather than a bare
point estimate. This is why the harness needs "a statistically significant
number of matches" — the width of that interval is what determines whether N
games were actually enough to resolve a claimed Elo gain between two engine
versions.
