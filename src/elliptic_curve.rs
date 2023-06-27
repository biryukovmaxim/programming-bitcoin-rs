use std::marker::PhantomData;
use std::ops::Add;

//y^2 = x^3 + A*x + B
struct Curve<const A: i64, const B: i64> {}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Default)]
struct Point<const A: i64 = 5, const B: i64 = 7> {
    coordinate: Option<Coordinate>,
    curve: PhantomData<Curve<A, B>>,
}

impl<const A: i64, const B: i64> Add for Point<A, B> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self.coordinate, rhs.coordinate) {
            (None, None) => Point::default(),
            (None, Some(_)) => rhs,
            (Some(_), None) => self,
            (Some(Coordinate { x: x1, y: y1 }), Some(Coordinate { x: x2, y: y2 }))
                if x1 == x2 && y1 != y2 =>
            {
                Point::default()
            }
            (p1 @ Some(Coordinate { y, .. }), p2) if p1 == p2 && y == 0 => Point::default(),
            (p1 @ Some(Coordinate { x: x1, y: y1 }), p2) if p1 == p2 => {
                let s = (3 * x1 * x1 + A) / 2 / y1;
                let x = s * s - 2 * x1;
                let y = s * (x1 - x) - y1;
                Point::new(Some(Coordinate::new(x, y))).unwrap()
            }
            (Some(Coordinate { x: x1, y: y1 }), Some(Coordinate { x: x2, y: y2 })) => {
                let s = (y2 - y1) / (x2 - x1);
                let x = s * s - x1 - x2;
                let y = s * (x1 - x) - y1;
                Point::new(Some(Coordinate::new(x, y))).unwrap()
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Default)]
struct Coordinate {
    x: i64,
    y: i64,
}

impl From<(i64, i64)> for Coordinate {
    fn from((x, y): (i64, i64)) -> Self {
        Self { x, y }
    }
}

impl Coordinate {
    pub fn new(x: i64, y: i64) -> Coordinate {
        Self { x, y }
    }
}

impl<const A: i64, const B: i64> Point<A, B> {
    pub fn new(coordinate: Option<Coordinate>) -> Result<Point<A, B>, ()> {
        match coordinate {
            Some(Coordinate { x, y }) if y.pow(2) != x.pow(3) + A * x + B => Err(()),
            _ => Ok(Self {
                coordinate,
                curve: Default::default(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        assert!(Point::<5, 7>::new(Some((-1i64, -1i64).into())).is_ok());
        assert!(Point::<5, 7>::new(Some((-1i64, -2i64).into())).is_err());
    }
    #[test]
    fn test_ne() {
        let a = Point::<5, 7>::new(Some((3, -7).into())).unwrap();
        let b = Point::new(Some((18, 77).into())).unwrap();
        assert_eq!(a, a);
        assert_ne!(a, b);
    }
    #[test]
    fn test_add0() {
        let a = Point::<5, 7>::default();
        let b = Point::new(Some((2, 5).into())).unwrap();
        let c = Point::new(Some((2, -5).into())).unwrap();
        assert_eq!(a + b, b);
        assert_eq!(b + a, b);
        assert_eq!(b + c, a);
    }
    #[test]
    fn test_add1() {
        let a = Point::<5, 7>::new(Some((3, 7).into())).unwrap();
        let b = Point::new(Some((-1, -1).into())).unwrap();
        let c = Point::new(Some((2, -5).into())).unwrap();
        assert_eq!(a + b, c);
    }
    #[test]
    fn test_add2() {
        let a = Point::<5, 7>::new(Some((-1, -1).into())).unwrap();
        assert_eq!(a + a, Point::new(Some((18, 77).into())).unwrap())
    }
    #[test]
    fn test_add3() {
        let a = Point::<1, -10>::new(Some((2, 0).into())).unwrap();
        let b = Point::new(Some((2, 0).into())).unwrap();
        let c = Point::new(None).unwrap();
        assert_eq!(a + b, c);
    }
}
