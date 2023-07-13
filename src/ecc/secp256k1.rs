use super::elliptic_curve_finite_field::Point as ECPoint;
use crate::{
    ecc::elliptic_curve_finite_field::Coordinate as ECCoordinate,
    ecc::elliptic_curve_finite_field::CurveOverFiniteField, ecc::finite_field::FieldElement,
};
use anyhow::{anyhow, Result};
use hex_literal::hex;
use num_bigint::{BigInt, RandBigInt, Sign};
use num_integer::Integer;
use std::ops::{Add, Div, Mul};

use crate::ecc::secp256k1::sec_format::SecFormat;
use lazy_static::lazy_static;
use rand::thread_rng;

pub mod sec_format;

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
    pub fn sqrt(&self) -> Self {
        Field(self.0.pow((&*P + BigInt::from(1)) / 4))
    }
    pub fn pow<T: Into<BigInt>>(&self, rhs: T) -> Self {
        Field(self.0.pow(rhs))
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

impl TryFrom<&[u8]> for Point {
    type Error = anyhow::Error;
    /// returns a Point object from a SEC binary
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let Some(lead_byte) = value.first() else { return Err(anyhow!("empty input"))};
        match lead_byte {
            b'\x04' if value.len() < 65 => {
                Err(anyhow!("unacceptable length of uncompressed sec signature"))
            }
            b'\x04' => Point::new(Some(Coordinate::from((
                BigInt::from_bytes_be(Sign::Plus, &value[1..33]),
                BigInt::from_bytes_be(Sign::Plus, &value[33..65]),
            )))),
            b'\x02' | b'\x03' if value.len() < 33 => {
                Err(anyhow!("unacceptable length of compressed sec signature"))
            }

            b'\x02' | b'\x03' => {
                let y_is_even = *lead_byte == b'\x02';
                let x = Field::from(&value[1..33]);
                let alpha: Field = Field((x.pow(3).0 + Field::new(B.clone()).0).unwrap());
                let beta = alpha.sqrt();
                let chosen_beta = {
                    // choose even_beta
                    if y_is_even && beta.0.num.is_even() {
                        beta
                    } else {
                        // choose odd_beta
                        Field::new(&*P - &beta.0.num)
                    }
                };
                Point::new(Some(Coordinate::new(x, chosen_beta)))
            }
            _ => Err(anyhow!("unacceptable lead byte")),
        }
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

    /// returns the binary version of the SEC format
    pub fn sec<F: SecFormat>(&self) -> F::Output {
        F::sec(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecc::secp256k1::sec_format::{Compressed, Uncompressed};

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
        (0..5).into_iter().for_each(|_| {
            let pk = PrivateKey::new(thread_rng().gen_bigint(129));
            let z = thread_rng().gen_bigint_range(&BigInt::from(0), &BigInt::from(2).pow(256));
            let sig = pk.sign(&z).unwrap();
            assert!(pk.point.verify(&z, &sig))
        });
    }

    #[test]
    fn test_sec_uncompressed() {
        let secrets = [
            BigInt::from(5000),
            BigInt::from(2018).pow(5),
            BigInt::from_bytes_be(Sign::Plus, hex!("0deadbeef12345").as_slice()),
        ];
        let expected_secs = [
            "04ffe558e388852f0120e46af2d1b370f85854a8eb0841811ece0e3e03d282d57c315dc72890a4f10a1481c031b03b351b0dc79901ca18a00cf009dbdb157a1d10",
            "04027f3da1918455e03c46f659266a1bb5204e959db7364d2f473bdf8f0a13cc9dff87647fd023c13b4a4994f17691895806e1b40b57f4fd22581a4f46851f3b06",
            "04d90cd625ee87dd38656dd95cf79f65f60f7273b67d3096e68bd81e4f5342691f842efa762fd59961d0e99803c61edba8b3e3f7dc3a341836f97733aebf987121"
        ];

        for (idx, secret) in secrets.iter().enumerate() {
            let pk = PrivateKey::new(secret.clone());
            let actual = hex::encode(pk.sec::<Uncompressed>().unwrap());
            assert_eq!(actual, expected_secs[idx]);
        }
    }
    #[test]
    fn test_sec_compressed() {
        let secrets = [
            BigInt::from(5001),
            BigInt::from(2019).pow(5),
            BigInt::from_bytes_be(Sign::Plus, hex!("0deadbeef54321").as_slice()),
        ];

        let expected_secs = [
            "0357a4f368868a8a6d572991e484e664810ff14c05c0fa023275251151fe0e53d1",
            "02933ec2d2b111b92737ec12f1c5d20f3233a0ad21cd8b36d0bca7a0cfa5cb8701",
            "0296be5b1292f6c856b3c5654e886fc13511462059089cdf9c479623bfcbe77690",
        ];
        for (idx, secret) in secrets.iter().enumerate() {
            let pk = PrivateKey::new(secret.clone());
            let actual = hex::encode(pk.sec::<Compressed>().unwrap());
            assert_eq!(actual, expected_secs[idx]);
        }
    }
}
