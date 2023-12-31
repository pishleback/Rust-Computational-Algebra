#[allow(dead_code)]
use std::collections::HashMap;

use super::{poly::PolynomialRing, ring::*};
use malachite_base::num::arithmetic::traits::{DivMod, UnsignedAbs};
use malachite_nz::{integer::Integer, natural::Natural};
use malachite_q::Rational;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegerRing;
pub const ZZ: IntegerRing = IntegerRing;

impl ComRing for IntegerRing {
    type ElemT = Integer;

    fn to_string(&self, elem: &Self::ElemT) -> String {
        elem.to_string()
    }

    fn equal(&self, a: &Self::ElemT, b: &Self::ElemT) -> bool {
        a == b
    }

    fn zero(&self) -> Self::ElemT {
        Self::ElemT::from(0)
    }
    fn one(&self) -> Self::ElemT {
        Self::ElemT::from(1)
    }

    fn neg_mut(&self, elem: &mut Self::ElemT) {
        *elem *= Self::ElemT::from(-1)
    }
    fn neg(&self, elem: Self::ElemT) -> Self::ElemT {
        -elem
    }

    fn add_mut(&self, elem: &mut Self::ElemT, x: &Self::ElemT) {
        *elem += x;
    }
    fn add(&self, a: Self::ElemT, b: Self::ElemT) -> Self::ElemT {
        a + b
    }
    fn add_ref(&self, a: Self::ElemT, b: &Self::ElemT) -> Self::ElemT {
        a + b
    }
    fn add_refs(&self, a: &Self::ElemT, b: &Self::ElemT) -> Self::ElemT {
        a + b
    }

    fn mul_mut(&self, elem: &mut Self::ElemT, x: &Self::ElemT) {
        *elem *= x;
    }
    fn mul(&self, a: Self::ElemT, b: Self::ElemT) -> Self::ElemT {
        a * b
    }
    fn mul_ref(&self, a: Self::ElemT, b: &Self::ElemT) -> Self::ElemT {
        a * b
    }
    fn mul_refs(&self, a: &Self::ElemT, b: &Self::ElemT) -> Self::ElemT {
        a * b
    }

    fn div(&self, a: Self::ElemT, b: Self::ElemT) -> Result<Self::ElemT, RingDivisionError> {
        match self.quorem(a, b) {
            Some((q, r)) => {
                if r == self.zero() {
                    Ok(q)
                } else {
                    Err(RingDivisionError::NotDivisible)
                }
            }
            None => Err(RingDivisionError::DivideByZero),
        }
    }

    fn div_lref(&self, a: &Self::ElemT, b: Self::ElemT) -> Result<Self::ElemT, RingDivisionError> {
        match self.quorem_lref(a, b) {
            Some((q, r)) => {
                if r == self.zero() {
                    Ok(q)
                } else {
                    Err(RingDivisionError::NotDivisible)
                }
            }
            None => Err(RingDivisionError::DivideByZero),
        }
    }

    fn div_rref(&self, a: Self::ElemT, b: &Self::ElemT) -> Result<Self::ElemT, RingDivisionError> {
        match self.quorem_rref(a, b) {
            Some((q, r)) => {
                if r == self.zero() {
                    Ok(q)
                } else {
                    Err(RingDivisionError::NotDivisible)
                }
            }
            None => Err(RingDivisionError::DivideByZero),
        }
    }

    fn div_refs(&self, a: &Self::ElemT, b: &Self::ElemT) -> Result<Self::ElemT, RingDivisionError> {
        match self.quorem_refs(a, b) {
            Some((q, r)) => {
                if r == self.zero() {
                    Ok(q)
                } else {
                    Err(RingDivisionError::NotDivisible)
                }
            }
            None => Err(RingDivisionError::DivideByZero),
        }
    }
}

impl CharacteristicZero for IntegerRing {}

impl FiniteUnits for IntegerRing {
    fn all_units(&self) -> Vec<Self::ElemT> {
        vec![Self::ElemT::from(1), Self::ElemT::from(-1)]
    }
}

impl IntegralDomain for IntegerRing {}

impl FavoriteAssociate for IntegerRing {
    fn factor_fav_assoc(&self, elem: Self::ElemT) -> (Self::ElemT, Self::ElemT) {
        if elem == 0 {
            (self.one(), self.zero())
        } else if elem < 0 {
            (Integer::from(-1), self.neg(elem))
        } else {
            (Integer::from(1), elem)
        }
    }
}

impl UniqueFactorizationDomain for IntegerRing {
    fn factor(&self, elem: &Self::ElemT) -> Option<Factored<Self::ElemT>> {
        if elem == &0 {
            None
        } else {
            let unit;
            if elem < &0 {
                unit = Integer::from(-1);
            } else {
                unit = Integer::from(1);
            }

            fn factor_nat(mut n: Natural) -> HashMap<Natural, Natural> {
                //TODO: more efficient implementations
                assert_ne!(n, 0);
                let mut fs = HashMap::new();
                let mut p = Natural::from(2u8);
                while n > 1 && p <= n {
                    while &n % &p == 0 {
                        *fs.entry(p.clone()).or_insert(Natural::from(0u8)) += Natural::from(1u8);
                        n /= &p;
                    }
                    p += Natural::from(1u8);
                }
                fs
            }

            Some(Factored::new_unchecked(
                unit,
                factor_nat(elem.unsigned_abs())
                    .into_iter()
                    .map(|(p, k)| (Integer::from(p), k))
                    .collect(),
            ))
        }
    }
}

impl EuclideanDomain for IntegerRing {
    fn norm(&self, elem: &Self::ElemT) -> Option<Natural> {
        if elem == &Integer::from(0) {
            None
        } else {
            Some(elem.unsigned_abs())
        }
    }

    fn quorem(&self, a: Self::ElemT, b: Self::ElemT) -> Option<(Self::ElemT, Self::ElemT)> {
        if b == Integer::from(0) {
            None
        } else {
            Some(a.div_mod(b.clone()))
        }
    }
}

impl Real for IntegerRing {
    fn as_f64(&self, x: Self::ElemT) -> f64 {
        if x < 0 {
            -self.as_f64(-x)
        } else {
            let limbs = x.into_twos_complement_limbs_asc();
            let mut flt = 0.0;
            for (i, k) in limbs.into_iter().enumerate() {
                flt += (k as f64) * (2.0 as f64).powf(i as f64 * 64.0);
            }
            flt
        }
    }
}

impl DenseReal for RationalField {
    fn from_f64_approx(&self, x: f64) -> Self::ElemT {
        let mut x = Rational::from_sci_string_simplest(x.to_string().as_str()).unwrap();
        malachite_q::arithmetic::traits::ApproximateAssign::approximate_assign(
            &mut x,
            &Natural::from(self.from_f64_approx_max_denom),
        );
        x
    }
}

impl<'a> UniqueFactorizationDomain for PolynomialRing<'a, IntegerRing> {
    fn factor(&self, elem: &Self::ElemT) -> Option<Factored<Self::ElemT>> {
        //TODO: use a more efficient algorithm: reduce mod large prime and lift, knapsack alg using LLL to pick factor subsets efficiently
        self.factorize_by_kroneckers_method(elem)
    }
}

// impl InterpolatablePolynomials for ZZ {
//     fn interpolate(points: &Vec<(Self::ElemT, Self::ElemT)>) -> Option<Polynomial<Self>> {
//         Polynomial::interpolate_by_lagrange_basis(points)
//     }
// }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RationalField {
    from_f64_approx_max_denom: u128,
}
pub const QQ: RationalField = RationalField {
    from_f64_approx_max_denom: 100,
};

impl ComRing for RationalField {
    type ElemT = Rational;

    fn to_string(&self, elem: &Self::ElemT) -> String {
        elem.to_string()
    }

    fn equal(&self, a: &Self::ElemT, b: &Self::ElemT) -> bool {
        a == b
    }

    fn zero(&self) -> Self::ElemT {
        Self::ElemT::from(0)
    }
    fn one(&self) -> Self::ElemT {
        Self::ElemT::from(1)
    }

    fn neg_mut(&self, elem: &mut Self::ElemT) {
        *elem *= Self::ElemT::from(-1);
    }
    fn neg_ref(&self, elem: &Self::ElemT) -> Self::ElemT {
        -elem
    }
    fn neg(&self, elem: Self::ElemT) -> Self::ElemT {
        -elem
    }

    fn add_mut(&self, elem: &mut Self::ElemT, x: &Self::ElemT) {
        *elem += x;
    }
    fn add(&self, a: Self::ElemT, b: Self::ElemT) -> Self::ElemT {
        a + b
    }
    fn add_ref(&self, a: Self::ElemT, b: &Self::ElemT) -> Self::ElemT {
        a + b
    }
    fn add_refs(&self, a: &Self::ElemT, b: &Self::ElemT) -> Self::ElemT {
        a + b
    }

    fn mul_mut(&self, elem: &mut Self::ElemT, x: &Self::ElemT) {
        *elem *= x;
    }
    fn mul(&self, a: Self::ElemT, b: Self::ElemT) -> Self::ElemT {
        a * b
    }
    fn mul_ref(&self, a: Self::ElemT, b: &Self::ElemT) -> Self::ElemT {
        a * b
    }
    fn mul_refs(&self, a: &Self::ElemT, b: &Self::ElemT) -> Self::ElemT {
        a * b
    }

    fn div(&self, a: Self::ElemT, b: Self::ElemT) -> Result<Self::ElemT, RingDivisionError> {
        if b == Rational::from(0) {
            Err(RingDivisionError::DivideByZero)
        } else {
            Ok(a / b)
        }
    }
}
impl IntegralDomain for RationalField {}

impl Field for RationalField {}

impl FieldOfFractions for RationalField {
    type R = IntegerRing;

    fn base_ring(&self) -> &Self::R {
        &ZZ
    }

    fn numerator(&self, elem: &Self::ElemT) -> <Self::R as ComRing>::ElemT {
        if elem >= &0 {
            Integer::from(elem.numerator_ref())
        } else {
            -Integer::from(elem.numerator_ref())
        }
    }

    fn denominator(&self, elem: &Self::ElemT) -> <Self::R as ComRing>::ElemT {
        Integer::from(elem.denominator_ref())
    }

    fn from_base_ring(&self, elem: <Self::R as ComRing>::ElemT) -> Self::ElemT {
        Rational::from(elem)
    }
}

impl FiniteUnits for EuclideanQuotient<true, IntegerRing> {
    fn all_units(&self) -> Vec<Self::ElemT> {
        let mut units = vec![];
        let mut u = self.one();
        while u < self.get_n() {
            units.push(u.clone());
            u += self.one();
        }
        units
    }
}

pub struct NaturalPrimeGenerator {
    n: Natural,
    primes: Vec<Natural>,
}

impl NaturalPrimeGenerator {
    pub fn new() -> Self {
        Self {
            n: Natural::from(2u8),
            primes: vec![],
        }
    }
}

impl Iterator for NaturalPrimeGenerator {
    type Item = Natural;

    fn next(&mut self) -> Option<Self::Item> {
        'next_loop: loop {
            //todo: only check primes up to sqrt n
            for p in &self.primes {
                if &self.n % p == 0 {
                    self.n += Natural::from(1u8);
                    continue 'next_loop;
                }
            }
            let next_p = self.n.clone();
            self.n += Natural::from(1u8);
            self.primes.push(next_p.clone());
            return Some(next_p);
        }
    }
}

pub fn factorial(n: Natural) -> Natural {
    let mut k = Natural::from(1u8);
    let mut i = Natural::from(1u8);
    while i <= n {
        k *= &i;
        i += Natural::from(1u8);
    }
    k
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int() {
        //happy div
        {
            let a = Integer::from(18);
            let b = Integer::from(6);
            let c = ZZ.div(a, b);
            match c {
                Ok(_) => {}
                Err(_e) => panic!(),
            }
        }

        //sad div
        {
            let a = Integer::from(18);
            let b = Integer::from(7);
            let c = ZZ.div(a, b);
            match c {
                Ok(_) => panic!(),
                Err(e) => match e {
                    RingDivisionError::DivideByZero => panic!(),
                    RingDivisionError::NotDivisible => {}
                },
            }
        }

        //euclidean div
        {
            let a = Integer::from(18);
            let b = Integer::from(7);
            let (q, r) = ZZ.quorem_refs(&a, &b).unwrap();
            assert!(ZZ.norm(&r) < ZZ.norm(&b));
            assert_eq!(a, b * q + r);
        }

        //xgcd
        {
            let a = Integer::from(31);
            let b = Integer::from(57);
            let (g, x, y) = ZZ.xgcd(a.clone(), b.clone());
            assert_eq!(x * a + y * b, g);
        }
    }

    #[test]
    fn test_rational_numerator_and_denominator() {
        let x = Rational::from_signeds(-22, 7);
        let (n, d) = (QQ.numerator(&x), QQ.denominator(&x));
        assert_eq!(n, Integer::from(-22));
        assert_eq!(d, Integer::from(7));

        let x = Rational::from_signeds(22, -7);
        let (n, d) = (QQ.numerator(&x), QQ.denominator(&x));
        assert_eq!(n, Integer::from(-22));
        assert_eq!(d, Integer::from(7));

        let x = Rational::from_signeds(22, 7);
        let (n, d) = (QQ.numerator(&x), QQ.denominator(&x));
        assert_eq!(n, Integer::from(22));
        assert_eq!(d, Integer::from(7));

        let x = Rational::from_signeds(-22, -7);
        let (n, d) = (QQ.numerator(&x), QQ.denominator(&x));
        assert_eq!(n, Integer::from(22));
        assert_eq!(d, Integer::from(7));

        let x = Rational::from_signeds(0, 1);
        let (n, d) = (QQ.numerator(&x), QQ.denominator(&x));
        assert_eq!(n, Integer::from(0));
        assert_eq!(d, Integer::from(1));
    }

    // #[test]
    // fn test_factor_int() {}

    #[test]
    fn test_factorial() {
        debug_assert_eq!(factorial(Natural::from(0u8)), Natural::from(1u8));
        debug_assert_eq!(factorial(Natural::from(1u8)), Natural::from(1u8));
        debug_assert_eq!(factorial(Natural::from(2u8)), Natural::from(2u8));
        debug_assert_eq!(factorial(Natural::from(3u8)), Natural::from(6u8));
        debug_assert_eq!(factorial(Natural::from(4u8)), Natural::from(24u8));
        debug_assert_eq!(factorial(Natural::from(5u8)), Natural::from(120u8));
        debug_assert_eq!(factorial(Natural::from(6u8)), Natural::from(720u16));
    }
}
