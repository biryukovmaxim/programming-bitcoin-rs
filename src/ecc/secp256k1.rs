use super::elliptic_curve_finite_field::Point as ECPoint;
use crate::{
    ecc::elliptic_curve_finite_field::Coordinate as ECCoordinate,
    ecc::elliptic_curve_finite_field::CurveOverFiniteField, ecc::finite_field::FieldElement,
};
use anyhow::Result;
use hex_literal::hex;
use num_bigint::{BigInt, RandBigInt, Sign};
use num_integer::Integer;
use std::ops::{Add, Div, Mul};

use lazy_static::lazy_static;
use rand::thread_rng;

const _N: [u8; 32] = hex!("fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141");
const _A: u64 = 0;
const _B: u64 = 7;

lazy_static! {
    static ref N: BigInt = BigInt::from_bytes_be(Sign::Plus, _N.as_slice());
    static ref A: BigInt = BigInt::from(_A);
    static ref B: BigInt = BigInt::from(_B);
    static ref P: BigInt = BigInt::from(2).pow(256) - BigInt::from(2).pow(32) - 977;
    static ref G: Point = {
        let g_x = hex!("79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798");
        let g_y = hex!("483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8");
        Point::new(Some(Coordinate::new(
            Field::from(g_x.as_slice()),
            Field::from(g_y.as_slice()),
        )))
        .unwrap()
    };
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Field(FieldElement);

impl From<&[u8]> for Field {
    fn from(value: &[u8]) -> Self {
        Field::new(BigInt::from_bytes_be(Sign::Plus, value))
    }
}

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
impl<XT: Into<BigInt>, YT: Into<BigInt>> From<(XT, YT)> for Coordinate {
    fn from((x, y): (XT, YT)) -> Self {
        Self {
            x: Field::new(x),
            y: Field::new(y),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Point(ECPoint);

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

    pub fn verify(&self, z: &BigInt, sig: &Signature) -> bool {
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
        Point((&self.0).mul(&rhs.mod_floor(&*N)))
    }
}

#[derive(Debug, Default)]
pub struct Signature {
    r: BigInt,
    s: BigInt,
}

impl<R: Into<BigInt>, S: Into<BigInt>> From<(R, S)> for Signature {
    fn from((r, s): (R, S)) -> Self {
        Signature::new(r.into(), s.into())
    }
}

impl Signature {
    pub fn new(r: BigInt, s: BigInt) -> Self {
        Self { r, s }
    }
}

#[derive(Debug)]
pub struct PrivateKey {
    secret: BigInt,
    point: Point,
}

impl PrivateKey {
    pub fn new(secret: BigInt) -> Self {
        let point = &*G * &secret;
        Self { secret, point }
    }

    pub fn sign(&self, z: &BigInt) -> Option<Signature> {
        let k: BigInt = thread_rng().gen_bigint(129);
        let Point(ECPoint {
            coordinate: Some(ECCoordinate { x:FieldElement{num: r, ..}, .. }),
            ..
        }) = &*G * &k else {return None};
        let k_inv = k.modpow(dbg!(&(dbg!(&*N) - 2)), &N);
        let s = ((z + &r * &self.secret) * k_inv).mod_floor(&N);
        let s = if s > (&*N).div(2) { &*N - s } else { s };
        Some(Signature { r, s })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infinity() {
        let inf: Point = &*G * &*N;
        assert_eq!(inf.coordinate(), None);
    }

    #[test]
    fn test_pubpoint() {
        let points = [
            (
                BigInt::from(7),
                hex!("5cbdf0646e5db4eaa398f365f2ea7a0e3d419b7e0330e39ce92bddedcac4f9bc"),
                hex!("6aebca40ba255960a3178d6d861a54dba813d0b813fde7b5a5082628087264da"),
            ),
            (
                BigInt::from(1485),
                hex!("c982196a7466fbbbb0e27a940b6af926c1a74d5ad07128c82824a11b5398afda"),
                hex!("7a91f9eae64438afb9ce6448a1c133db2d8fb9254e4546b6f001637d50901f55"),
            ),
            (
                BigInt::from(2).pow(128),
                hex!("8f68b9d2f63b5f339239c1ad981f162ee88c5678723ea3351b7b444c9ec4c0da"),
                hex!("662a9f2dba063986de1d90c2b6be215dbbea2cfe95510bfdf23cbf79501fff82"),
            ),
            (
                BigInt::from(2).pow(240).add(BigInt::from(2).pow(31)),
                hex!("9577ff57c8234558f293df502ca4f09cbc65a6572c842b39b366f21717945116"),
                hex!("10b49c67fa9365ad7b90dab070be339a1daf9052373ec30ffae4f72d5e66d053"),
            ),
        ];
        for (secret, x, y) in points {
            let point = Point::new(Some(Coordinate::new(
                x.as_slice().into(),
                y.as_slice().into(),
            )))
            .unwrap();
            assert_eq!(&*G * &secret, point);
        }
    }
    #[test]
    fn test_sign() {
        let pk = PrivateKey::new(thread_rng().gen_bigint(129));
        let z = thread_rng().gen_bigint_range(&BigInt::from(0), &BigInt::from(2).pow(256));
        let sig = pk.sign(&z).unwrap();
        assert!(pk.point.verify(&z, &sig))
    }
}
