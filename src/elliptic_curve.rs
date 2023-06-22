use std::marker::PhantomData;

//y^2 = x^3 + A*x + B
struct Curve<const A: i64, const B: i64> {}

#[derive(Debug)]
struct Point<const A: i64 = 5, const B: i64 = 7> {
    x: i64,
    y: i64,
    curve: PhantomData<Curve<A, B>>,
}

impl<const A: i64, const B: i64> Point<A, B> {
    pub fn new(x: i64, y: i64) -> Result<Point<A, B>, ()> {
        if y.pow(2) != x.pow(3) + A * x + B {
            Err(())
        } else {
            Ok(Self {
                x,
                y,
                curve: Default::default(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        assert!(Point::<5, 7>::new(-1, -1).is_ok());
        assert!(Point::<5, 7>::new(-1, -2).is_err());
    }
}
