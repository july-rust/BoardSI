# M0 scalar numerical contract (version 0.1)

Scope: external, uniform, scalar circuit RLGC -> single-ended 2-port only.
Specification v1.0 §§8, 8.2, 11, 13.1, 14.2, 15 is unchanged. Related #3,
#4, #16 remain open. No input-format dependency; ODB++ is only one future importer.
This crate neither defines nor replaces the Canonical Board Model.

## Inputs and conventions

- R [Ω/m], L [H/m], G [S/m], C [F/m], length [m], frequency f [Hz].
  Scalar circuit values, not multi-conductor Maxwell matrices. R,G >= 0;
  L,C > 0; length,f >= 0. All inputs must be finite binary64.
- Values apply at the requested frequency. Caller supplies dispersion and loss
  parameters; no skin effect, material, or geometry extraction is inferred.
- omega = 2*pi*f [rad/s]. Time exp(+j*omega*t), forward propagation exp(-gamma*x).
  gamma = alpha+j*beta [1/m], alpha >= 0, beta >= 0 for f >= 0.
  Delay beta*length/omega is a phase delay; group delay is not implemented.
- Port 1 at x=0, port 2 at x=length; currents into the network. S[row][column]
  is outgoing wave at row / incident wave at column (zero-based Rust indices).
  S[1][0] = S21. Both references are the SAME positive real Zref [Ω].
  a=(V+Zref*I)/(2*sqrt(Zref)), b=(V-Zref*I)/(2*sqrt(Zref)).
  Unequal or complex reference impedances are unsupported by this API.
- Complex real/imaginary parts are f64. No f32, magnitude-only comparison, phase
  unwrap, fitted response, or automatic passivity correction.

## Propagation and stable scattering

Let z=R+j*omega*L, y=G+j*omega*C. Passive first-quadrant square roots sz=sqrt(z),
sy=sqrt(y) select Zc=sz/sy and gamma=sz*sy. This avoids first multiplying z*y or
z/y. Compute alpha as
`0.5*(R*(Im(y)/beta)+G*(Im(z)/beta))` to avoid subtracting nearly equal
root products and to avoid underflow of R*Im(y) or G*Im(z). Divide before
multiplying. When beta has rounded to zero at a positive AC frequency, retain
the directly computed positive real root instead of dividing by zero. For R=G=0 explicitly use
Zc=sqrt(L)/sqrt(C), beta=omega*sqrt(L)*sqrt(C), alpha=0, including DC.

Define q=Zc/Zref, rho=(q-1)/(q+1), t=exp(-gamma*length), u=1-t^2,
a0=1-rho^2, d=a0+rho^2*u. Then

- S11=S22=rho*u/d
- S21=S12=a0*t/d

u uses exp_m1 for attenuation and 2*sin(beta*length)^2 for 1-cos(2*beta*length).
No growing exp(+alpha*length), cosh, sinh, or cascading unstable ABCD matrices
is used in production. The oracle deliberately uses ABCD instead.

## Edge policy and conservative numerical envelope

| Condition | Policy |
| --- | --- |
| Invalid/nonfinite inputs | InvalidInput, no S values |
| Zero length after validating every input | Ideal thru, Zc/gamma=None; includes lossy DC |
| Lossless DC, nonzero length | Exact DC limit: gamma=0, Zc=sqrt(L/C), ideal thru |
| Lossy DC, nonzero length | Unsupported; distributed resistive/shunt DC model deferred |
| Degenerate L=0 or C=0 | InvalidInput; RL/RC-only networks not this model |
| Lossy series/shunt magnitude outside [1e-120,1e120] | Unsupported |
| abs(Zc/Zref) outside [1e-6,1e6] | Unsupported |
| alpha*length > 700 or beta*length > 1e6 rad, nonfinite product | Unsupported |
| abs(scattering denominator) <= 1e-12 | NumericalInstability, no result |
| 64*epsilon*(1+abs(gamma*length))/abs(denominator) > 1e-7 | NumericalInstability; conservative sensitivity guard |
| Nonfinite propagation/S, nonpassive output residual > 1e-6 | NumericalInstability |

These limits protect numerical evaluation; they do NOT establish a verified
physical frequency/geometry/material range. Underflow of extremely small
individual S components may occur within f64; no relative accuracy promise is
made for arbitrary near-zero values. Matched exp(-700) is explicitly tested.
The envelope and sensitivity guard are conservative; the guard is a heuristic,
not a formal forward-error certificate. Long, severely mismatched resonant lines
can be rejected even within the individual limits. Errors do not fall back to
plausible values.
Complex arithmetic is a small local scalar helper, not a reusable arbitrary
range numerical backend; zero-denominator complex division is undefined.

## Evidence and statuses

All successful API results carry **Approximate** physical model status because
external RLGC provenance and applicability are unknown. Passing numerical tests
never promotes an arbitrary physical line to Validated. Validated is reserved
for future structure/material/band/reference records with independent physical
evidence (§8.2); no API path emits it today. Unsupported is conveyed by an
Unsupported error with a reason and no response matrix. InvalidInput and
NumericalInstability are execution errors, distinct from physical applicability.

The report records **numerical** PASS/FAIL/Not Run separately. The maximum singular
value is sqrt(lambda_max(S^H*S)); passivity residual=max(0,sigma_max-1).
Reciprocity residual=max_ij abs(Sij-Sji), equal to abs(S12-S21) for 2-port.
Per-point results preserve errors; at_frequencies is only a convenience map,
not Progressive Sweep, scheduler, cache, or convergence certification.

## Independent reference derivation and licenses

Telegrapher equations: dV/dx=-z*I, dI/dx=-y*V. Define output current flowing
along +x, so [V1,I1]^T = exp([[0,z],[y,0]]*length)*[V2,-I2]^T.
The generator's square gives z*y times identity, giving
A=D=cosh(gamma*length), B=Zc*sinh(gamma*length), C=sinh(gamma*length)/Zc.
Substituting the power-wave definitions yields denominator A+B/Zref+C*Zref+D,
S11=(A+B/Zref-C*Zref-D)/denominator and S21=2/denominator. Symmetry and
reciprocity give S22=S11, S12=S21. This derivation is self-contained, not a
copied external implementation. The half-wave and distortionless analytic
checks provide additional independent sign/delay/attenuation anchors.

`tools/validation/generate_reference.py` uses independent Python mpmath at
80 decimal digits, serializes 45-digit ABCD values, and never calls the Rust
code. Ordinary cargo test uses the committed CSV without Python or networking.
The 72 fixed synthetic cases span mismatched/matched/lossless/lossy lines,
1 Hz..30 GHz, extremely short lines, high attenuation, and non-50 Ω reference.
They are evidence at those points, not a proof across the entire envelope.

No third-party Rust dependencies; Rust standard library/toolchain licenses:
https://www.rust-lang.org/policies/licenses . Generator-only mpmath 1.2.1 is BSD
licensed: https://mpmath.org/ and https://github.com/mpmath/mpmath/blob/1.2.1/LICENSE .
No mpmath source is redistributed. Equations, generator and synthetic CSV are
new project work. Repository-wide source license is not selected by this PR;
release licensing remains the §15 decision gate.
