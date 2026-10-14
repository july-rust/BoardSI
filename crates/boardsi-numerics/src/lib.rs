//! Uniform scalar RLGC transmission line only. See docs/architecture/numerical-contract.md.
use std::f64::consts::TAU;
use std::ops::{Add, Div, Mul, Neg, Sub};

/// Real and imaginary parts are IEEE-754 binary64. Not a general-purpose algebra backend.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}
impl Complex {
    pub const fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }
    pub fn abs(self) -> f64 {
        self.re.hypot(self.im)
    }
    pub fn conj(self) -> Self {
        Self::new(self.re, -self.im)
    }
    pub fn is_finite(self) -> bool {
        self.re.is_finite() && self.im.is_finite()
    }
    // Only used for nonzero first-quadrant series impedance and shunt admittance.
    fn passive_sqrt(self) -> Self {
        let re = (self.abs() / 2.0 + self.re / 2.0).sqrt();
        Self::new(re, (self.im / 2.0) / re)
    }
}
impl Add for Complex {
    type Output = Self;
    fn add(self, r: Self) -> Self {
        Self::new(self.re + r.re, self.im + r.im)
    }
}
impl Sub for Complex {
    type Output = Self;
    fn sub(self, r: Self) -> Self {
        Self::new(self.re - r.re, self.im - r.im)
    }
}
impl Neg for Complex {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.re, -self.im)
    }
}
impl Mul for Complex {
    type Output = Self;
    fn mul(self, r: Self) -> Self {
        Self::new(
            self.re * r.re - self.im * r.im,
            self.re * r.im + self.im * r.re,
        )
    }
}
impl Mul<f64> for Complex {
    type Output = Self;
    fn mul(self, r: f64) -> Self {
        Self::new(self.re * r, self.im * r)
    }
}
impl Div for Complex {
    type Output = Self;
    fn div(self, r: Self) -> Self {
        // Scaled division avoids squaring a very large or very small denominator.
        let scale = r.re.abs().max(r.im.abs());
        let a = r.re / scale;
        let b = r.im / scale;
        let den = a * a + b * b;
        Self::new(
            ((self.re / scale) * a + (self.im / scale) * b) / den,
            ((self.im / scale) * a - (self.re / scale) * b) / den,
        )
    }
}
const ONE: Complex = Complex::new(1.0, 0.0);
const ZERO: Complex = Complex::new(0.0, 0.0);

/// Per-unit-length scalar circuit values, not Maxwell matrices.
#[derive(Clone, Copy, Debug)]
pub struct Rlgc {
    pub r_ohm_per_m: f64,
    pub l_h_per_m: f64,
    pub g_s_per_m: f64,
    pub c_f_per_m: f64,
}
#[derive(Clone, Copy, Debug)]
pub struct UniformLine {
    pub rlgc: Rlgc,
    pub length_m: f64,
    /// Same positive, real power-wave reference at both ports.
    pub reference_ohm: f64,
}
/// Physical applicability is separate from numerical benchmark evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelStatus {
    Validated,
    Approximate,
    Unsupported,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidInput(&'static str),
    Unsupported(&'static str),
    NumericalInstability(&'static str),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}
pub type SMatrix = [[Complex; 2]; 2];
#[derive(Clone, Copy, Debug)]
pub struct LineResult {
    pub frequency_hz: f64,
    /// None for a zero-length line when propagation is undefined/unsupported.
    pub zc_ohm: Option<Complex>,
    pub gamma_per_m: Option<Complex>,
    /// Row: outgoing port, column: incident port. Port 1 at x=0, port 2 at x=length.
    pub s: SMatrix,
    pub model_status: ModelStatus,
    pub passivity_residual: f64,
    pub reciprocity_residual: f64,
}
impl UniformLine {
    fn validate(self, frequency_hz: f64) -> Result<(), Error> {
        let p = self.rlgc;
        for (v, name) in [
            (p.r_ohm_per_m, "R"),
            (p.l_h_per_m, "L"),
            (p.g_s_per_m, "G"),
            (p.c_f_per_m, "C"),
            (self.length_m, "length"),
            (self.reference_ohm, "reference"),
            (frequency_hz, "frequency"),
        ] {
            if !v.is_finite() {
                return Err(Error::InvalidInput(name));
            }
        }
        if p.r_ohm_per_m < 0.0 || p.g_s_per_m < 0.0 || p.l_h_per_m <= 0.0 || p.c_f_per_m <= 0.0 {
            return Err(Error::InvalidInput("require R,G >= 0 and L,C > 0"));
        }
        if self.length_m < 0.0 || frequency_hz < 0.0 || self.reference_ohm <= 0.0 {
            return Err(Error::InvalidInput(
                "length/frequency >= 0 and reference > 0",
            ));
        }
        Ok(())
    }
    /// Calculate one point; errors never contain fabricated S data.
    pub fn at(self, frequency_hz: f64) -> Result<LineResult, Error> {
        self.validate(frequency_hz)?;
        let p = self.rlgc;
        let lossless = p.r_ohm_per_m == 0.0 && p.g_s_per_m == 0.0;
        // A zero-length passive line is an identity network, even for unsupported lossy DC.
        if self.length_m == 0.0 {
            return Ok(result(frequency_hz, None, None, [[ZERO, ONE], [ONE, ZERO]]));
        }
        if frequency_hz == 0.0 && !lossless {
            return Err(Error::Unsupported("lossy DC model not implemented"));
        }
        let omega = TAU * frequency_hz;
        let (zc, gamma) = if lossless {
            // Explicit DC limit; sqrt(L)/sqrt(C) avoids L/C overflow.
            let zc = Complex::new(p.l_h_per_m.sqrt() / p.c_f_per_m.sqrt(), 0.0);
            let beta = (omega * p.l_h_per_m.sqrt()) * p.c_f_per_m.sqrt();
            (zc, Complex::new(0.0, beta))
        } else {
            let z = Complex::new(p.r_ohm_per_m, omega * p.l_h_per_m);
            let y = Complex::new(p.g_s_per_m, omega * p.c_f_per_m);
            for v in [z.abs(), y.abs()] {
                if !v.is_finite() || !(1e-120..=1e120).contains(&v) {
                    return Err(Error::Unsupported(
                        "series/shunt magnitude outside numerical envelope",
                    ));
                }
            }
            let sz = z.passive_sqrt();
            let sy = y.passive_sqrt();
            let mut gamma = sz * sy;
            // Positive alpha from Im(gamma^2)=2*alpha*beta avoids loss cancellation.
            gamma.re = (z.re * y.im + z.im * y.re) / (2.0 * gamma.im);
            (sz / sy, gamma)
        };
        if !zc.is_finite() || !gamma.is_finite() || zc.re <= 0.0 || gamma.re < 0.0 || gamma.im < 0.0
        {
            return Err(Error::NumericalInstability(
                "nonfinite/invalid propagation constants",
            ));
        }
        let q = zc / Complex::new(self.reference_ohm, 0.0);
        if !q.is_finite() || !(1e-6..=1e6).contains(&q.abs()) {
            return Err(Error::Unsupported("Zc/Zref magnitude outside [1e-6,1e6]"));
        }
        let gl = gamma * self.length_m;
        if !gl.is_finite() || gl.re > 700.0 || gl.im > 1e6 {
            return Err(Error::Unsupported(
                "alpha*length > 700 or beta*length > 1e6 rad",
            ));
        }
        let rho = (q - ONE) / (q + ONE);
        let t = Complex::new(gl.im.cos(), -gl.im.sin()) * (-gl.re).exp();
        // Stable 1-exp(-2*gamma*l); exp_m1 and sin avoid cancellation near l=0.
        let sin_b = gl.im.sin();
        let attenuation2 = (-2.0 * gl.re).exp();
        let u = Complex::new(
            -(-2.0 * gl.re).exp_m1() + attenuation2 * 2.0 * sin_b * sin_b,
            attenuation2 * (2.0 * gl.im).sin(),
        );
        let a = ONE - rho * rho;
        let den = a + rho * rho * u;
        if !den.is_finite() || den.abs() <= 1e-12 {
            return Err(Error::NumericalInstability(
                "ill-conditioned scattering denominator",
            ));
        }
        // Conservative sensitivity guard, not a formal forward-error certificate.
        // Long phase accumulation near a mismatched resonance amplifies f64 error.
        if 64.0 * f64::EPSILON * (1.0 + gl.abs()) / den.abs() > 1e-7 {
            return Err(Error::NumericalInstability(
                "phase/denominator sensitivity exceeds precision budget",
            ));
        }
        let reflection = rho * u / den;
        let transmission = a * t / den;
        let s = [[reflection, transmission], [transmission, reflection]];
        if s.iter().flatten().any(|v| !v.is_finite()) {
            return Err(Error::NumericalInstability("nonfinite scattering response"));
        }
        let out = result(frequency_hz, Some(zc), Some(gamma), s);
        if out.passivity_residual > 1e-6 {
            return Err(Error::NumericalInstability(
                "passivity residual exceeds 1e-6",
            ));
        }
        Ok(out)
    }
    /// Independent point statuses; this is not a sweep scheduler or result cache.
    pub fn at_frequencies(self, frequencies_hz: &[f64]) -> Vec<Result<LineResult, Error>> {
        frequencies_hz.iter().map(|&f| self.at(f)).collect()
    }
}
fn result(
    frequency_hz: f64,
    zc_ohm: Option<Complex>,
    gamma_per_m: Option<Complex>,
    s: SMatrix,
) -> LineResult {
    LineResult {
        frequency_hz,
        zc_ohm,
        gamma_per_m,
        s,
        model_status: ModelStatus::Approximate,
        passivity_residual: (max_singular_value(s) - 1.0).max(0.0),
        reciprocity_residual: (s[0][1] - s[1][0]).abs(),
    }
}
/// Largest singular value from the Hermitian 2x2 Gram matrix S^H S.
/// Scale first; accepts finite matrices. Invalid matrices return NaN.
pub fn max_singular_value(s: SMatrix) -> f64 {
    if s.iter().flatten().any(|v| !v.is_finite()) {
        return f64::NAN;
    }
    let scale = s.iter().flatten().map(|v| v.abs()).fold(0.0, f64::max);
    if scale == 0.0 {
        return 0.0;
    }
    if !scale.is_finite() {
        return f64::INFINITY;
    }
    let mut m = s;
    for v in m.iter_mut().flatten() {
        *v = *v / Complex::new(scale, 0.0);
    }
    let a = m[0][0].abs().powi(2) + m[1][0].abs().powi(2);
    let d = m[0][1].abs().powi(2) + m[1][1].abs().powi(2);
    let b = m[0][0].conj() * m[0][1] + m[1][0].conj() * m[1][1];
    scale * ((a + d + (a - d).hypot(2.0 * b.abs())) / 2.0).sqrt()
}
