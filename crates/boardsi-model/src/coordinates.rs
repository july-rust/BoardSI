//! Exact source grids and checked SI conversion. No CAD coordinate is rounded in storage.
use crate::{ModelError, ObjectId};

/// Reduced signed rational. Arithmetic is checked; intermediate overflow is an error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rational {
    numerator: i128,
    denominator: i128,
}
impl Rational {
    pub fn new(numerator: i128, denominator: i128) -> Result<Self, ModelError> {
        if denominator <= 0 || numerator == i128::MIN {
            return Err(ModelError::InvalidRational);
        }
        let mut a = numerator.abs();
        let mut b = denominator;
        while b != 0 {
            let r = a % b;
            a = b;
            b = r;
        }
        Ok(Self {
            numerator: numerator / a,
            denominator: denominator / a,
        })
    }
    pub fn integer(value: i64) -> Self {
        Self {
            numerator: i128::from(value),
            denominator: 1,
        }
    }
    pub fn numerator(self) -> i128 {
        self.numerator
    }
    pub fn denominator(self) -> i128 {
        self.denominator
    }
    pub fn positive(self) -> bool {
        self.numerator > 0
    }
    pub fn checked_add(self, rhs: Self) -> Result<Self, ModelError> {
        let n = self
            .numerator
            .checked_mul(rhs.denominator)
            .and_then(|a| {
                rhs.numerator
                    .checked_mul(self.denominator)
                    .and_then(|b| a.checked_add(b))
            })
            .ok_or(ModelError::CoordinateOverflow)?;
        let d = self
            .denominator
            .checked_mul(rhs.denominator)
            .ok_or(ModelError::CoordinateOverflow)?;
        Self::new(n, d)
    }
    pub fn checked_mul(self, rhs: Self) -> Result<Self, ModelError> {
        Self::new(
            self.numerator
                .checked_mul(rhs.numerator)
                .ok_or(ModelError::CoordinateOverflow)?,
            self.denominator
                .checked_mul(rhs.denominator)
                .ok_or(ModelError::CoordinateOverflow)?,
        )
    }
    pub fn checked_neg(self) -> Result<Self, ModelError> {
        Self::new(-self.numerator, self.denominator)
    }
    pub fn checked_cmp(self, rhs: Self) -> Result<std::cmp::Ordering, ModelError> {
        Ok(self
            .numerator
            .checked_mul(rhs.denominator)
            .ok_or(ModelError::CoordinateOverflow)?
            .cmp(
                &rhs.numerator
                    .checked_mul(self.denominator)
                    .ok_or(ModelError::CoordinateOverflow)?,
            ))
    }
    /// Conservative rounding bound for integer casts and one binary64 division.
    /// This does not assert that a decimal SI coordinate is exactly binary representable.
    pub fn to_f64(self) -> Result<f64, ModelError> {
        let result = self.numerator as f64 / self.denominator as f64;
        if !result.is_finite() || (self.numerator != 0 && result == 0.0) {
            Err(ModelError::PrecisionLoss)
        } else {
            Ok(result)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuarterTurn {
    Zero,
    One,
    Two,
    Three,
}
/// Source-to-board mapping. Arbitrary rotations are preserved but conversion is unsupported.
#[derive(Clone, Debug, PartialEq)]
pub enum Orientation {
    Orthogonal { turn: QuarterTurn, mirror_x: bool },
    PendingRotation { radians: f64, mirror_x: bool },
}
#[derive(Clone, Debug, PartialEq)]
pub struct CoordinateFrame {
    pub meta: crate::ObjectMeta,
    pub unit_label: String,
    pub metres_per_tick: Rational,
    pub origin_m: [Rational; 2],
    pub orientation: Orientation,
    /// Provenance only; layers still require explicit mapping to canonical Layer IDs.
    pub source_layer_order_reversed: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Point {
    pub frame: ObjectId,
    pub ticks: [i64; 2],
}
impl CoordinateFrame {
    pub fn exact_m(&self, point: &Point) -> Result<[Rational; 2], ModelError> {
        if point.frame != self.meta.id {
            return Err(ModelError::InvalidReference);
        }
        if !self.metres_per_tick.positive() {
            return Err(ModelError::InvalidDimension);
        }
        let (turn, mirror) = match self.orientation {
            Orientation::Orthogonal { turn, mirror_x } => (turn, mirror_x),
            Orientation::PendingRotation { .. } => return Err(ModelError::UnsupportedTransform),
        };
        let mut x = Rational::integer(point.ticks[0]).checked_mul(self.metres_per_tick)?;
        let y = Rational::integer(point.ticks[1]).checked_mul(self.metres_per_tick)?;
        if mirror {
            x = x.checked_neg()?;
        }
        let xy = match turn {
            QuarterTurn::Zero => [x, y],
            QuarterTurn::One => [y.checked_neg()?, x],
            QuarterTurn::Two => [x.checked_neg()?, y.checked_neg()?],
            QuarterTurn::Three => [y, x.checked_neg()?],
        };
        Ok([
            xy[0].checked_add(self.origin_m[0])?,
            xy[1].checked_add(self.origin_m[1])?,
        ])
    }
    /// Round only at the solver boundary. Reject when a conservative error bound
    /// can consume half an input tick, including large-origin cancellation.
    pub fn solver_m(&self, point: &Point) -> Result<[f64; 2], ModelError> {
        let exact = self.exact_m(point)?;
        let tick = self.metres_per_tick.to_f64()?;
        let mut result = [0.0; 2];
        for i in 0..2 {
            result[i] = exact[i].to_f64()?;
            let error_bound = result[i].abs() * (4.0 * f64::EPSILON);
            if error_bound >= tick * 0.5 || tick * 0.5 == 0.0 {
                return Err(ModelError::PrecisionLoss);
            }
        }
        Ok(result)
    }
    /// Exact rational regridding in board axes; never rounds a fractional target tick.
    pub fn regrid(&self, point: &Point, target_tick_m: Rational) -> Result<[i64; 2], ModelError> {
        if !target_tick_m.positive() {
            return Err(ModelError::InvalidDimension);
        }
        let exact = self.exact_m(point)?;
        let reciprocal = Rational::new(target_tick_m.denominator, target_tick_m.numerator)?;
        let mut ticks = [0; 2];
        for i in 0..2 {
            let value = exact[i].checked_mul(reciprocal)?;
            if value.denominator != 1 {
                return Err(ModelError::PrecisionLoss);
            }
            ticks[i] =
                i64::try_from(value.numerator).map_err(|_| ModelError::CoordinateOverflow)?;
        }
        Ok(ticks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rational_normalization_and_overflow() {
        assert_eq!(Rational::new(2, 4).unwrap(), Rational::new(1, 2).unwrap());
        assert!(Rational::new(1, 0).is_err());
        assert!(Rational::new(i128::MIN, 1).is_err());
        assert!(Rational::new(i128::MAX, 1)
            .unwrap()
            .checked_add(Rational::integer(1))
            .is_err());
        assert_eq!(Rational::new(0, 10).unwrap(), Rational::integer(0));
    }
}
