use crate::ecc::finite_field::FieldElement;
use num_bigint::BigInt;
use std::ops::Add;

//y^2 = x^3 + A*x + B
#[derive(Clone, Debug, PartialEq, Eq)]
struct CurveOverFiniteField {
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
struct Coordinate {
    x: FieldElement,
    y: FieldElement,
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
struct Point {
    coordinate: Option<Coordinate>,
    curve: CurveOverFiniteField,
}

impl Point {
    pub fn new(coordinate: Option<Coordinate>, curve: CurveOverFiniteField) -> Result<Point, ()> {
        match &coordinate {
            Some(Coordinate { x, y })
                if (y * y).unwrap()
                    != (x * &curve.a)
                        .and_then(|v| v + x.pow(3))
                        .and_then(|v| v + &curve.b)
                        .unwrap() =>
            {
                Err(())
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
    type Output = Result<Point, ()>;

    fn add(self, rhs: Self) -> Self::Output {
        if self.curve != rhs.curve {
            return Err(());
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
                Err(())
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
                let two = FieldElement::new(2, x1.prime.clone()).unwrap();
                let s = FieldElement::new(3, x1.prime.clone())
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
    type Output = Result<Point, ()>;

    fn add(self, rhs: &Point) -> Self::Output {
        if self.curve != rhs.curve {
            return Err(());
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
                Err(())
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
                let two = FieldElement::new(2, x1.prime.clone()).unwrap();
                let s = FieldElement::new(3, x1.prime.clone())
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
    type Output = Result<Point, ()>;

    fn add(self, rhs: &Point) -> Self::Output {
        if self.curve != rhs.curve {
            return Err(());
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
                Err(())
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
                let two = FieldElement::new(2, x1.prime.clone()).unwrap();
                let s = FieldElement::new(3, x1.prime.clone())
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
        let a = FieldElement::new(0, prime.clone()).unwrap();
        let b = FieldElement::new(7, prime.clone()).unwrap();
        let curve = CurveOverFiniteField::new(a, b);

        let valid_points: &[(i64, i64)] = &[(192i64, 105i64), (17, 56), (1, 193)][..];
        let invalid_points = &[(200i64, 119i64), (42, 99)][..];

        let check = |points: &[(i64, i64)], expected| {
            points
                .into_iter()
                .map(|(x_raw, y_raw)| {
                    FieldElement::new(*x_raw, prime.clone())
                        .and_then(|x| FieldElement::new(*y_raw, prime.clone()).map(|y| (x, y)))
                })
                .map(|res| res.map(|(x, y)| Point::new(Some(Coordinate::new(x, y)), curve.clone())))
                .all(|res| matches!(res, Ok(Ok(_))) == expected)
        };
        let valid = check(valid_points, true);
        assert!(valid);
        let valid = check(invalid_points, false);
        assert!(valid);
    }

    #[test]
    fn test_add() {
        let prime = BigInt::from(223);
        let a = FieldElement::new(0, prime.clone()).unwrap();
        let b = FieldElement::new(7, prime.clone()).unwrap();
        let curve = CurveOverFiniteField::new(a, b);

        let p = Point::new(
            Some(Coordinate::new(
                FieldElement::new(192, prime.clone()).unwrap(),
                FieldElement::new(105, prime.clone()).unwrap(),
            )),
            curve.clone(),
        )
        .unwrap();
        assert_eq!(
            (&p + &p).unwrap(),
            Point::new(
                Some(Coordinate::new(
                    FieldElement::new(49, prime.clone()).unwrap(),
                    FieldElement::new(71, prime.clone()).unwrap(),
                )),
                curve.clone(),
            )
            .unwrap()
        );
        let additions = vec![
            [(192, 105), (17, 56), (170, 142)],
            [(47, 71), (117, 141), (60, 139)],
            [(143, 98), (76, 66), (47, 71)],
        ];
    }
}
