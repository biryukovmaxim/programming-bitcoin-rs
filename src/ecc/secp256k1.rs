use super::elliptic_curve_finite_field::Point as ECPoint;
use crate::{
    ecc::elliptic_curve_finite_field::Coordinate as ECCoordinate,
    ecc::elliptic_curve_finite_field::CurveOverFiniteField, ecc::finite_field::FieldElement,
};
use anyhow::Result;
use hex_literal::hex;
use num_bigint::BigInt;
use num_integer::Integer;
use std::ops::{Add, Mul};

use lazy_static::lazy_static;

const _N: [u8; 32] = hex!("fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141");
const _A: u64 = 0;
const _B: u64 = 7;

lazy_static! {
    static ref N: BigInt = BigInt::from_signed_bytes_be(_N.as_slice());
    static ref A: BigInt = BigInt::from(_A);
    static ref B: BigInt = BigInt::from(_B);
    static ref P: BigInt = BigInt::from(2).pow(256) - BigInt::from(2).pow(32) - 977;
    static ref G: Point = {
        let g_x = hex!("79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798");
        let g_y = hex!("483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8");
        Point::new(Some(Coordinate::new(
            Field::new(BigInt::from_signed_bytes_be(g_x.as_slice())),
            Field::new(BigInt::from_signed_bytes_be(g_y.as_slice())),
        )))
        .unwrap()
    };
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Field(FieldElement);

impl From<Field> for FieldElement {
    fn from(value: Field) -> Self {
        value.0
    }
}

impl Field {
    pub fn new(num: impl Into<BigInt>) -> Self {
        Field(FieldElement::new(num, P.clone()))
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Coordinate {
    x: Field,
    y: Field,
}

impl Coordinate {
    pub fn new(x: Field, y: Field) -> Self {
        Self { x, y }
    }
}

impl From<Coordinate> for ECCoordinate {
    fn from(Coordinate { x, y }: Coordinate) -> Self {
        ECCoordinate::new(x.0, y.0)
    }
}

#[derive(Debug, Clone)]
pub struct Point(ECPoint);

impl PartialEq<Point> for Point {
    fn eq(&self, other: &Point) -> bool {
        self.0 == other.0
    }
}

impl PartialEq<&Point> for Point {
    fn eq(&self, other: &&Point) -> bool {
        self.0 == other.0
    }
}

impl Add for Point {
    type Output = Result<Point>;

    fn add(self, rhs: Self) -> Self::Output {
        (self.0 + rhs.0).map(Point)
    }
}

impl Add<&Point> for Point {
    type Output = Result<Point>;

    fn add(self, rhs: &Point) -> Self::Output {
        (&self.0 + &rhs.0).map(Point)
    }
}

impl Add<&Point> for &Point {
    type Output = Result<Point>;

    fn add(self, rhs: &Point) -> Self::Output {
        (&self.0 + &rhs.0).map(Point)
    }
}

impl Point {
    pub fn new(coordinate: Option<Coordinate>) -> Result<Self> {
        ECPoint::new(
            coordinate.map(Into::into),
            CurveOverFiniteField::new(Field::new(A.clone()), Field::new(B.clone())),
        )
        .map(Point)
    }

    pub fn coordinate(&self) -> Option<&ECCoordinate> {
        self.0.coordinate.as_ref()
    }

    pub fn verify(&self, z: BigInt, sig: Signature) -> bool {
        let s_inv = sig.s.modpow(&(&*N - 2), &N);
        let u = (z * &s_inv).mod_floor(&N);
        let v = (&sig.r * &s_inv).mod_floor(&N);
        let total = &*G * &u + self * &v;
        total
            .map(|p| {
                p.coordinate()
                    .map(|ECCoordinate { x, .. }| x.num == sig.r)
                    .unwrap_or_default()
            })
            .unwrap_or_default()
    }
}

impl Mul<&BigInt> for Point {
    type Output = Point;

    fn mul(self, rhs: &BigInt) -> Self::Output {
        (&self).mul(rhs)
    }
}

impl Mul<BigInt> for Point {
    type Output = Point;

    fn mul(self, rhs: BigInt) -> Self::Output {
        (&self).mul(&rhs)
    }
}

impl Mul<&BigInt> for &Point {
    type Output = Point;

    fn mul(self, rhs: &BigInt) -> Self::Output {
        Point((&self.0).mul(&rhs.mod_floor(&N)))
    }
}

#[derive(Debug, Default)]
pub struct Signature {
    r: BigInt,
    s: BigInt,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infinity() {
        let inf: Point = &*G * &*N;
        assert_eq!(inf.coordinate(), None);
    }
}
