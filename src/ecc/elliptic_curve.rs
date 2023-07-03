use num_bigint::BigInt;
use std::ops::Add;

//y^2 = x^3 + A*x + B
#[derive(Clone, Debug, PartialEq, Eq)]
struct Curve {
    a: BigInt,
    b: BigInt,
}

impl Default for Curve {
    fn default() -> Self {
        Self {
            a: 5i64.into(),
            b: 7i64.into(),
        }
    }
}

impl Curve {
    pub fn new<A: Into<BigInt>, B: Into<BigInt>>(a: A, b: B) -> Self {
        Self {
            a: a.into(),
            b: b.into(),
        }
    }
}

#[derive(Debug, Eq, Clone, Default)]
struct Point {
    coordinate: Option<Coordinate>,
    curve: Curve,
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
                let s: BigInt = (3 * x1 * x1 + &self.curve.a) / 2 / y1;
                let x: BigInt = &s * &s - 2 * x1;
                let y: BigInt = s * (x1 - &x) - y1;
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
                let s: BigInt = (3 * x1 * x1 + &self.curve.a) / 2 / y1;
                let x: BigInt = &s * &s - 2 * x1;
                let y: BigInt = s * (x1 - &x) - y1;
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

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Coordinate {
    x: BigInt,
    y: BigInt,
}

impl<X: Into<BigInt>, Y: Into<BigInt>> From<(X, Y)> for Coordinate {
    fn from((x, y): (X, Y)) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
        }
    }
}

impl Coordinate {
    pub fn new(x: BigInt, y: BigInt) -> Coordinate {
        Self { x, y }
    }
}

impl Point {
    pub fn new(coordinate: Option<Coordinate>, curve: Curve) -> Result<Point, ()> {
        match &coordinate {
            Some(Coordinate { x, y }) if y.pow(2) != x.pow(3) + &curve.a * x + &curve.b => Err(()),
            _ => Ok(Self { coordinate, curve }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        assert!(Point::new(Some((-1i64, -1i64).into()), Curve::default()).is_ok());
        assert!(Point::new(Some((-1i64, -2i64).into()), Curve::default()).is_err());
    }
    #[test]
    fn test_ne() {
        let a = Point::new(Some((3, -7).into()), Curve::default()).unwrap();
        let b = Point::new(Some((18, 77).into()), Curve::default()).unwrap();
        assert_eq!(a, a);
        assert_ne!(a, b);
    }
    #[test]
    fn test_add0() {
        let a = Point::default();
        let b = Point::new(Some((2, 5).into()), Curve::default()).unwrap();
        let c = Point::new(Some((2, -5).into()), Curve::default()).unwrap();
        assert_eq!(&a + &b, &b);
        assert_eq!(&b + &a, &b);
        assert_eq!(&b + &c, &a);
    }
    #[test]
    fn test_add1() {
        let a = Point::new(Some((3, 7).into()), Curve::default()).unwrap();
        let b = Point::new(Some((-1, -1).into()), Curve::default()).unwrap();
        let c = Point::new(Some((2, -5).into()), Curve::default()).unwrap();
        assert_eq!(a + b, c);
    }
    #[test]
    fn test_add2() {
        let a = Point::new(Some((-1, -1).into()), Curve::default()).unwrap();
        assert_eq!(
            a.clone() + a,
            Point::new(Some((18, 77).into()), Curve::default()).unwrap()
        )
    }
    #[test]
    fn test_add3() {
        let curve = Curve::new(1, -10);
        let a = Point::new(Some((2, 0).into()), curve.clone()).unwrap();
        let b = Point::new(Some((2, 0).into()), curve.clone()).unwrap();
        let c = Point::new(None, curve).unwrap();
        assert_eq!(a + b, c);
    }
}
