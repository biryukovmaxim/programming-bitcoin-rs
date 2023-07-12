use crate::ecc::finite_field::FieldElement;
use anyhow::{anyhow, Result};
use num_bigint::BigInt;
use std::ops::{Add, Mul};

//y^2 = x^3 + A*x + B
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CurveOverFiniteField {
    a: FieldElement,
    b: FieldElement,
}

impl CurveOverFiniteField {
    pub fn new<A: Into<FieldElement>, B: Into<FieldElement>>(a: A, b: B) -> Self {
        Self {
            a: a.into(),
            b: b.into(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Coordinate {
    pub x: FieldElement,
    pub y: FieldElement,
}

impl<X: Into<FieldElement>, Y: Into<FieldElement>> From<(X, Y)> for Coordinate {
    fn from((x, y): (X, Y)) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
        }
    }
}

impl Coordinate {
    pub fn new(x: FieldElement, y: FieldElement) -> Coordinate {
        Self { x, y }
    }
}

#[derive(Debug, Clone)]
pub struct Point {
    pub coordinate: Option<Coordinate>,
    curve: CurveOverFiniteField,
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
        if rhs == &BigInt::default() {
            return Point::new(None, self.curve.clone()).unwrap();
        }
        let mut rhs = rhs.clone();
        let mut current = self.clone();
        let zero = BigInt::from(0);
        let one = BigInt::from(1);
        let mut res = Point::new(None, self.curve.clone()).unwrap();
        while rhs > zero {
            if &rhs & &one > zero {
                res = (&res + &current).unwrap();
            }
            current = (&current + &current).unwrap();
            rhs = &rhs >> 1;
        }
        res
    }
}

impl Point {
    pub fn new(coordinate: Option<Coordinate>, curve: CurveOverFiniteField) -> Result<Point> {
        match &coordinate {
            Some(Coordinate { x, y })
                if (y * y).unwrap()
                    != (x * &curve.a)
                        .and_then(|v| v + x.pow(3))
                        .and_then(|v| v + &curve.b)
                        .unwrap() =>
            {
                Err(anyhow!("Invalid coordinate"))
            }
            _ => Ok(Self { coordinate, curve }),
        }
    }
}

impl PartialEq<Point> for Point {
    fn eq(&self, other: &Point) -> bool {
        self.curve == other.curve && self.coordinate == other.coordinate
    }
}

impl PartialEq<&Point> for Point {
    fn eq(&self, other: &&Point) -> bool {
        self.curve == other.curve && self.coordinate == other.coordinate
    }
}

impl Add for Point {
    type Output = Result<Point>;

    fn add(self, rhs: Self) -> Self::Output {
        if self.curve != rhs.curve {
            return Err(anyhow!("Cannot add points on different curves"));
        }
        let curve = self.curve.clone();
        match (&self.coordinate, &rhs.coordinate) {
            (None, None) => Ok(Point::new(None, self.curve).unwrap()),
            (None, Some(_)) => Ok(rhs),
            (Some(_), None) => Ok(self),
            (
                Some(Coordinate {
                    x:
                        FieldElement {
                            prime: self_prime_x,
                            ..
                        },
                    y:
                        FieldElement {
                            prime: self_prime_y,
                            ..
                        },
                }),
                Some(Coordinate {
                    x:
                        FieldElement {
                            prime: rhs_prime_x, ..
                        },
                    y:
                        FieldElement {
                            prime: rhs_prime_y, ..
                        },
                }),
            ) if self_prime_x != self_prime_y
                || self_prime_y != rhs_prime_x
                || rhs_prime_x != rhs_prime_y =>
            {
                Err(anyhow!("Invalid prime numbers for self/rhs"))
            }
            (Some(Coordinate { x: x1, y: y1 }), Some(Coordinate { x: x2, y: y2 }))
                if x1 == x2 && y1 != y2 =>
            {
                Ok(Point::new(None, self.curve).unwrap())
            }
            (p1 @ Some(Coordinate { y, .. }), p2) if p1 == p2 && y.num == BigInt::from(0i64) => {
                Ok(Point::new(None, self.curve).unwrap())
            }
            (p1 @ Some(Coordinate { x: x1, y: y1 }), p2) if p1 == p2 => {
                let two = FieldElement::new(2, x1.prime.clone());
                let s = Ok(FieldElement::new(3, x1.prime.clone()))
                    .and_then(|v| v * x1)
                    .and_then(|v| v * x1)
                    .and_then(|v| v + self.curve.a)
                    .and_then(|v| v / &two)
                    .and_then(|v| v / y1)
                    .unwrap();
                let x = ((&s * &s).unwrap() - (two * x1).unwrap()).unwrap();
                let y = (x1 - &x).and_then(|v| v * s).and_then(|v| v - y1).unwrap();
                Ok(Point::new(Some(Coordinate::new(x, y)), curve).unwrap())
            }
            (Some(Coordinate { x: x1, y: y1 }), Some(Coordinate { x: x2, y: y2 })) => {
                let s = ((y2 - y1).unwrap() / (x2 - x1).unwrap()).unwrap();
                let x = (&s * &s).and_then(|v| v - x1).and_then(|v| v - x2).unwrap();
                let y = (x1 - &x).and_then(|v| v * &s).and_then(|v| v - y1).unwrap();
                Ok(Point::new(Some(Coordinate::new(x, y)), curve).unwrap())
            }
        }
    }
}

impl Add<&Point> for Point {
    type Output = Result<Point>;

    fn add(self, rhs: &Point) -> Self::Output {
        if self.curve != rhs.curve {
            return Err(anyhow!("Cannot add points on different curves"));
        }
        let curve = self.curve.clone();
        match (&self.coordinate, &rhs.coordinate) {
            (None, None) => Ok(Point::new(None, self.curve).unwrap()),
            (None, Some(_)) => Ok(rhs.clone()),
            (Some(_), None) => Ok(self),
            (
                Some(Coordinate {
                    x:
                        FieldElement {
                            prime: self_prime_x,
                            ..
                        },
                    y:
                        FieldElement {
                            prime: self_prime_y,
                            ..
                        },
                }),
                Some(Coordinate {
                    x:
                        FieldElement {
                            prime: rhs_prime_x, ..
                        },
                    y:
                        FieldElement {
                            prime: rhs_prime_y, ..
                        },
                }),
            ) if self_prime_x != self_prime_y
                || self_prime_y != rhs_prime_x
                || rhs_prime_x != rhs_prime_y =>
            {
                Err(anyhow!("Invalid prime numbers for self/rhs"))
            }
            (Some(Coordinate { x: x1, y: y1 }), Some(Coordinate { x: x2, y: y2 }))
                if x1 == x2 && y1 != y2 =>
            {
                Ok(Point::new(None, self.curve).unwrap())
            }
            (p1 @ Some(Coordinate { y, .. }), p2) if p1 == p2 && y.num == BigInt::from(0i64) => {
                Ok(Point::new(None, self.curve).unwrap())
            }
            (p1 @ Some(Coordinate { x: x1, y: y1 }), p2) if p1 == p2 => {
                let two = FieldElement::new(2, x1.prime.clone());
                let s = Ok(FieldElement::new(3, x1.prime.clone()))
                    .and_then(|v| v * x1)
                    .and_then(|v| v * x1)
                    .and_then(|v| v + self.curve.a)
                    .and_then(|v| v / &two)
                    .and_then(|v| v / y1)
                    .unwrap();
                let x = ((&s * &s).unwrap() - (two * x1).unwrap()).unwrap();
                let y = (x1 - &x).and_then(|v| v * s).and_then(|v| v - y1).unwrap();
                Ok(Point::new(Some(Coordinate::new(x, y)), curve).unwrap())
            }
            (Some(Coordinate { x: x1, y: y1 }), Some(Coordinate { x: x2, y: y2 })) => {
                let s = ((y2 - y1).unwrap() / (x2 - x1).unwrap()).unwrap();
                let x = (&s * &s).and_then(|v| v - x1).and_then(|v| v - x2).unwrap();
                let y = (x1 - &x).and_then(|v| v * &s).and_then(|v| v - y1).unwrap();
                Ok(Point::new(Some(Coordinate::new(x, y)), curve).unwrap())
            }
        }
    }
}

impl Add<&Point> for &Point {
    type Output = Result<Point>;

    fn add(self, rhs: &Point) -> Self::Output {
        if self.curve != rhs.curve {
            return Err(anyhow!("Cannot add points on different curves"));
        }
        let curve = self.curve.clone();
        match (&self.coordinate, &rhs.coordinate) {
            (None, None) => Ok(Point::new(None, self.curve.clone()).unwrap()),
            (None, Some(_)) => Ok(rhs.clone()),
            (Some(_), None) => Ok(self.clone()),
            (
                Some(Coordinate {
                    x:
                        FieldElement {
                            prime: self_prime_x,
                            ..
                        },
                    y:
                        FieldElement {
                            prime: self_prime_y,
                            ..
                        },
                }),
                Some(Coordinate {
                    x:
                        FieldElement {
                            prime: rhs_prime_x, ..
                        },
                    y:
                        FieldElement {
                            prime: rhs_prime_y, ..
                        },
                }),
            ) if self_prime_x != self_prime_y
                || self_prime_y != rhs_prime_x
                || rhs_prime_x != rhs_prime_y =>
            {
                Err(anyhow!("Invalid prime numbers for self/rhs"))
            }
            (Some(Coordinate { x: x1, y: y1 }), Some(Coordinate { x: x2, y: y2 }))
                if x1 == x2 && y1 != y2 =>
            {
                Ok(Point::new(None, self.curve.clone()).unwrap())
            }
            (p1 @ Some(Coordinate { y, .. }), p2) if p1 == p2 && y.num == BigInt::from(0i64) => {
                Ok(Point::new(None, self.curve.clone()).unwrap())
            }
            (p1 @ Some(Coordinate { x: x1, y: y1 }), p2) if p1 == p2 => {
                let two = FieldElement::new(2, x1.prime.clone());
                let s = Ok(FieldElement::new(3, x1.prime.clone()))
                    .and_then(|v| v * x1)
                    .and_then(|v| v * x1)
                    .and_then(|v| v + &self.curve.a)
                    .and_then(|v| v / &two)
                    .and_then(|v| v / y1)
                    .unwrap();
                let x = ((&s * &s).unwrap() - (two * x1).unwrap()).unwrap();
                let y = (x1 - &x).and_then(|v| v * s).and_then(|v| v - y1).unwrap();
                Ok(Point::new(Some(Coordinate::new(x, y)), curve).unwrap())
            }
            (Some(Coordinate { x: x1, y: y1 }), Some(Coordinate { x: x2, y: y2 })) => {
                let s = ((y2 - y1).unwrap() / (x2 - x1).unwrap()).unwrap();
                let x = (&s * &s).and_then(|v| v - x1).and_then(|v| v - x2).unwrap();
                let y = (x1 - &x).and_then(|v| v * &s).and_then(|v| v - y1).unwrap();
                Ok(Point::new(Some(Coordinate::new(x, y)), curve).unwrap())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigInt;

    #[test]
    fn test_on_curve() {
        let prime = BigInt::from(223);
        let a = FieldElement::new(0, prime.clone());
        let b = FieldElement::new(7, prime.clone());
        let curve = CurveOverFiniteField::new(a, b);

        let valid_points: &[(i64, i64)] = &[(192i64, 105i64), (17, 56), (1, 193)][..];
        let invalid_points = &[(200i64, 119i64), (42, 99)][..];

        let check = |points: &[(i64, i64)], expected| {
            points
                .into_iter()
                .map(|(x_raw, y_raw)| {
                    (
                        FieldElement::new(*x_raw, prime.clone()),
                        FieldElement::new(*y_raw, prime.clone()),
                    )
                })
                .map(|(x, y)| Point::new(Some(Coordinate::new(x, y)), curve.clone()))
                .all(|res| res.is_ok() == expected)
        };
        let valid = check(valid_points, true);
        assert!(valid);
        let valid = check(invalid_points, false);
        assert!(valid);
    }

    #[test]
    fn test_add() {
        let prime = BigInt::from(223);
        let a = FieldElement::new(0, prime.clone());
        let b = FieldElement::new(7, prime.clone());
        let curve = CurveOverFiniteField::new(a, b);

        let p = Point::new(
            Some(Coordinate::new(
                FieldElement::new(192, prime.clone()),
                FieldElement::new(105, prime.clone()),
            )),
            curve.clone(),
        )
        .unwrap();
        assert_eq!(
            (&p + &p).unwrap(),
            Point::new(
                Some(Coordinate::new(
                    FieldElement::new(49, prime.clone()),
                    FieldElement::new(71, prime.clone()),
                )),
                curve.clone(),
            )
            .unwrap()
        );

        let additions = [
            ((192, 105), (17, 56), (170, 142)),
            ((47, 71), (117, 141), (60, 139)),
            ((143, 98), (76, 66), (47, 71)),
        ];

        for ((x1, y1), (x2, y2), (x3, y3)) in additions {
            let p1 = Point::new(
                Some(Coordinate::new(
                    FieldElement::new(x1, prime.clone()),
                    FieldElement::new(y1, prime.clone()),
                )),
                curve.clone(),
            )
            .unwrap();
            let p2 = Point::new(
                Some(Coordinate::new(
                    FieldElement::new(x2, prime.clone()),
                    FieldElement::new(y2, prime.clone()),
                )),
                curve.clone(),
            )
            .unwrap();
            let p3 = Point::new(
                Some(Coordinate::new(
                    FieldElement::new(x3, prime.clone()),
                    FieldElement::new(y3, prime.clone()),
                )),
                curve.clone(),
            )
            .unwrap();

            assert_eq!(p1.add(p2).unwrap(), p3);
        }
    }
    #[test]
    fn test_mul() {
        let prime = BigInt::from(223);
        let a = FieldElement::new(0, prime.clone());
        let b = FieldElement::new(7, prime.clone());
        let curve = CurveOverFiniteField::new(a, b);

        let multiplications = [
            (2, (192, 105), Some((49, 71))),
            (2, (143, 98), Some((64, 168))),
            (2, (47, 71), Some((36, 111))),
            (4, (47, 71), Some((194, 51))),
            (8, (47, 71), Some((116, 55))),
            (21, (47, 71), None),
        ];

        for (s, (x1, y1), c2) in multiplications {
            let p1 = Point::new(
                Some(Coordinate::new(
                    FieldElement::new(x1, prime.clone()),
                    FieldElement::new(y1, prime.clone()),
                )),
                curve.clone(),
            )
            .unwrap();
            let p2 = if let Some((x2, y2)) = c2 {
                Point::new(
                    Some(Coordinate::new(
                        FieldElement::new(x2, prime.clone()),
                        FieldElement::new(y2, prime.clone()),
                    )),
                    curve.clone(),
                )
                .unwrap()
            } else {
                Point::new(None, curve.clone()).unwrap()
            };

            assert_eq!(p1 * BigInt::from(s), p2);
        }
    }
}
