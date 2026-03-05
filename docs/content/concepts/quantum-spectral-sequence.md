# Quantum Spectral Sequence

## Definition

The **quantum level spectral sequence** is an algebraic machinery for deciding
whether a {@class https://uor.foundation/type/CompleteType} at quantum level Q_n
can be lifted to Q_{n+1} without losing completeness. A
{@class https://uor.foundation/type/QuantumLift} record represents the candidate
lift: it carries a {@prop https://uor.foundation/type/liftBase} (the Q_n
CompleteType), a {@prop https://uor.foundation/type/liftTargetLevel} (the target
QuantumLevel), and a {@prop https://uor.foundation/type/liftObstruction} link.

## Obstruction Theory

A {@class https://uor.foundation/type/LiftObstruction} captures the algebraic
obstruction to the lift. If
{@prop https://uor.foundation/type/obstructionTrivial} is true the lift succeeds;
otherwise it is localised to a single fiber coordinate
({@prop https://uor.foundation/type/obstructionFiber}) and represented as a
{@class https://uor.foundation/observable/LiftObstructionClass} — a cohomology
class in H²(N(C(T)); ℤ/2ℤ).

## Spectral Sequence Pages

The convergence of the obstruction computation is tracked page by page. Each
{@class https://uor.foundation/observable/SpectralSequencePage} carries:

- {@prop https://uor.foundation/observable/pageIndex} — the page index r (r=1
  is the initial page).
- {@prop https://uor.foundation/observable/differentialIsZero} — whether all
  differentials on this page vanish.
- {@prop https://uor.foundation/observable/convergedAt} — the page index at
  which the sequence collapsed (only set on the final page).

## Incremental Resolver

The {@class https://uor.foundation/resolver/IncrementalCompletenessResolver}
determines lift completeness without re-running the full ψ-pipeline from
scratch. When the obstruction is non-trivial it returns a
{@class https://uor.foundation/resolver/LiftRefinementSuggestion} carrying
{@prop https://uor.foundation/resolver/liftFiberPosition} and
{@prop https://uor.foundation/resolver/obstructionClass}.

## Identity Algebra (QLS\_ series)

| Identity | Statement |
|----------|-----------|
| QLS\_1 | QuantumLift T' is CompleteType iff spectral sequence collapses at E₂ |
| QLS\_2 | LiftObstruction is localised to single fiber at position n+1 |
| QLS\_3 | basisSize(T') = basisSize(T) + 1 for a closed constraint set |
| QLS\_4 | Spectral sequence converges by E\_{d+2} for depth-d configuration |
| QLS\_5 | Every universally valid identity holds with lifted constraint set |
| QLS\_6 | ψ-pipeline ChainComplex(T') restricts to ChainComplex(T) |

## Related

- {@class https://uor.foundation/type/QuantumLift}
- {@class https://uor.foundation/observable/SpectralSequencePage}
- {@class https://uor.foundation/resolver/IncrementalCompletenessResolver}
- [Quantum Universality](quantum-universality.html)
- [Type Completeness](type-completeness.html)
