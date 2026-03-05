# Incremental Completeness Guide

The {@class https://uor.foundation/resolver/IncrementalCompletenessResolver}
determines whether a {@class https://uor.foundation/type/CompleteType} at
quantum level Q_n lifts to Q_{n+1} without re-running the full ψ-pipeline from
scratch.

## When to Use This Resolver

Use `IncrementalCompletenessResolver` instead of the full
{@class https://uor.foundation/resolver/CompletenessResolver} when:

- You already have a certified CompleteType at Q_n.
- You want to promote it to Q_{n+1} (the next quantum level).
- Re-running the full ψ-pipeline would be prohibitively expensive.

Identity QLS\_4 guarantees the spectral sequence converges by page E\_{d+2}
where d is the constraint depth, making the incremental approach tractable.

## Step 1 — Declare a QuantumLift

```turtle
@prefix type:     <https://uor.foundation/type/> .
@prefix schema:   <https://uor.foundation/schema/> .
@prefix owl:      <http://www.w3.org/2002/07/owl#> .

type:my_lift a owl:NamedIndividual, type:QuantumLift ;
    type:liftBase         type:my_q0_complete_type ;
    type:liftTargetLevel  schema:Q1 .
```

## Step 2 — Run the IncrementalCompletenessResolver

```turtle
@prefix resolver: <https://uor.foundation/resolver/> .

resolver:my_icr a owl:NamedIndividual, resolver:IncrementalCompletenessResolver ;
    resolver:liftTarget  type:my_lift .
```

The resolver computes the {@class https://uor.foundation/observable/SpectralSequencePage}
sequence. When {@prop https://uor.foundation/observable/differentialIsZero}
becomes true at page r, the sequence has converged (QLS\_4) and
{@prop https://uor.foundation/observable/convergedAt} is set to r.

## Step 3 — Inspect the LiftObstruction

The lift carries a {@class https://uor.foundation/type/LiftObstruction}:

- If {@prop https://uor.foundation/type/obstructionTrivial} is `true`: the lift
  is complete — identity QLS\_1 applies.
- If `false`: the obstruction is a non-zero cohomology class localised to fiber
  position n+1 (QLS\_2). The resolver returns a
  {@class https://uor.foundation/resolver/LiftRefinementSuggestion} with:
  - {@prop https://uor.foundation/resolver/liftFiberPosition} — the fiber to
    target.
  - {@prop https://uor.foundation/resolver/obstructionClass} — the
    {@class https://uor.foundation/observable/LiftObstructionClass} to kill.

## Step 4 — Handle a Non-Trivial Obstruction

A non-trivial obstruction means the type is a
{@class https://uor.foundation/type/TwistedType}. Per identity MN\_7,
TwistedType implies a non-zero class in H²(N(C(T')); ℤ/2ℤ). Add the suggested
constraint from `liftFiberPosition`, re-run the synthesis step, and retry the
lift. Identity QLS\_3 guarantees basisSize(T') = basisSize(T) + 1 after a
successful resolution.

## Related

- [Quantum Spectral Sequence](../concepts/quantum-spectral-sequence.html)
- [Monodromy](../concepts/monodromy.html)
- [Type Synthesis Guide](type-synthesis.html)
