use lazy_static::lazy_static;
use crate::utils::bn_to_field;
use halo2_proofs::pairing::bn256::Fr;
use std::ops::{SubAssign, MulAssign, AddAssign};
use ff::Field;
use num_bigint::BigUint;

use ff_ce::BitIterator;
use franklin_crypto::alt_babyjubjub::fs;

lazy_static! {
    static ref D_BIG: BigUint = BigUint::parse_bytes(b"12181644023421730124874158521699555681764249180949974110617291017600649128846", 10).unwrap();
    static ref D: Fr = bn_to_field(&(D_BIG));
    static ref A_BIG: BigUint = BigUint::parse_bytes(b"21888242871839275222246405745257275088548364400416034343698204186575808495616", 10).unwrap();
    static ref A: Fr = bn_to_field(&(A_BIG));
    pub static ref Q: BigUint = BigUint::parse_bytes(
        b"21888242871839275222246405745257275088548364400416034343698204186575808495617",10
    ).unwrap();
}

#[derive(Clone, Debug)]
pub struct PointProjective {
    pub x: Fr,
    pub y: Fr,
    pub z: Fr,
}

impl PointProjective {
    pub fn affine(&self) -> Point {
        if self.z.is_zero_vartime() {
            return Point {
                x: Fr::zero(),
                y: Fr::zero(),
            };
        }

        let zinv = self.z.invert().unwrap();
        let mut x = self.x;
        x.mul_assign(&zinv);
        let mut y = self.y;
        y.mul_assign(&zinv);

        Point { x, y }
    }

    #[allow(clippy::many_single_char_names)]
    pub fn add(&self, q: &PointProjective) -> PointProjective {
        // add-2008-bbjlp https://hyperelliptic.org/EFD/g1p/auto-twisted-projective.html#addition-add-2008-bbjlp
        let mut a = self.z;
        a.mul_assign(&q.z);
        let mut b = a;
        b = b.square();
        let mut c = self.x;
        c.mul_assign(&q.x);
        let mut d = self.y;
        d.mul_assign(&q.y);
        let mut e = *D;
        e.mul_assign(&c);
        e.mul_assign(&d);
        let mut f = b;
        f.sub_assign(&e);
        let mut g = b;
        g.add_assign(&e);
        let mut x1y1 = self.x;
        x1y1.add_assign(&self.y);
        let mut x2y2 = q.x;
        x2y2.add_assign(&q.y);
        let mut aux = x1y1;
        aux.mul_assign(&x2y2);
        aux.sub_assign(&c);
        aux.sub_assign(&d);
        let mut x3 = a;
        x3.mul_assign(&f);
        x3.mul_assign(&aux);
        let mut ac = *A;
        ac.mul_assign(&c);
        let mut dac = d;
        dac.sub_assign(&ac);
        let mut y3 = a;
        y3.mul_assign(&g);
        y3.mul_assign(&dac);
        let mut z3 = f;
        z3.mul_assign(&g);

        PointProjective {
            x: x3,
            y: y3,
            z: z3,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Point {
    pub x: Fr,
    pub y: Fr,
}

impl Point {
    pub fn identity() -> Self {
        Point {
            x: Fr::zero(),
            y: Fr::one(),
        }
    }
    pub fn zero() -> Self {
        Point {
            x: Fr::zero(),
            y: Fr::one(),
        }
    }
    pub fn projective(&self) -> PointProjective {
        PointProjective {
            x: self.x,
            y: self.y,
            z: Fr::one(),
        }
    }

    pub fn add(&self, other: &Point) -> Point{
        self.projective().add(&other.projective()).affine()
    }

    pub fn mul_scalar(&self, n: &BigUint) -> Point {
        let mut r = Point::zero();
        let mut exp = self.clone();
        let b = n.to_bytes_le();
        //little-end wise, like 6, it is 0,1,1 sequence
        for i in 0..n.bits() {
            if test_bit(&b, i.try_into().unwrap()) {
                r = r.add(&exp);
            }
            exp = exp.add(&exp);
        }
        r
    }


    pub fn mul_scalar_fs(&self, scalar: fs::Fs) -> Point {
        let mut r = Point::zero();
        //big-end wise, like 6, it is 1,1,0 sequence
        for b in BitIterator::<fs::FsRepr>::new(scalar.into()) {
            r = r.add(&r);
            if b {
                r = r.add(self);
            }
        }
        r
    }
}

pub fn test_bit(b: &[u8], i: usize) -> bool {
    b[i / 8] & (1 << (i % 8)) != 0
}

#[cfg(test)]

mod tests {
    use super::Point;
    use num_bigint::BigUint;
    use ff_ce::PrimeField;
    use franklin_crypto::alt_babyjubjub::fs;
    use franklin_crypto::jubjub::ToUniform;
    use crate::utils::bn_to_field;
    use std::str::FromStr;
    #[test]
    pub fn verify_alt_jubjub_signature() {
        let msg = b"Foo bar";

        // pad with zeroes to match representation length
        let mut msg_padded: Vec<u8> = msg.iter().cloned().collect();
        msg_padded.resize(32, 0u8);

        let c = fs::Fs::to_uniform_32(msg_padded.as_ref());
        // let mut c_bytes = [0u8; 32];
        // c.into_repr().write_le(& mut c_bytes[..]).expect("get LE bytes of signature S");
        // let c_repr_bigint = BigInt::from_signed_bytes_le(&c_bytes);
        // println!("c {}",c_repr_bigint.to_str_radix(10));

        let vk = Point {
            x: bn_to_field(&(BigUint::parse_bytes(b"139f1d319d2a51a1938aef20ae4aa05b4bacef0c95ec2acf6d70b0430bed7808", 16).unwrap())),
            y: bn_to_field(&(BigUint::parse_bytes(b"023abdc9dac65b2e858cf258c0a9b0c2c8a83a86ec2ebbaab8fdb5169b262597", 16).unwrap())),
        };

        let sig_r = Point {
            x: bn_to_field(&(BigUint::parse_bytes(b"00d711880dcccc0767dad1aa321fa2f54462c0d91e7c708836b5ac274215e4ca", 16).unwrap())),
            y: bn_to_field(&(BigUint::parse_bytes(b"303438ab520086fb5e723bdc3c5e0f6a99b7d1caca0b8871ce16ab467d4baf5c", 16).unwrap())),
        };
        let p_g = Point {
            x: bn_to_field(&(BigUint::parse_bytes(b"2ef3f9b423a2c8c74e9803958f6c320e854a1c1c06cd5cc8fd221dc052d76df7", 16).unwrap())),
            y: bn_to_field(&(BigUint::parse_bytes(b"05a01167ea785d3f784224644a68e4067532c815f5f6d57d984b5c0e9c6c94b7", 16).unwrap())),
        };
        let sig_s_str = "1902101563350775171813864964289368622061698554691074493911860015574812994359";
        let sig_s = fs::Fs::from_str(sig_s_str).unwrap();

        // 0 = c . vk + R -S . P_G that requires all points to be in the same group
        // self.0
        //     .mul(c, params)
        //     .add(&sig.r, params)
        //     .add(
        //         &params.generator(p_g).mul(sig.s, params).negate().into(),
        //         params,
        //     )
        //     .eq(&Point::zero())
        let lhs = vk.mul_scalar_fs(c).add(&sig_r);
        let rhs = p_g.mul_scalar_fs(sig_s);
        // println!("lhs x={},y={}",lhs.x,lhs.y);
        // println!("rhs x={},y={}",rhs.x,rhs.y);
        assert_eq!(lhs,rhs);

        let c = BigUint::from_bytes_le(&msg_padded);
        let sig_s = BigUint::from_str(sig_s_str).unwrap();
        let lhs = vk.mul_scalar(&c).add(&sig_r);
        let rhs = p_g.mul_scalar(&sig_s);
        // println!("lhs x={},y={}",lhs.x,lhs.y);
        // println!("rhs x={},y={}",rhs.x,rhs.y);
        assert_eq!(lhs,rhs)
    }

}
