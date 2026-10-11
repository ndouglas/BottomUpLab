# W1-A: prospective finite-engagement measurements

Date: 2026-10-10. Status: **proposed design; not registered or authorized for collection**.

This proposal continues the delivered [W1 engineering benchmark](../../war.md). It
preserves the original [design](2026-10-09-war-1-engagement-design.md),
[implementation plan](../plans/2026-10-09-war-1-engagement.md) and
[engineering amendment](2026-10-10-war-1-engineering-amendment.md).
The concrete [arm roster](2026-10-10-war-1-measurement-proposal.json) is a design
artifact. It contains no sampled scientific seeds, binary declaration or activation.
No ensemble has been collected for this proposal.

## Intent, alternatives and success

The user's next step is to obtain measured benchmark results after engineering
verification. Address geometry, force/rate imbalance, population size, timestep
and termination. Success means a complete, prospective, source-bound population
whose saved evidence supports honest finite-model comparisons and uncertainty.
The study does not introduce movements, morale, supply, economy, recruitment,
retreat or historical time calibration. Its rates/time remain dimensionless.

Three approaches were considered:

1. **Recommended: bounded comparison plus exact tiny-count references.** Use
   54 comparative/mirror arms and 12 small-law arms. This separates a count-law
   conformance question from timestep changes and continuum deviations, at a
   fixed work/output ceiling.
2. **Full factorial:** add all scale/profile/timestep interactions, giving
   130 arms and 84,992 runs. Its upper bound is 5,100,142,592 actor-steps. Defer
   this extra breadth until separately proposed; it is not authorized as fallback.
3. **Tiny-law study only:** use the 12 oracle arms. It is cheaper and sharpens
   the mutual-extinction question, but does not measure population scale or
   general geometry/imbalance response.

The recommended design is descriptive for general contrasts. Small differences,
particularly near equality, may remain unresolved. More repetitions reduce seed
sampling uncertainty; they do not remove discretization or finite-population
bias. At fixed finite counts, an ensemble mean is not generally the continuous
ODE solution. In duels, E[min(B,R)] need not equal min(E[B],E[R]); absorption also
changes the stopped aimed process. This distinction accords with the discussion
of stochastic versus deterministic averages in [Gillespie 1977, introduction,
p. 2340](https://www.ma.imperial.ac.uk/~nsjones/gillespie_1977.pdf). It is conceptual
background, not an empirical battle law or a theorem applied without conditions.

## Exact proposed population

Both geometries use H=8 and observation times 1, 2, 4, 8. Timestep values are
1/16, 1/32, 1/64; horizons are exactly 128, 256, 512 settlements. These quantities
and rates below are exactly representable in binary64. N is divisible by 16.

| Profile | B0 | R0 | Blue rate b | Red rate r | Purpose |
|---|---:|---:|---:|---:|---|
| E | N | N | 1 | 1 | Exact equality |
| NC | 17N/16 | N | 1 | 1 | Fixed near-equal count ratio |
| NR | N | N | 1 | 15/16 | Near-equal rate ratio |
| C | 2N | N | 1 | 1 | Clear count advantage |
| L | 2N | N | 1/2 | 1 | Matched-reference balance |
| S | 2N | N | 1/4 | 1 | Aimed-reference balance |

- Core: six profiles, N=64, both geometries, three dt: 36 arms × 512 runs.
- Scale: E/C only, N=16/256, both geometries, dt=1/64: 8 × 512.
- Mirror: NC/NR/C/L/S at N=64 and dt=1/64, both geometries, swap counts/rates:
  10 × 512. Replicates are independent across arms; no same-seed pairing.
- Oracle: 1-versus-1 and 2-versus-1, b=r=1, both geometries, three dt:
  12 × 2048.

Total: 66 arms, 52,224 runs. The JSON roster fixes the ordering, resolved configs,
arm IDs, replicate counts and namespace. It is the machine-readable census
against which a future manifest factory must be checked; it is not an executable
registration. Unlisted profile/scale/dt interactions remain unmeasured. NC keeps
17:16 across scales; it is not a +1 perturbation whose relative size changes.
L/S distinguish the two mathematical invariants; no deterministic winner is
assigned to a stochastic replicate from an invariant sign.

Empty sides, rate zero, one-sided rates, forced final-step extinction, supplied
coupled side swaps, invalid numeric inputs and failed output are engineering
controls. They use construction seeds 7/8 or supplied outcomes, remain separately
labeled, and add no stochastic scientific arms. Literal World presets/goldens
remain regression controls; their tick/economic semantics are not pooled with
graph durations or presented as a graph comparison population.

## Seed population and prospective freezing

The primary target is the distribution of the deterministic native Engine(config,
seed) for seed uniform over all unsigned 64-bit values. Before any scientific
outcome, prepare independent OS-uniform u64 draws **with replacement**, separately
for each arm and replicate. Freeze every actual value in the committed manifest.
Use the roster's namespace as an identifier, not a constraint on high bits or a
hashed-root generator. Record sampler command/version and OS entropy interface.
Draw no seeds in this design turn. Retain duplicate values if they occur; keys
are (arm ID, replicate index), so sampling with replacement and its denominator
are preserved. Reject duplicate keys, not repeated values. No redrawing by value,
interim stopping, successful-only subset or post-result horizon extension.

This randomized input design supports design-based inference for the u64-seed
population. It does not assume PCG words are independent ideal uniforms. The
ideal-word finite-process law below is a separate comparison target. A discrepancy
may reflect transition/numerical/RNG-distribution differences; agreement does
not prove RNG independence or complete joint-path equivalence. A deterministic
counter/hashed seed census would be a different, descriptive registration and
cannot inherit these design-based confidence claims.

All engineering inspections and known analytical predictions are disclosed.
This is prospective, not blind. No claim of a holdout is made. Kernel FNV stream
seeding, RNG algorithm, shuffle and casualty ordering stay unchanged.

## Independent mathematical comparisons

Three quantities must remain distinct: mathematical p*=1-exp(-h dt), native
stored binary64 p, and realized q=ceil(p·2^53)/2^53. Native-law predictions use q.
The quantization bound q-p<2^-53 does not bound libm error relative to p*.

### Small-state discrete law

Under an ideal independent 53-bit-word model, conditional losses are independent:

- Aimed: D_B~Binomial(B,q_B), h_B=rR/B; D_R~Binomial(R,q_R), h_R=bB/R.
- Duel: m=min(B,R); D_B~Binomial(m,q_r), D_R~Binomial(m,q_b).

The joint count transition is the product of those binomial masses. Empty-side
states absorb. Horizon applies only after settlement K; extinction on K remains
an observed extinction. The tiny grids need at most six states. This law
marginalizes homogeneous matching; it does not validate IDs, unmatched actors,
stream independence or supplied-outcome equivariance.

Implement the transition/propagation independently of Engagement, its matching,
probability helper and RNG. A no-simulation native export supplies typed binary64
hazard/dose/p bit patterns for each reachable tiny state. The independent checker
recomputes q from those stored p bits; q is an exact dyadic rational. Certify
state propagation with exact rational arithmetic and outward-bounded conversion,
with total interval width ≤1e-10 for each registered normalized quantity and
probability mass enclosing one. If the certificate fails, stop preparation.

Separately compare native p to an independently evaluated high-precision real
exponential at the exact native binary64 dose. Require error ≤4 ULP, and bound
dose-rounding error against exact rational rate/count/dt arithmetic separately.
For the tiny registered states (1,1) and (2,1), the exact rational hazard/dose
uses only integer or half-integer hazards and dyadic dt: require native dose bits
to equal that exact rational value, not merely fall below an unspecified error
bound. Define one ULP as the larger spacing to either adjacent binary64 value
around the uniquely correctly rounded p*. Refine the certified p* interval if
its nearest binary64 value is ambiguous; unresolved rounding blocks preparation.
The entire native-p versus p* interval must fit within four such ULPs.
For these positive moderate doses, [stdlib Decimal.exp](https://docs.python.org/3/library/decimal.html#decimal.Decimal.exp) at ≥100 digits plus its
rounding envelope can supply the independent calculation; implementation must
verify the numerical contract and add rounding-boundary/underflow fixtures.
A four-ULP check is a local engineering arithmetic criterion, not historical fit
or proof of correct rounding for all libm inputs. Record intervals, p bits and q
numerators in the preparation packet. Do not substitute p* for the native q law.

### One versus one: the key timestep question

For native q_B,q_R let s=(1-q_B)(1-q_R), K=H/dt and
A_K=(1-s^K)/(1-s), with A_K=K when s=1. Exact horizon probabilities are:

- Blue survivor: (1-q_B)q_R A_K.
- Red survivor: q_B(1-q_R) A_K.
- Double extinction: q_Bq_R A_K.
- Administrative censoring: s^K.

Restricted mean duration is dt·A_K. For mathematical equal rates k, the eventual
mutual-extinction probability is tanh(k dt/2), tending to zero as dt shrinks.
This is a derivation, not a newly measured result; it highlights that fair
simultaneous settlement can still produce timestep-dependent mutual destruction.

The one-versus-one continuous-time competing-clock reference has departure rate
b+r, zero double extinction, censoring exp(-(b+r)H), blue-survivor probability
b/(b+r)·(1-exp(-(b+r)H)), mirrored red probability, and restricted mean
(1-exp(-(b+r)H))/(b+r). Both geometries coincide in this case. At dt=1/64 require
absolute differences ≤0.02 between exact-q discrete and continuous-time formulas
for the four horizon categories and restricted mean/H. Report every dt and all
individual margins regardless of pass/fail. This criterion applies only to this
case; it is neither a Monte Carlo equivalence test nor a global timestep bound.

General finite-state CTMC solution and claims of general quantitative dt bias
are deferred. Elsewhere report fixed-dt differences and ODE deviations, retaining
the independent continuous references and their availability reasons. Do not
require stochastic paths to conserve an ODE invariant or observed errors to fall
monotonically over three meshes.

## Outcomes, observation conventions and failures

For each replicate save four count observations, initial/final counts, native
ending/winner/censoring, completed and active clocks, totals of casualties and
integrated contributed exposure by side, and reference availability. Save final
checkpoint/RNG receipt and a versioned hash of the canonical semantic frame
stream, computed without drawing any RNG. Define that transcript byte recipe
before implementation; it is replay-checkable, not retained full raw evidence.

For a requested time after an observed ending, derive a constant absorbing
survivor count in analysis with origin `derived_absorbing`; retain the native
ending time and unavailable future-frame reason. Do not fabricate additional
native frames. ODE forces are held at their own declared boundary. All replicates
contribute to unconditional trajectory means, including extinct/censored ones.

Define T as first step-boundary extinction, U=min(T,H). Report mean U/H and cause
probabilities. At t=H use the explicit censor flag for P(T>H); U's atom at H mixes
actual final-step extinction with administrative censoring. Never infer the
unrestricted tail or average only completed extinctions as the overall duration.
Rate-zero controls have unavailable extinction time, not biological duration zero.

A kernel/observation failure, torn write or wrong source binding is a data-quality
failure, not a win, draw, extinction or horizon. Keep planned, attempted,
complete-valid, failed, invalid, partial, pending and unstarted censuses. Any
missing/invalid registered outcome makes primary ensemble inference unavailable;
never shrink n or replace seeds. A reference failure affects its comparison
endpoint only, with the same planned denominator and explicit reason; it does
not erase a valid finite outcome. A normalized quantity with an unavailable or
zero denominator is unavailable with a reason. Restore/resume/retry of scientific
histories is outside this first design; interruption preserves evidence and
leaves the campaign incomplete.

## Statistical family and precision

Fix 14 scalar means per arm: Blue/B0 and Red/R0 at four times (eight), five
terminal-category indicators (Blue survivor, Red survivor, double extinction,
rate_zero, horizon), and U/H. All lie in [0,1]. Fix three CDFs per arm: U/H and
both terminal survivor fractions. Even structural zero slots remain in the
family. Thus M=924 scalar slots and C=198 CDF slots.

Allocate α=0.025 to each family. For arm sample size n use

h_n=sqrt(log(2M/0.025)/(2n)); d_n=sqrt(log(2C/0.025)/(2n)).

Evaluate empirical means from integer counts/clocks as exact rationals. Compute
radii and reported bounds with outward rounding; the implementation must enclose
the exact formulas, not narrow a band through binary64 rounding. Clip scalar
intervals mean±h_n and CDF bands Fhat±d_n to [0,1]. Hoeffding and a
union bound supply simultaneous scalar coverage; DKW plus a union bound supplies
CDF coverage uniformly over the real threshold. Within-run correlation is
allowed. Independence is the seed-sampling design across replicates. Sources:
[Hoeffding 1963, Theorem 2](https://www.cs.rpi.edu/academics/courses/spring06/random/hoefding.pdf),
[Reeve 2024, Theorem 1/Corollary 1](https://arxiv.org/html/2403.16651v1) and
[Massart 1990](https://doi.org/10.1214/aop/1176990746).

At n=512 these radii are 0.104633 and 0.097178; at n=2048 they halve. General
near-equality comparisons are therefore descriptive and coarse. A 5-point mean
contrast equivalence claim would need at least 8,969 repetitions per arm merely
to fit its conservative interval inside that margin; this is not a power promise.
No such equivalence claim is registered here.

Report the fixed contrasts: NC−E, NR−E, C−E, L−C and S−C at N=64,
separately for every geometry and dt; both geometries within every common profile/N/dt;
adjacent meshes at N=64; each N=16/256 arm against its N=64 counterpart at finest
dt; and mapped original/mirror variables. Each contrast interval derives from
simultaneous arm intervals, with halfwidth h_a+h_b. The JSON fixes 121 scalar-contrast groups, left minus right, each evaluated for
all 14 mapped scalar endpoints. Mirrors map original Blue to mirror Red and
original Red to mirror Blue, including the two survivor-category indicators;
other categories and U/H stay unchanged. Every E/1-versus-1 arm also has a
within-arm Blue/Red mapping contrast. Endpoint IDs and CDF support definitions
are authoritative in the roster. Radius decimals there are display-only; derive
certified bounds from the formulas above. Cross-arm comparisons are unpaired. Same-arm symmetry uses the simultaneous
marginal bands, allowing within-run dependence. The roster fixes 46 CDF
comparisons: three mapped endpoints for each mirror pair, plus Blue/Red terminal
fractions in every E/1-versus-1 arm. Evaluate right-continuous CDF sup distances
over the union of exact rational support points, including 0/1 endpoints. Compare
mapped side CDFs by sup distance with envelope d_a+d_b; nonrejection does not
establish full distributional/pathwise symmetry.

For tiny-law conformance, compare all 14 scalars and three CDFs in each oracle
arm against certified q-law predictions. Observed disagreement exceeding the
registered band plus oracle numerical envelope is `incompatible_with_ideal_word_law`;
otherwise `not_rejected`, never `proved_equal`. Under the joint null this has
the stated familywise false-rejection bound. Nonfinite/unavailable math gives an
unavailable comparison, not an adjusted threshold. General geometry, scale, ODE
and mesh results are descriptive estimates with their full uncertainty.

## Bounded collection tooling and provenance

The existing engineering `war1` CLI remains unchanged and unregistered. Its
capture setting still emits every frame and fsyncs each one; multiplying those
journals into this population is unsuitable. Specify a separate native
`war1-study` collector and standalone registration/analysis tooling. Prefer the
public Engagement new/step/checkpoint interfaces for graph collection; an
immutable compact observer cannot change RNG, matching or settlement. Compare
its engineering runs at seeds 7/8 exactly against the existing run_to frames,
clocks and checkpoints. Do not modify World or the kernel to accelerate science.

Use source-bound create-only archives and durable start/outcome receipts, one
compact outcome per replicate. A completed simulation, successful callback and
durable acknowledgment remain distinct. Sync the attempt-start receipt before
constructing the engine; sync a complete outcome before acknowledging it. Rotate
fixed blocks at 128 replicate keys, with immutable block digests/acknowledged
prefixes. A torn tail or crash-after-start retains its prefix and partial key;
no automatic retry or replacement. A final index requires the exact complete
key roster, all row/accounting/source validations and synchronization. Semantic
terminal records do not declare artifact completion.

Store one full audit trajectory for replicate index 0 of every arm, chosen by
index before outcomes. Preserve every audited casualty ID/frame and requested
capture reason. Do not choose representative-looking histories later. Compact
non-audit rows and transcript hashes are explicitly not a lossless raw archive.
Analysis uses saved outcomes only and cannot invoke the simulator or write over
existing reports. Save every registered estimate/contrast/availability, not a
significance-selected subset. Publication remains a separate reviewed action.

The scientific compiled closure must include new collector/registration/analysis
sources, normative methods, this protocol, approved roster, frozen actual-seed
manifest, engineering contracts/locks and all literal core compiled inputs.
Keep existing F5/BT and engineering-case selectors separate. Freeze committed
sources after writers stop, build and independently verify the declared closure,
resolved config roster, numeric inputs/oracles, source inventory, toolchain/flags
and actual executable SHA. The final declaration refers to these actual artifacts;
it and subsequent acceptance/execution receipts are excluded from their own
compiled inventory to prevent a binary/declaration hash cycle. Declared hashes
are reproducibility identifiers, not proof of identity or historical authenticity.

Collection requires the exact declaration, genuine independent prospective
acceptance and explicit execution authorization for that exact workload. Typed
receipt validation cannot create permission from a supplied approval string.
No scientific engine construction or output creation precedes this preflight.
The output path is new; symlinks/nonregular inputs and existing destinations are
refused. Verify bindings again after collection and before final index creation.

## Work, memory, output and stopping budgets

| Component | Arms | Runs | Maximum settlements | Initial actor-steps |
|---|---:|---:|---:|---:|
| Core | 36 | 18,432 | 5,505,024 | 884,473,856 |
| Scale | 8 | 4,096 | 2,097,152 | 713,031,680 |
| Mirror | 10 | 5,120 | 2,621,440 | 438,304,768 |
| Oracle | 12 | 24,576 | 7,340,032 | 18,350,080 |
| Total | 66 | 52,224 | 17,563,648 | 2,054,160,384 |

Every replicate is charged its full horizon, not predicted early ending.
Maximum initial population is 768; maximum per-case actor work is 393,216.
All existing core limits hold. Conservatively, positive reachable doses lie
between 1/131072 and 32, well above sampler resolution and below saturation;
preparation must independently verify tighter actual-arm bounds over every
interior count ratio. Quantization alone has common-uniform path discrepancy
bound L·2^-53, at most 4.37e-11 per case here. It does not cover libm or RNG error.

Proposed new archive caps: 32 KiB compact outcome, 1 KiB durable start, 32 KiB
audit frame, 16 MiB metadata/manifest file, 4 GiB total uncompressed archive.
The total includes source/binary/numeric/build/declaration/evidence artifacts,
audit headers/initial/terminal records and failure reserves. Compact starts and
outcomes contribute at most 1,764,753,408 bytes; audit settlement frames at most
66×512×32768=1,107,296,256 bytes. Reserve the remaining >1 GiB for provenance,
audit envelopes/metadata/index/failures. Before activation, prove actual encoded
worst-case lengths for the complete schemas against all caps and validate the
exact total; do not depend on compression or enlarge caps after observing data.
Missing this proof prevents activation. Per-field string/array/count bounds and
failure-receipt size must be explicit in the implementation plan.

One worker, 2 GiB RSS, 12-hour collection wall cap. These are policy ceilings,
not a measured runtime forecast. Preparation uses only construction/synthetic
fixtures to check resource behavior; registered seed outputs stay unopened.
Require a conservative preparation forecast within the ceilings before acceptance.
Actual resource/byte-limit failure stops with the existing complete prefix,
marks the campaign incomplete and makes primary inference unavailable.
Do not confuse an administrative resource stop with a native horizon ending.

## Review and execution sequence

1. Review this written design and proposed arm roster. Nothing is registered.
2. Write/review an implementation plan; use subagent-driven execution as already
   requested. Implement/test preparation, compact collection and analysis with
   construction/synthetic fixtures; preserve old studies and engineering APIs.
3. Prepare the actual randomly sampled seed manifest, freeze sources and build
   the source-bound binary. Commit the manifest before collecting any outcomes.
   Produce the concrete source/binary/numeric/resource declaration packet.
4. Independent prospective readiness review, then explicit exact-workload
   execution approval. A software code-review pass does not substitute for either.
5. Collect once, validate the complete census, reanalyze saved rows, independently
   review findings, and obtain publication approval if public evidence is desired.

No original source-bound specification or old registration is rewritten. Any
method/source/arm/budget correction after freezing receives a new dated identity
and review. Known math, prior engineering and separate foraging work are disclosed;
no claim of blinded discovery, historical fit or universal conflict regularity
follows from this stationary benchmark.
