use super::PrivateKey;

use crate::ecc::{
    elliptic_curve_finite_field::Coordinate as ECCoordinate, finite_field::FieldElement,
};

use num_integer::Integer;

pub struct Compressed;
pub struct Uncompressed;

pub trait SecFormat {
    type Output;

    fn sec(pk: &PrivateKey) -> Self::Output;
}

impl SecFormat for Compressed {
    type Output = Option<[u8; 33]>;

    fn sec(pk: &PrivateKey) -> Self::Output {
        pk.point.coordinate().map(
            |ECCoordinate {
                 x: FieldElement { num: x, .. },
                 y: FieldElement { num: y, .. },
             }| {
                let mut res = [0; 33];
                res[0] = if y.is_even() { b'\x02' } else { b'\x03' };
                let x_dest = &mut res[1..33];
                x_dest.copy_from_slice(&x.to_bytes_be().1[..]);

                res
            },
        )
    }
}

impl SecFormat for Uncompressed {
    type Output = Option<[u8; 65]>;

    fn sec(pk: &PrivateKey) -> Self::Output {
        pk.point.coordinate().map(
            |ECCoordinate {
                 x: FieldElement { num: x, .. },
                 y: FieldElement { num: y, .. },
             }| {
                let mut res = [0; 65];
                res[0] = b'\x04';
                let x_dest = &mut res[1..33];
                x_dest.copy_from_slice(&x.to_bytes_be().1[..]);
                let y_dest = &mut res[33..65];
                y_dest.copy_from_slice(&y.to_bytes_be().1[..]);

                res
            },
        )
    }
}
