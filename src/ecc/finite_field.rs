use anyhow::anyhow;
use num_bigint::BigInt;
use num_integer::Integer;
use std::ops::{Add, Div, Mul, Sub};

type Result<T> = std::result::Result<T, anyhow::Error>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldElement {
    pub num: BigInt,
    pub prime: BigInt,
}

impl FieldElement {
    pub fn new(num: impl Into<BigInt>, prime: impl Into<BigInt>) -> Result<Self> {
        let prime = prime.into();
        let (num, prime) = (num.into() % &prime, prime);
        if prime.lt(&num) {
            Err(anyhow!(
                "prime should be gt value, got num: {num}, prime: {prime}"
            ))
        } else {
            Ok(Self { num, prime })
        }
    }

    pub fn pow(&self, rhs: impl Into<BigInt>) -> Self {
        let exponent = rhs.into();
        let exponent = if exponent.lt(&0i32.into()) {
            BigInt::from(-1i64).add(&self.prime).add(&exponent)
        } else {
            exponent
        };

        Self {
            num: self.num.modpow(&exponent, &self.prime),
            prime: self.prime.clone(),
        }
    }
}

impl Add for FieldElement {
    type Output = Result<FieldElement>;

    fn add(self, rhs: Self) -> Self::Output {
        if self.prime != rhs.prime {
            Err(anyhow!("Cannot add two numbers in different Fields"))
        } else {
            Ok(Self {
                num: self.num.add(rhs.num).mod_floor(&self.prime),
                ..self
            })
        }
    }
}

impl Add<&FieldElement> for FieldElement {
    type Output = Result<FieldElement>;

    fn add(self, rhs: &FieldElement) -> Self::Output {
        if self.prime != rhs.prime {
            Err(anyhow!("Cannot add two numbers in different Fields"))
        } else {
            Ok(FieldElement {
                num: (&self.num).add(&rhs.num).mod_floor(&self.prime),
                prime: self.prime,
            })
        }
    }
}

impl Add<&FieldElement> for &FieldElement {
    type Output = Result<FieldElement>;

    fn add(self, rhs: &FieldElement) -> Self::Output {
        if self.prime != rhs.prime {
            Err(anyhow!("Cannot add two numbers in different Fields"))
        } else {
            Ok(FieldElement {
                num: (&self.num).add(&rhs.num).mod_floor(&self.prime),
                prime: self.prime.clone(),
            })
        }
    }
}

impl Sub for FieldElement {
    type Output = Result<FieldElement>;

    fn sub(self, rhs: Self) -> Self::Output {
        if self.prime != rhs.prime {
            Err(anyhow!("Cannot add two numbers in different Fields"))
        } else if self.num >= rhs.num {
            Ok(Self {
                num: self.num.sub(rhs.num),
                ..self
            })
        } else {
            Ok(Self {
                num: rhs.prime - rhs.num.sub(self.num),
                ..self
            })
        }
    }
}

impl Sub<&FieldElement> for FieldElement {
    type Output = Result<FieldElement>;

    fn sub(self, rhs: &FieldElement) -> Self::Output {
        if self.prime != rhs.prime {
            Err(anyhow!("Cannot add two numbers in different Fields"))
        } else if self.num >= rhs.num {
            Ok(FieldElement {
                num: (&self.num).sub(&rhs.num),
                prime: self.prime,
            })
        } else {
            Ok(FieldElement {
                num: &rhs.prime - (&rhs.num).sub(&self.num),
                prime: self.prime,
            })
        }
    }
}

impl Sub<&FieldElement> for &FieldElement {
    type Output = Result<FieldElement>;

    fn sub(self, rhs: &FieldElement) -> Self::Output {
        if self.prime != rhs.prime {
            Err(anyhow!("Cannot add two numbers in different Fields"))
        } else if self.num >= rhs.num {
            Ok(FieldElement {
                num: (&self.num).sub(&rhs.num),
                prime: self.prime.clone(),
            })
        } else {
            Ok(FieldElement {
                num: &rhs.prime - (&rhs.num).sub(&self.num),
                prime: rhs.prime.clone(),
            })
        }
    }
}

impl Div for FieldElement {
    type Output = Result<FieldElement>;

    fn div(self, rhs: Self) -> Self::Output {
        if self.prime != rhs.prime {
            Err(anyhow!("Cannot add two numbers in different Fields"))
        } else {
            let num = (self.num * rhs.num.modpow(&self.prime.sub(2), &rhs.prime)) % &rhs.prime;
            Ok(Self {
                num,
                prime: rhs.prime,
            })
        }
    }
}

impl Div<&FieldElement> for FieldElement {
    type Output = Result<FieldElement>;

    fn div(self, rhs: &FieldElement) -> Self::Output {
        if self.prime != rhs.prime {
            Err(anyhow!("Cannot add two numbers in different Fields"))
        } else {
            let num = (&self.num
                * &(rhs.num).modpow(&(&self.prime).sub(&BigInt::from(2)), &rhs.prime))
                % &rhs.prime;
            Ok(FieldElement {
                num,
                prime: self.prime,
            })
        }
    }
}

impl Div<&FieldElement> for &FieldElement {
    type Output = Result<FieldElement>;

    fn div(self, rhs: &FieldElement) -> Self::Output {
        if self.prime != rhs.prime {
            Err(anyhow!("Cannot add two numbers in different Fields"))
        } else {
            let num = (&self.num
                * &(rhs.num).modpow(&(&self.prime).sub(&BigInt::from(2)), &rhs.prime))
                % &rhs.prime;
            Ok(FieldElement {
                num,
                prime: self.prime.clone(),
            })
        }
    }
}

impl Mul for FieldElement {
    type Output = Result<FieldElement>;

    fn mul(self, rhs: Self) -> Self::Output {
        if self.prime != rhs.prime {
            Err(anyhow!("Cannot add two numbers in different Fields"))
        } else {
            Ok(Self {
                num: self.num.mul(rhs.num).mod_floor(&self.prime),
                ..self
            })
        }
    }
}

impl Mul<&FieldElement> for FieldElement {
    type Output = Result<FieldElement>;

    fn mul(self, rhs: &FieldElement) -> Self::Output {
        if self.prime != rhs.prime {
            Err(anyhow!("Cannot add two numbers in different Fields"))
        } else {
            Ok(FieldElement {
                num: (&self.num).mul(&rhs.num).mod_floor(&self.prime),
                prime: self.prime,
            })
        }
    }
}

impl Mul<&FieldElement> for &FieldElement {
    type Output = Result<FieldElement>;

    fn mul(self, rhs: &FieldElement) -> Self::Output {
        if self.prime != rhs.prime {
            Err(anyhow!("Cannot add two numbers in different Fields"))
        } else {
            Ok(FieldElement {
                num: (&self.num).mul(&rhs.num).mod_floor(&self.prime),
                prime: self.prime.clone(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        assert!(FieldElement::new(BigInt::from(1u64), BigInt::from(7u64)).is_ok());
        assert!(FieldElement::new(BigInt::from(8u64), BigInt::from(7u64)).is_err());
    }

    #[test]
    fn test_ne() {
        let a = FieldElement::new(2u64, 31u64).unwrap();
        let b = FieldElement::new(2u64, 31u64).unwrap();
        let c = FieldElement::new(15u64, 31u64).unwrap();

        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_add() {
        let a = FieldElement::new(2u64, 31u64).unwrap();
        let b = FieldElement::new(15u64, 31u64).unwrap();
        assert_eq!(a.add(b).unwrap(), FieldElement::new(17u64, 31u64).unwrap());

        let a = FieldElement::new(17u64, 31u64).unwrap();
        let b = FieldElement::new(21u64, 31u64).unwrap();

        assert_eq!(a.add(b).unwrap(), FieldElement::new(7u64, 31u64).unwrap());
    }

    #[test]
    fn test_sub() {
        let a = FieldElement::new(29u64, 31u64).unwrap();
        assert_eq!((&a).sub(&a).unwrap(), FieldElement::new(0, 31u64).unwrap());

        let a = FieldElement::new(29u64, 31u64).unwrap();
        let b = FieldElement::new(4u64, 31u64).unwrap();
        assert_eq!(a.sub(b).unwrap(), FieldElement::new(25u64, 31u64).unwrap());

        let a = FieldElement::new(15u64, 31u64).unwrap();
        let b = FieldElement::new(30u64, 31u64).unwrap();
        assert_eq!(a.sub(b).unwrap(), FieldElement::new(16u64, 31u64).unwrap());

        let a = FieldElement::new(0u64, 31u64).unwrap();
        let b = FieldElement::new(30u64, 31u64).unwrap();
        assert_eq!(a.sub(b).unwrap(), FieldElement::new(1u64, 31u64).unwrap());
    }

    #[test]
    fn test_mul() {
        let a = FieldElement::new(24u64, 31u64).unwrap();
        let b = FieldElement::new(19u64, 31u64).unwrap();
        assert_eq!(a.mul(b).unwrap(), FieldElement::new(22u64, 31u64).unwrap());

        let a = FieldElement::new(3u64, 13u64).unwrap();
        let b = FieldElement::new(12u64, 13u64).unwrap();
        assert_eq!(a.mul(b).unwrap(), FieldElement::new(10u64, 13u64).unwrap());
    }

    #[test]
    fn test_pow() {
        let a = FieldElement::new(17u64, 31u64).unwrap();
        assert_eq!(a.pow(3u64), FieldElement::new(15u64, 31u64).unwrap());

        let a = FieldElement::new(5u64, 31u64).unwrap();
        let b = FieldElement::new(18u64, 31u64).unwrap();
        assert_eq!(
            a.pow(5u64).mul(b).unwrap(),
            FieldElement::new(16u64, 31u64).unwrap()
        );
    }

    /*def test_div(self):
    a = FieldElement(3, 31)
    b = FieldElement(24, 31)
    self.assertEqual(a / b, FieldElement(4, 31))
    a = FieldElement(17, 31)
    self.assertEqual(a**-3, FieldElement(29, 31))
    a = FieldElement(4, 31)
    b = FieldElement(11, 31)
    self.assertEqual(a**-4 * b, FieldElement(13, 31))*/
    #[test]
    fn test_div() {
        let a = FieldElement::new(2u64, 19u64).unwrap();
        let b = FieldElement::new(7u64, 19u64).unwrap();

        assert_eq!(a.div(b).unwrap(), FieldElement::new(3u64, 19u64).unwrap());

        let a = FieldElement::new(3u64, 31u64).unwrap();
        let b = FieldElement::new(24u64, 31u64).unwrap();

        assert_eq!(a.div(b).unwrap(), FieldElement::new(4u64, 31u64).unwrap());

        let a = FieldElement::new(5u64, 31u64).unwrap();
        let b = FieldElement::new(18u64, 31u64).unwrap();
        assert_eq!(
            a.pow(5u64).mul(b).unwrap(),
            FieldElement::new(16u64, 31u64).unwrap()
        );

        let a = FieldElement::new(17u64, 31u64).unwrap();
        assert_eq!(a.pow(-3i64), FieldElement::new(29u64, 31u64).unwrap());

        let a = FieldElement::new(4u64, 31u64).unwrap();
        let b = FieldElement::new(11u64, 31u64).unwrap();
        assert_eq!(
            a.pow(-4i64).mul(b).unwrap(),
            FieldElement::new(13u64, 31u64).unwrap(),
        )
    }
}
