use crate::ecc::finite_field::FieldElement;

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

#[derive(Debug, Clone)]
struct Point {
    coordinate: Option<Coordinate>,
    curve: CurveOverFiniteField,
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
/*
impl Add for &Point {
    type Output = Point;

    fn add(self, rhs: Self) -> Self::Output {
        let curve = self.curve.clone();
        match (&self.coordinate, &rhs.coordinate) {
            (None, None) => Point::new(None, curve).unwrap(),
            (None, Some(_)) => rhs.clone(),
            (Some(_), None) => self.clone(),
            (Some(Coordinate { x: x1, y: y1 }), Some(Coordinate { x: x2, y: y2 }))
                if x1 == x2 && y1 != y2 =>
            {
                Point::new(None, curve).unwrap()
            }
            (p1 @ Some(Coordinate { y, .. }), p2) if p1 == p2 && y == &BigInt::from(0i64) => {
                Point::new(None, curve).unwrap()
            }
            (p1 @ Some(Coordinate { x: x1, y: y1 }), p2) if p1 == p2 => {
                let s: FieldElement = (3 * x1 * x1 + &self.curve.a) / 2 / y1;
                let x: FieldElement = &s * &s - 2 * x1;
                let y: FieldElement = s * (x1 - &x) - y1;
                Point::new(Some(Coordinate::new(x, y)), curve).unwrap()
            }
            (Some(Coordinate { x: x1, y: y1 }), Some(Coordinate { x: x2, y: y2 })) => {
                let s = (y2 - y1) / (x2 - x1);
                let x = &s * &s - x1 - x2;
                let y = &s * (x1 - &x) - y1;
                Point::new(Some(Coordinate::new(x, y)), curve).unwrap()
            }
        }
    }
}

impl Add for Point {
    type Output = Point;

    fn add(self, rhs: Self) -> Self::Output {
        match (&self.coordinate, &rhs.coordinate) {
            (None, None) => Point::new(None, self.curve).unwrap(),
            (None, Some(_)) => rhs,
            (Some(_), None) => self,
            (Some(Coordinate { x: x1, y: y1 }), Some(Coordinate { x: x2, y: y2 }))
                if x1 == x2 && y1 != y2 =>
            {
                Point::new(None, self.curve).unwrap()
            }
            (p1 @ Some(Coordinate { y, .. }), p2) if p1 == p2 && y == &BigInt::from(0i64) => {
                Point::new(None, self.curve).unwrap()
            }
            (p1 @ Some(Coordinate { x: x1, y: y1 }), p2) if p1 == p2 => {
                let s: FieldElement = (3 * x1 * x1 + &self.curve.a) / 2 / y1;
                let x: FieldElement = &s * &s - 2 * x1;
                let y: FieldElement = s * (x1 - &x) - y1;
                Point::new(Some(Coordinate::new(x, y)), rhs.curve).unwrap()
            }
            (Some(Coordinate { x: x1, y: y1 }), Some(Coordinate { x: x2, y: y2 })) => {
                let s = (y2 - y1) / (x2 - x1);
                let x = &s * &s - x1 - x2;
                let y = &s * (x1 - &x) - y1;
                Point::new(Some(Coordinate::new(x, y)), self.curve).unwrap()
            }
        }
    }
}
*/
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

impl Point {
    pub fn new(coordinate: Option<Coordinate>, curve: CurveOverFiniteField) -> Result<Point, ()> {
        match &coordinate {
            Some(Coordinate { x, y })
                if (y * y).unwrap()
                    != (&(x.pow(3) + (&curve.a * x).unwrap()).unwrap() + &curve.b).unwrap() =>
            {
                Err(())
            }
            _ => Ok(Self { coordinate, curve }),
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
}
