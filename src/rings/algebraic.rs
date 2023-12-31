use malachite_base::num::arithmetic::traits::NegAssign;
use malachite_nz::integer::Integer;
use malachite_nz::natural::Natural;
use malachite_q::Rational;

use super::multipoly::*;
use super::nzq::*;
use super::poly::*;
use super::ring::ComRing;
use super::ring::*;

pub const QQ_BAR_REAL: RealAlgebraicField = RealAlgebraicField {};
pub const QQ_BAR: ComplexAlgebraicField = ComplexAlgebraicField {};

fn root_sum_poly(p: &Polynomial<Integer>, q: &Polynomial<Integer>) -> Polynomial<Integer> {
    let zz_poly_of_multi_poly = PolynomialRing::new(&ZZ_MULTIPOLY);

    let x = Variable::new(String::from("x"));
    let z = Variable::new(String::from("z"));

    let p = ZZ_POLY.apply_map(&ZZ_MULTIPOLY, p, |c| ZZ_MULTIPOLY.constant(c.clone()));
    let q = ZZ_POLY.apply_map(&ZZ_MULTIPOLY, q, |c| ZZ_MULTIPOLY.constant(c.clone()));
    let r = ZZ_MULTIPOLY.expand(
        &zz_poly_of_multi_poly.evaluate(
            &q,
            &ZZ_MULTIPOLY.add(
                ZZ_MULTIPOLY.var(z.clone()),
                ZZ_MULTIPOLY.neg(ZZ_MULTIPOLY.var(x.clone())),
            ),
        ),
        &x,
    );

    let root_sum_poly = zz_poly_of_multi_poly.apply_map(
        &ZZ,
        &ZZ_MULTIPOLY.expand(&zz_poly_of_multi_poly.resultant(p, r), &z),
        |c| ZZ_MULTIPOLY.as_constant(c).unwrap(),
    );
    ZZ_POLY.primitive_squarefree_part(root_sum_poly)
}

fn root_prod_poly(p: &Polynomial<Integer>, q: &Polynomial<Integer>) -> Polynomial<Integer> {
    let zz_poly_of_multi_poly = PolynomialRing::new(&ZZ_MULTIPOLY);

    let x = Variable::new(String::from("x"));
    let t = Variable::new(String::from("t"));

    let p = ZZ_POLY.apply_map(&ZZ_MULTIPOLY, p, |c| ZZ_MULTIPOLY.constant(c.clone()));
    let q = zz_poly_of_multi_poly.evaluate(
        &ZZ_POLY.apply_map(&ZZ_MULTIPOLY, q, |c| ZZ_MULTIPOLY.constant(c.clone())),
        &ZZ_MULTIPOLY.var(x.clone()),
    );
    let r = ZZ_MULTIPOLY.expand(&ZZ_MULTIPOLY.homogenize(&q, &t), &t);
    //x ** q.degree() * q(t * x ** -1)

    let root_prod_poly = zz_poly_of_multi_poly.apply_map(
        &ZZ,
        &ZZ_MULTIPOLY.expand(&zz_poly_of_multi_poly.resultant(p, r), &x),
        |c| ZZ_MULTIPOLY.as_constant(c).unwrap(),
    );
    ZZ_POLY.primitive_squarefree_part(root_prod_poly)
}

fn evaluate_at_rational(poly: &Polynomial<Integer>, val: &Rational) -> Rational {
    QQ_POLY.evaluate(&ZZ_POLY.apply_map(&QQ, poly, |x| Rational::from(x)), &val)
}

fn bisect_box(
    poly: &Polynomial<Integer>,
    n: usize,
    a: &Rational,
    b: &Rational,
    c: &Rational,
    d: &Rational,
) -> (
    (usize, Rational, Rational, Rational, Rational),
    (usize, Rational, Rational, Rational, Rational),
) {
    let pgen = NaturalPrimeGenerator::new();
    for y in pgen {
        let mut x = Natural::from(1u8);
        while x < y {
            {
                let f = Rational::from_naturals_ref(&x, &y);

                let ba = b - a;
                let dc = d - c;

                let ((a1, b1, c1, d1), (a2, b2, c2, d2)) = {
                    if ba >= dc {
                        let m = a + f * ba;
                        (
                            (a.clone(), m.clone(), c.clone(), d.clone()),
                            (m, b.clone(), c.clone(), d.clone()),
                        )
                    } else {
                        let m = c + f * dc;
                        (
                            (a.clone(), b.clone(), c.clone(), m.clone()),
                            (a.clone(), b.clone(), m, d.clone()),
                        )
                    }
                };

                match (
                    ZZ_POLY.count_complex_roots(poly, &a1, &b1, &c1, &d1),
                    ZZ_POLY.count_complex_roots(poly, &a2, &b2, &c2, &d2),
                ) {
                    (Some(n1), Some(n2)) => {
                        debug_assert_eq!(n1 + n2, n);
                        return ((n1, a1, b1, c1, d1), (n2, a2, b2, c2, d2));
                    }
                    _ => {}
                }
                x += Natural::from(1u8);
            }
        }
    }
    panic!();
}

#[derive(Debug, Clone)]
enum LowerBound {
    Inf,
    Finite(Rational),
}

#[derive(Debug, Clone)]
enum UpperBound {
    Inf,
    Finite(Rational),
}

impl LowerBound {
    pub fn neg(self) -> UpperBound {
        match self {
            LowerBound::Inf => UpperBound::Inf,
            LowerBound::Finite(a) => UpperBound::Finite(QQ.neg(a)),
        }
    }
}

impl UpperBound {
    pub fn neg(self) -> LowerBound {
        match self {
            UpperBound::Inf => LowerBound::Inf,
            UpperBound::Finite(a) => LowerBound::Finite(QQ.neg(a)),
        }
    }
}

impl std::hash::Hash for LowerBound {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            LowerBound::Inf => {}
            LowerBound::Finite(x) => {
                x.hash(state);
            }
        }
    }
}

impl std::hash::Hash for UpperBound {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            UpperBound::Inf => {}
            UpperBound::Finite(x) => {
                x.hash(state);
            }
        }
    }
}

impl PartialEq<UpperBound> for LowerBound {
    fn eq(&self, other: &UpperBound) -> bool {
        match (self, other) {
            (LowerBound::Finite(a), UpperBound::Finite(b)) => a == b,
            _ => false,
        }
    }
}

impl PartialOrd<UpperBound> for LowerBound {
    fn partial_cmp(&self, other: &UpperBound) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (LowerBound::Finite(a), UpperBound::Finite(b)) => a.partial_cmp(b),
            _ => Some(std::cmp::Ordering::Less),
        }
    }
}

impl PartialEq<Rational> for LowerBound {
    fn eq(&self, b: &Rational) -> bool {
        match self {
            LowerBound::Finite(a) => a == b,
            _ => false,
        }
    }
}

impl PartialOrd<Rational> for LowerBound {
    fn partial_cmp(&self, b: &Rational) -> Option<std::cmp::Ordering> {
        match self {
            LowerBound::Finite(a) => a.partial_cmp(b),
            _ => Some(std::cmp::Ordering::Less),
        }
    }
}

impl PartialEq<UpperBound> for Rational {
    fn eq(&self, other: &UpperBound) -> bool {
        match other {
            UpperBound::Finite(b) => self == b,
            _ => false,
        }
    }
}

impl PartialOrd<UpperBound> for Rational {
    fn partial_cmp(&self, other: &UpperBound) -> Option<std::cmp::Ordering> {
        match other {
            UpperBound::Finite(b) => self.partial_cmp(b),
            _ => Some(std::cmp::Ordering::Less),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Interleave {
    First,
    Second,
}

#[derive(Debug, Clone)]
enum SquarefreePolyRealRootInterval {
    Rational(Rational),
    //lower bound, upper bound, increasing
    //increasing = false : decreasing i.e. poly(a) > poly(b), true : increasing i.e. poly(a) < poly(b)
    Real(Rational, Rational, bool),
}

#[derive(Debug, Clone)]
struct SquarefreePolyRealRoots {
    poly_sqfr: Polynomial<Integer>,
    //an ordered list of isolating intervals for the squarefree polynomial
    //e.g. if r represents a real algebraic number and | represents a rational root
    //        (      r    )      |  ( r     )   |   |   (        r   )
    //note: it is allowed that some r might actually be rational but not known to be
    intervals: Vec<SquarefreePolyRealRootInterval>,
}

impl SquarefreePolyRealRoots {
    pub fn check_invariants(&self) -> Result<(), &'static str> {
        //poly should be squarefree
        if ZZ_POLY
            .degree(&ZZ_POLY.primitive_squarefree_part(self.poly_sqfr.clone()))
            .unwrap()
            != ZZ_POLY.degree(&self.poly_sqfr).unwrap()
        {
            return Err("poly should be squarefree");
        }

        //check the isolating intervals
        if self.intervals.len() != 0 {
            for i in 0..self.intervals.len() - 1 {
                let int1 = &self.intervals[i];
                let int2 = &self.intervals[i + 1];
                match (int1, int2) {
                    (
                        SquarefreePolyRealRootInterval::Rational(a),
                        SquarefreePolyRealRootInterval::Rational(x),
                    ) => {
                        if !(a < x) {
                            return Err("interval values should be strictly increasing");
                        }
                    }
                    (
                        SquarefreePolyRealRootInterval::Rational(a),
                        SquarefreePolyRealRootInterval::Real(x, y, _),
                    ) => {
                        if !(a < x) {
                            return Err("interval values should be strictly increasing");
                        }
                        if !(x < y) {
                            return Err("interval values should be strictly increasing");
                        }
                    }
                    (
                        SquarefreePolyRealRootInterval::Real(a, b, _),
                        SquarefreePolyRealRootInterval::Rational(x),
                    ) => {
                        if !(a < b) {
                            return Err("interval values should be strictly increasing");
                        }
                        if !(b < x) {
                            return Err("interval values should be strictly increasing");
                        }
                    }
                    (
                        SquarefreePolyRealRootInterval::Real(a, b, _),
                        SquarefreePolyRealRootInterval::Real(x, y, _),
                    ) => {
                        if !(a < b) {
                            return Err("interval values should be strictly increasing");
                        }
                        if !(b <= x) {
                            return Err("interval values should be increasing");
                        }
                        if !(x < y) {
                            return Err("interval values should be strictly increasing");
                        }
                    }
                }
            }
        }

        for interval in self.intervals.iter() {
            match interval {
                SquarefreePolyRealRootInterval::Rational(a) => {
                    if evaluate_at_rational(&self.poly_sqfr, a) != Rational::from(0) {
                        return Err("poly should be zero at a rational root");
                    }
                }
                SquarefreePolyRealRootInterval::Real(a, b, incr) => {
                    let at_a = evaluate_at_rational(&self.poly_sqfr, a);
                    let at_b = evaluate_at_rational(&self.poly_sqfr, b);

                    if at_a == Rational::from(0) || at_b == Rational::from(0) {
                        return Err("poly should not be zero at boundary of isolating interval");
                    }

                    if (at_a > Rational::from(0)) == (at_b > Rational::from(0)) {
                        return Err("sign of poly should be different at a and at b");
                    }

                    if *incr {
                        if !((at_a < Rational::from(0)) && (at_b > Rational::from(0))) {
                            return Err("sign of poly should go from neg to pos here");
                        }
                    } else {
                        if !((at_a > Rational::from(0)) && (at_b < Rational::from(0))) {
                            return Err("sign of poly should go from pos to neg here");
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn get_wide_interval(&self, idx: usize) -> (LowerBound, UpperBound) {
        assert!(idx < self.intervals.len());

        let wide_a = {
            if idx == 0 {
                LowerBound::Inf
            } else {
                LowerBound::Finite({
                    match &self.intervals[idx - 1] {
                        SquarefreePolyRealRootInterval::Rational(a) => a.clone(),
                        SquarefreePolyRealRootInterval::Real(_, prev_b, _) => prev_b.clone(),
                    }
                })
            }
        };
        let wide_b = {
            if idx == self.intervals.len() - 1 {
                UpperBound::Inf
            } else {
                UpperBound::Finite({
                    match &self.intervals[idx + 1] {
                        SquarefreePolyRealRootInterval::Rational(a) => a.clone(),
                        SquarefreePolyRealRootInterval::Real(prev_a, _, _) => prev_a.clone(),
                    }
                })
            }
        };
        debug_assert!(wide_a.clone() < wide_b.clone());
        (wide_a, wide_b)
    }

    fn refine(&mut self, idx: usize) {
        assert!(idx < self.intervals.len());

        match &mut self.intervals[idx] {
            SquarefreePolyRealRootInterval::Rational(_a) => {}
            SquarefreePolyRealRootInterval::Real(a, b, dir) => {
                let m = (&*a + &*b) / Rational::from(2);
                match evaluate_at_rational(&self.poly_sqfr, &m).cmp(&Rational::from(0)) {
                    std::cmp::Ordering::Less => match dir {
                        true => {
                            *a = m;
                        }
                        false => {
                            *b = m;
                        }
                    },
                    std::cmp::Ordering::Equal => {
                        self.intervals[idx] = SquarefreePolyRealRootInterval::Rational(m);
                    }
                    std::cmp::Ordering::Greater => match dir {
                        true => {
                            *b = m;
                        }
                        false => {
                            *a = m;
                        }
                    },
                }
            }
        }
    }

    fn refine_all(&mut self) {
        for idx in 0..self.intervals.len() {
            self.refine(idx);
        }
    }

    fn to_real_roots(self) -> Vec<RealAlgebraicNumber> {
        debug_assert!(ZZ_POLY.is_irreducible(&self.poly_sqfr).unwrap());
        let deg = ZZ_POLY.degree(&self.poly_sqfr).unwrap();
        if deg == 0 {
            vec![]
        } else if deg == 1 {
            if self.intervals.len() == 0 {
                vec![]
            } else if self.intervals.len() == 1 {
                match self.intervals.into_iter().next().unwrap() {
                    SquarefreePolyRealRootInterval::Rational(a) => {
                        vec![RealAlgebraicNumber::Rational(a)]
                    }
                    SquarefreePolyRealRootInterval::Real(_, _, _) => {
                        // panic!("degree 1 polynomial should have had rational root found exactly");
                        vec![RealAlgebraicNumber::Rational(-Rational::from_integers(
                            ZZ_POLY.coeff(&self.poly_sqfr, 0),
                            ZZ_POLY.coeff(&self.poly_sqfr, 1),
                        ))]
                    }
                }
            } else {
                panic!();
            }
        } else {
            let mut roots = vec![];
            for (idx, interval) in self.intervals.iter().enumerate() {
                roots.push({
                    match interval {
                        SquarefreePolyRealRootInterval::Rational(a) => {
                            RealAlgebraicNumber::Rational(a.clone())
                        }
                        SquarefreePolyRealRootInterval::Real(tight_a, tight_b, dir) => {
                            let (wide_a, wide_b) = self.get_wide_interval(idx);
                            RealAlgebraicNumber::Real(RealAlgebraicRoot {
                                poly: self.poly_sqfr.clone(),
                                tight_a: tight_a.clone(),
                                tight_b: tight_b.clone(),
                                wide_a,
                                wide_b,
                                dir: *dir,
                            })
                        }
                    }
                });
            }
            roots
        }
    }

    //separate the isolating intervals of the roots in roots1 and roots2
    //return Err if a root in roots1 and a root in roots2 are equal and thus cant be separated
    //ends of real root intervals should not equal rational roots in the other one
    fn separate(roots1: &mut Self, roots2: &mut Self) -> Result<Vec<(Interleave, usize)>, ()> {
        let poly_gcd_sqfr =
            ZZ_POLY.subresultant_gcd(roots1.poly_sqfr.clone(), roots2.poly_sqfr.clone());
        let (_, poly_gcd_sqfr) = ZZ_POLY.factor_primitive(poly_gcd_sqfr).unwrap();

        //compute which of the roots are equals to some root from the other one
        let is_gcdroot_1: Vec<_> = roots1
            .intervals
            .iter()
            .map(|root| match root {
                SquarefreePolyRealRootInterval::Rational(x) => {
                    evaluate_at_rational(&poly_gcd_sqfr, x) == Rational::from(0)
                }
                SquarefreePolyRealRootInterval::Real(a, b, _dir) => {
                    debug_assert_ne!(evaluate_at_rational(&poly_gcd_sqfr, a), Rational::from(0));
                    debug_assert_ne!(evaluate_at_rational(&poly_gcd_sqfr, b), Rational::from(0));
                    (evaluate_at_rational(&poly_gcd_sqfr, a) > Rational::from(0))
                        != (evaluate_at_rational(&poly_gcd_sqfr, b) > Rational::from(0))
                }
            })
            .collect();
        let is_gcdroot_2: Vec<_> = roots2
            .intervals
            .iter()
            .map(|root| match root {
                SquarefreePolyRealRootInterval::Rational(x) => {
                    evaluate_at_rational(&poly_gcd_sqfr, x) == Rational::from(0)
                }
                SquarefreePolyRealRootInterval::Real(a, b, _dir) => {
                    debug_assert_ne!(evaluate_at_rational(&poly_gcd_sqfr, a), Rational::from(0));
                    debug_assert_ne!(evaluate_at_rational(&poly_gcd_sqfr, b), Rational::from(0));
                    (evaluate_at_rational(&poly_gcd_sqfr, a) > Rational::from(0))
                        != (evaluate_at_rational(&poly_gcd_sqfr, b) > Rational::from(0))
                }
            })
            .collect();

        //do the separation
        let mut all_roots = vec![];

        let mut idx1 = 0;
        let mut idx2 = 0;
        while idx1 < roots1.intervals.len() && idx2 < roots2.intervals.len() {
            let (wide1_a, wide1_b) = roots1.get_wide_interval(idx1);
            let (wide2_a, wide2_b) = roots2.get_wide_interval(idx2);

            let root1 = &roots1.intervals[idx1];
            let root2 = &roots2.intervals[idx2];

            //check if the roots are equal
            if is_gcdroot_1[idx1] && is_gcdroot_2[idx2] {
                match root1 {
                    SquarefreePolyRealRootInterval::Rational(x1) => {
                        if &wide2_a < x1 && x1 < &wide2_b {
                            return Err(());
                        }
                    }
                    SquarefreePolyRealRootInterval::Real(a1, b1, _dir1) => {
                        if &wide2_a < a1 && b1 < &wide2_b {
                            return Err(());
                        }
                    }
                }
                match root2 {
                    SquarefreePolyRealRootInterval::Rational(x2) => {
                        if &wide1_a < x2 && x2 < &wide1_b {
                            return Err(());
                        }
                    }
                    SquarefreePolyRealRootInterval::Real(a2, b2, _dir2) => {
                        if &wide1_a < a2 && b2 < &wide1_b {
                            return Err(());
                        }
                    }
                }
            }

            //check if one is bigger than the other
            match (root1, root2) {
                (
                    SquarefreePolyRealRootInterval::Rational(x1),
                    SquarefreePolyRealRootInterval::Rational(x2),
                ) => match x1.cmp(x2) {
                    std::cmp::Ordering::Less => {
                        all_roots.push((Interleave::First, idx1));
                        idx1 += 1;
                        continue;
                    }
                    std::cmp::Ordering::Equal => panic!(),
                    std::cmp::Ordering::Greater => {
                        all_roots.push((Interleave::Second, idx2));
                        idx2 += 1;
                        continue;
                    }
                },
                (
                    SquarefreePolyRealRootInterval::Rational(x1),
                    SquarefreePolyRealRootInterval::Real(a2, b2, _dir2),
                ) => {
                    if x1 < a2 {
                        all_roots.push((Interleave::First, idx1));
                        idx1 += 1;
                        continue;
                    }
                    if b2 < x1 {
                        all_roots.push((Interleave::Second, idx2));
                        idx2 += 1;
                        continue;
                    }
                }
                (
                    SquarefreePolyRealRootInterval::Real(a1, b1, _dir1),
                    SquarefreePolyRealRootInterval::Rational(x2),
                ) => {
                    if x2 < a1 {
                        all_roots.push((Interleave::Second, idx2));
                        idx2 += 1;
                        continue;
                    }
                    if b1 < x2 {
                        all_roots.push((Interleave::First, idx1));
                        idx1 += 1;
                        continue;
                    }
                }
                (
                    SquarefreePolyRealRootInterval::Real(a1, b1, _dir1),
                    SquarefreePolyRealRootInterval::Real(a2, b2, _dir2),
                ) => {
                    if b2 < a1 {
                        all_roots.push((Interleave::Second, idx2));
                        idx2 += 1;
                        continue;
                    }
                    if b1 < a2 {
                        all_roots.push((Interleave::First, idx1));
                        idx1 += 1;
                        continue;
                    }
                }
            }

            //refine and try again
            roots1.refine(idx1);
            roots2.refine(idx2);
        }

        debug_assert!(idx1 == roots1.intervals.len() || idx2 == roots2.intervals.len());

        while idx1 < roots1.intervals.len() {
            all_roots.push((Interleave::First, idx1));
            idx1 += 1;
        }

        while idx2 < roots2.intervals.len() {
            all_roots.push((Interleave::Second, idx2));
            idx2 += 1;
        }

        for r1 in &roots1.intervals {
            for r2 in &roots2.intervals {
                match (r1, r2) {
                    (
                        SquarefreePolyRealRootInterval::Rational(a),
                        SquarefreePolyRealRootInterval::Rational(x),
                    ) => {
                        debug_assert!(a != x);
                    }
                    (
                        SquarefreePolyRealRootInterval::Rational(a),
                        SquarefreePolyRealRootInterval::Real(x, y, _),
                    ) => {
                        debug_assert!(a < x || y < a);
                    }
                    (
                        SquarefreePolyRealRootInterval::Real(a, b, _),
                        SquarefreePolyRealRootInterval::Rational(x),
                    ) => {
                        debug_assert!(x < a || b < x);
                    }
                    (
                        SquarefreePolyRealRootInterval::Real(a, b, _),
                        SquarefreePolyRealRootInterval::Real(x, y, _),
                    ) => {
                        debug_assert!(b < x || y < a);
                    }
                }
            }
        }

        debug_assert_eq!(
            all_roots.len(),
            roots1.intervals.len() + roots2.intervals.len()
        );

        Ok(all_roots)
    }
}

impl<'a> PolynomialRing<'a, IntegerRing> {
    fn sign_variations(&self, poly: &Polynomial<Integer>) -> usize {
        //https://en.wikipedia.org/wiki/Descartes'_rule_of_signs
        //equals the number of strictly positive real roots modulo 2
        //and number of positive real roots is less than this number
        let nonzero_coeffs: Vec<_> = poly
            .coeffs()
            .into_iter()
            .filter(|c| c != &self.ring().zero())
            .collect();
        let mut v = 0;
        for i in 0..nonzero_coeffs.len() - 1 {
            if (nonzero_coeffs[i] < 0) != (nonzero_coeffs[i + 1] < 0) {
                v += 1;
            }
        }
        v
    }

    //Collins and Akritas algorithm %https://en.wikipedia.org/wiki/Real-root_isolation
    fn isolate_real_roots_by_collin_akritas(
        &self,
        poly: &Polynomial<Integer>,
    ) -> Vec<(Natural, usize, bool)> {
        //input: p(x), a square-free polynomial, such that p(0) p(1) ≠ 0, for which the roots in the interval [0, 1] are searched
        //output: a list of triples (c, k, h) representing isolating intervals of the form [c/2^k, (c+h)/2^k]
        debug_assert_ne!(self.evaluate(poly, &self.ring().zero()), self.ring().zero());
        debug_assert_ne!(self.evaluate(poly, &self.ring().one()), self.ring().zero());
        debug_assert_eq!(
            self.degree(&self.primitive_squarefree_part(poly.clone()))
                .unwrap(),
            self.degree(poly).unwrap()
        );

        let mut l = vec![(Natural::from(0u8), 0, poly.clone())];
        let mut isol = vec![];
        while l.len() != 0 {
            let (c, k, mut q) = l.pop().unwrap();
            if ZZ_POLY.evaluate(&q, &Integer::from(0)) == Integer::from(0) {
                //q = q/x
                q = self.div(q, self.var()).unwrap();
                isol.push((c.clone(), k.clone(), false)); //rational root
            }
            let v = self.sign_variations(&self.compose(
                &self.reversed(&q),
                &self.from_coeffs(vec![Integer::from(1), Integer::from(1)]),
            ));
            if v == 1 {
                isol.push((c, k, true)); //root
            } else if v > 1 {
                //bisect
                //q_small(x) = 2^n q(x/2)
                let q_small = self.apply_map_with_powers(&ZZ, &q, |(i, coeff)| {
                    coeff * Integer::from(2) << (ZZ_POLY.degree(&q).unwrap() - i)
                });
                l.push((
                    (c.clone() << 1) + Natural::from(1u8),
                    k + 1,
                    self.compose(
                        &q_small,
                        &self.from_coeffs(vec![Integer::from(1), Integer::from(1)]),
                    ),
                ));
                l.push((c << 1, k + 1, q_small));
            }
        }
        isol
    }

    //isolate all real roots of a squarefree (no repeated roots) polynomial between a and b
    fn real_roots_squarefree(
        &self,
        poly: Polynomial<Integer>,
        opt_a: Option<&Rational>,
        opt_b: Option<&Rational>,
        include_a: bool,
        include_b: bool,
    ) -> SquarefreePolyRealRoots {
        assert!(!ZZ_POLY.equal(&poly, &self.zero()));
        //poly should be squarefree
        debug_assert_eq!(
            self.degree(&self.primitive_squarefree_part(poly.clone()))
                .unwrap(),
            self.degree(&poly).unwrap()
        );

        match (opt_a, opt_b) {
            (Some(a), Some(b)) => {
                assert!(a < b);
            }
            _ => {}
        }

        let d = ZZ_POLY.degree(&poly).unwrap();
        if d == 0 {
            //constant polynomial has no roots
            SquarefreePolyRealRoots {
                poly_sqfr: poly,
                intervals: vec![],
            }
        } else if d == 1 {
            //poly = a+bx
            //root = -a/b
            let root = -Rational::from(self.coeff(&poly, 0)) / Rational::from(self.coeff(&poly, 1));

            if {
                match opt_a {
                    Some(a) => match a.cmp(&root) {
                        std::cmp::Ordering::Less => true,
                        std::cmp::Ordering::Equal => include_a,
                        std::cmp::Ordering::Greater => false,
                    },
                    None => true,
                }
            } && {
                match opt_b {
                    Some(b) => match b.cmp(&root) {
                        std::cmp::Ordering::Greater => true,
                        std::cmp::Ordering::Equal => include_b,
                        std::cmp::Ordering::Less => false,
                    },
                    None => true,
                }
            } {
                SquarefreePolyRealRoots {
                    poly_sqfr: poly,
                    intervals: vec![SquarefreePolyRealRootInterval::Rational(root)],
                }
            } else {
                SquarefreePolyRealRoots {
                    poly_sqfr: poly,
                    intervals: vec![],
                }
            }
        } else {
            if opt_a.is_none() || opt_b.is_none() {
                //compute a bound M on the absolute value of any root
                //m = (Cauchy's bound + 1) https://captainblack.wordpress.com/2009/03/08/cauchys-upper-bound-for-the-roots-of-a-polynomial/
                let m = Rational::from(2)
                    + Rational::from_integers(
                        Integer::from(
                            itertools::max(
                                (0..d).map(|i| ZZ_POLY.coeff(&poly, i).unsigned_abs_ref().clone()),
                            )
                            .unwrap(),
                        ),
                        ZZ_POLY.coeff(&poly, d),
                    );

                return match opt_a {
                    Some(a_val) => match opt_b {
                        Some(_b_val) => panic!(),
                        None => self.real_roots_squarefree(
                            poly,
                            Some(a_val),
                            Some(&m),
                            include_a,
                            include_b,
                        ),
                    },
                    None => match opt_b {
                        Some(b_val) => self.real_roots_squarefree(
                            poly,
                            Some(&-m),
                            Some(b_val),
                            include_a,
                            include_b,
                        ),
                        None => {
                            let neg_m = -m.clone();
                            self.real_roots_squarefree(
                                poly,
                                Some(&neg_m),
                                Some(&m),
                                include_a,
                                include_b,
                            )
                        }
                    },
                };
            }
            let (a, b) = (opt_a.unwrap(), opt_b.unwrap());
            debug_assert!(a < b);

            //deal with end roots
            let mut poly_no_endroots = poly.clone();
            let mut intervals = vec![];
            if evaluate_at_rational(&poly, a) == Rational::from(0) {
                poly_no_endroots = self
                    .div(
                        poly_no_endroots,
                        ZZ_POLY.from_coeffs(vec![-QQ.numerator(a), QQ.denominator(a)]),
                    )
                    .unwrap();
                if include_a {
                    intervals.push(SquarefreePolyRealRootInterval::Rational(a.clone()));
                }
            }
            let mut do_add_b = false;
            if evaluate_at_rational(&poly, b) == Rational::from(0) {
                poly_no_endroots = self
                    .div(
                        poly_no_endroots,
                        ZZ_POLY.from_coeffs(vec![-QQ.numerator(b), QQ.denominator(b)]),
                    )
                    .unwrap();
                if include_b {
                    do_add_b = true;
                }
            }

            debug_assert_ne!(
                evaluate_at_rational(&poly_no_endroots, a),
                Rational::from(0)
            );
            debug_assert_ne!(
                evaluate_at_rational(&poly_no_endroots, b),
                Rational::from(0)
            );

            //apply a transformation to p so that its roots in (a, b) are moved to roots in (0, 1)
            let (_, trans_poly) = QQ_POLY.factor_primitive_fof(&QQ_POLY.compose(
                &ZZ_POLY.apply_map(&QQ, &poly_no_endroots, |c| Rational::from(c)),
                &QQ_POLY.from_coeffs(vec![a.clone(), b.clone() - a.clone()]),
            ));

            for (c, k, h) in self.isolate_real_roots_by_collin_akritas(&trans_poly) {
                let d = Natural::from(1u8) << k;
                let mut interval_a = (b - a) * Rational::from_naturals(c.clone(), d.clone()) + a;
                if h {
                    let mut interval_b =
                        (b - a) * Rational::from_naturals(&c + Natural::from(1u8), d.clone()) + a;

                    //at the moment, interval_a and interval_b might be rational roots
                    //we need to strink them a little bit if so
                    if evaluate_at_rational(&poly, &interval_a) == Rational::from(0)
                        || evaluate_at_rational(&poly, &interval_b) == Rational::from(0)
                    {
                        let interval_m = (&interval_a + &interval_b) / Rational::from(2);
                        let mut shrunk_inerval_a = (&interval_a + &interval_m) / Rational::from(2);
                        let mut shrunk_inerval_b = (&interval_m + &interval_b) / Rational::from(2);
                        debug_assert_ne!(
                            evaluate_at_rational(&poly, &shrunk_inerval_a),
                            Rational::from(0)
                        );
                        debug_assert_ne!(
                            evaluate_at_rational(&poly, &shrunk_inerval_b),
                            Rational::from(0)
                        );
                        while (evaluate_at_rational(&poly, &shrunk_inerval_a) > Rational::from(0))
                            == (evaluate_at_rational(&poly, &shrunk_inerval_b) > Rational::from(0))
                        {
                            shrunk_inerval_a =
                                (&interval_a + &shrunk_inerval_a) / Rational::from(2);
                            shrunk_inerval_b =
                                (&shrunk_inerval_b + &interval_b) / Rational::from(2);
                        }
                        interval_a = shrunk_inerval_a;
                        interval_b = shrunk_inerval_b;
                    }

                    let sign_b = evaluate_at_rational(&poly, &interval_b) > Rational::from(0);
                    debug_assert_ne!(evaluate_at_rational(&poly, &interval_a), Rational::from(0));
                    debug_assert_ne!(evaluate_at_rational(&poly, &interval_b), Rational::from(0));
                    debug_assert_ne!(
                        evaluate_at_rational(&poly, &interval_a) > Rational::from(0),
                        sign_b
                    );
                    intervals.push(SquarefreePolyRealRootInterval::Real(
                        interval_a, interval_b, sign_b,
                    ));
                } else {
                    intervals.push(SquarefreePolyRealRootInterval::Rational(interval_a));
                }
            }

            if do_add_b {
                intervals.push(SquarefreePolyRealRootInterval::Rational(b.clone()));
            }

            let roots = SquarefreePolyRealRoots {
                poly_sqfr: poly,
                intervals,
            };
            debug_assert!(roots.check_invariants().is_ok());
            roots
        }
    }

    //isolate all real roots of the irreducible poly in the open interval (a, b)
    fn all_real_roots_squarefree(&self, poly: &Polynomial<Integer>) -> SquarefreePolyRealRoots {
        self.real_roots_squarefree(poly.clone(), None, None, false, false)
    }

    //isolate all real roots of the irreducible poly in the open interval (a, b)
    fn real_roots_irreducible(
        &self,
        poly: &Polynomial<Integer>,
        opt_a: Option<&Rational>,
        opt_b: Option<&Rational>,
        include_a: bool,
        include_b: bool,
    ) -> Vec<RealAlgebraicNumber> {
        assert!(!self.equal(poly, &self.zero()));
        debug_assert!(self.is_irreducible(&poly).unwrap());

        self.real_roots_squarefree(poly.clone(), opt_a, opt_b, include_a, include_b)
            .to_real_roots()
    }

    //get the real roots with multiplicity of poly
    pub fn real_roots(
        &self,
        poly: &Polynomial<Integer>,
        a: Option<&Rational>,
        b: Option<&Rational>,
        include_a: bool,
        include_b: bool,
    ) -> Vec<RealAlgebraicNumber> {
        assert!(!self.equal(poly, &self.zero()));
        let factors = self.factor(&poly).unwrap();
        let mut roots = vec![];
        for (factor, k) in factors.factors() {
            for root in self.real_roots_irreducible(factor, a, b, include_a, include_b) {
                let mut i = Natural::from(0u8);
                while &i < k {
                    roots.push(root.clone());
                    i += Natural::from(1u8);
                }
            }
        }
        roots
    }

    pub fn all_real_roots(&self, poly: &Polynomial<Integer>) -> Vec<RealAlgebraicNumber> {
        self.real_roots(poly, None, None, false, false)
    }

    fn at_fixed_re_or_im_impl<const RE_OR_IM: bool>(
        &self,
        poly: &Polynomial<Integer>,
        a: &Rational,
    ) -> (Polynomial<Integer>, Polynomial<Integer>) {
        //find real and imag polys of
        //poly(a + xi) if RE_OR_IM = false
        //poly(x + ai) if RE_OR_IM = true
        //up to rational multiples (its the roots we care about)
        match self.degree(&poly) {
            Some(n) => {
                let (a_numer, a_denom) = (QQ.numerator(a), QQ.denominator(a));
                //multiply everything by a_d^n so that everything is integers

                //compute 1, a, a^2, a^3, ..., a^n (after multiplying everything by a_d)
                // a_d^n(a_n/a_d)^k = a_n^k a_d^{n-k}
                let mut a_numer_pow = vec![Integer::from(1)];
                let mut a_denom_pow = vec![Integer::from(1)];
                for k in 1..n + 1 {
                    a_numer_pow.push(&a_numer * &a_numer_pow[k - 1]);
                    a_denom_pow.push(&a_denom * &a_denom_pow[k - 1]);
                }
                let mut a_pow = vec![];
                for k in 0..n + 1 {
                    a_pow.push(&a_numer_pow[k] * &a_denom_pow[n - k]);
                }

                let mut re = Vec::with_capacity(n + 1);
                let mut im = Vec::with_capacity(n + 1);
                for _ in 0..n + 1 {
                    re.push(Integer::from(0));
                    im.push(Integer::from(0));
                }
                let mut n_choose = vec![Integer::from(1)];
                for n in 0..n + 1 {
                    if n == 0 {
                        debug_assert_eq!(n_choose, vec![Integer::from(1)]);
                    } else if n == 1 {
                        debug_assert_eq!(n_choose, vec![Integer::from(1), Integer::from(1)]);
                    } else if n == 2 {
                        debug_assert_eq!(
                            n_choose,
                            vec![Integer::from(1), Integer::from(2), Integer::from(1)]
                        );
                    } else if n == 3 {
                        debug_assert_eq!(
                            n_choose,
                            vec![
                                Integer::from(1),
                                Integer::from(3),
                                Integer::from(3),
                                Integer::from(1)
                            ]
                        );
                    }

                    //if fixed real add
                    //(a + xi)^n = \sum_{k=0,1,...,n} \binom{n}{k} a^{n-k} (xi)^k
                    //           = \sum_{k=0,1,...,n} \binom{n}{k} a^{n-k} x^k i^k
                    //           = \sum_{k=0,1,...,n} {
                    //               k = 0 mod 4        + \binom{n}{k} a^{n-k} x^k
                    //               k = 1 mod 4        + \binom{n}{k} a^{n-k} x^k i
                    //               k = 2 mod 4        - \binom{n}{k} a^{n-k} x^k
                    //               k = 3 mod 4        - \binom{n}{k} a^{n-k} x^k i
                    //                                }
                    //
                    //if fixed imag add
                    //(a + xi)^n = \sum_{k=0,1,...,n} \binom{n}{k} a^{n-k} (xi)^k
                    //           = \sum_{k=0,1,...,n} \binom{n}{k} a^{n-k} x^k i^k
                    //           = \sum_{k=0,1,...,n} {
                    //               k = 0 mod 4        + \binom{n}{k} a^{n-k} x^k
                    //               k = 1 mod 4        + \binom{n}{k} a^{n-k} x^k i
                    //               k = 2 mod 4        - \binom{n}{k} a^{n-k} x^k
                    //               k = 3 mod 4        - \binom{n}{k} a^{n-k} x^k i
                    //
                    if self.coeff(&poly, n) != Integer::from(0) {
                        let mut k = 0;
                        loop {
                            //k = 0 mod 4
                            re[{
                                match RE_OR_IM {
                                    false => k,
                                    true => n - k,
                                }
                            }] += self.coeff(&poly, n)
                                * &n_choose[k]
                                * &a_pow[{
                                    match RE_OR_IM {
                                        false => n - k,
                                        true => k,
                                    }
                                }];
                            if k == n {
                                break;
                            }
                            k += 1;
                            //k = 1 mod 4
                            im[{
                                match RE_OR_IM {
                                    false => k,
                                    true => n - k,
                                }
                            }] += self.coeff(&poly, n)
                                * &n_choose[k]
                                * &a_pow[{
                                    match RE_OR_IM {
                                        false => n - k,
                                        true => k,
                                    }
                                }];
                            if k == n {
                                break;
                            }
                            k += 1;
                            //k = 2 mod 4
                            re[{
                                match RE_OR_IM {
                                    false => k,
                                    true => n - k,
                                }
                            }] -= self.coeff(&poly, n)
                                * &n_choose[k]
                                * &a_pow[{
                                    match RE_OR_IM {
                                        false => n - k,
                                        true => k,
                                    }
                                }];
                            if k == n {
                                break;
                            }
                            k += 1;
                            //k = 3 mod 4
                            im[{
                                match RE_OR_IM {
                                    false => k,
                                    true => n - k,
                                }
                            }] -= self.coeff(&poly, n)
                                * &n_choose[k]
                                * &a_pow[{
                                    match RE_OR_IM {
                                        false => n - k,
                                        true => k,
                                    }
                                }];
                            if k == n {
                                break;
                            }
                            k += 1;
                        }
                    }
                    //update n choose k
                    //e.g. for n=3 do
                    //[1, 3, 3, 1]
                    //[1, 3, 3, 1, 1]
                    //[1, 3, 3, 4, 1]
                    //[1, 3, 6, 4, 1]
                    //[1, 4, 6, 4, 1]
                    n_choose.push(Integer::from(1));
                    for i in (1..n + 1).rev() {
                        n_choose[i] = &n_choose[i] + &n_choose[i - 1];
                    }
                }
                (ZZ_POLY.from_coeffs(re), ZZ_POLY.from_coeffs(im))
            }
            None => (self.zero(), self.zero()),
        }
    }

    fn at_fixed_re(
        &self,
        poly: &Polynomial<Integer>,
        a: &Rational,
    ) -> (Polynomial<Integer>, Polynomial<Integer>) {
        self.at_fixed_re_or_im_impl::<false>(poly, a)
    }

    fn at_fixed_im(
        &self,
        poly: &Polynomial<Integer>,
        a: &Rational,
    ) -> (Polynomial<Integer>, Polynomial<Integer>) {
        self.at_fixed_re_or_im_impl::<true>(poly, a)
    }

    //count how many complex roots are in the box a < re < b, c < im < d
    //or return None if there is a root on the boundary
    pub fn count_complex_roots(
        &self,
        poly: &Polynomial<Integer>,
        a: &Rational,
        b: &Rational,
        c: &Rational,
        d: &Rational,
    ) -> Option<usize> {
        assert!(a < b);
        assert!(c < d);

        //the idea is to compute the winding number of the path around the boundary of the box
        //this is done by computing where the value of the polynomial crosses the real and imaginary axes as the input traces the path
        //the crossing points and their order is done using the exact total ordering of real polynomial roots
        let (a_vert_re, a_vert_im) = self.at_fixed_re(poly, a);
        let (b_vert_re, b_vert_im) = self.at_fixed_re(poly, b);
        let (c_horz_re, c_horz_im) = self.at_fixed_im(poly, c);
        let (d_horz_re, d_horz_im) = self.at_fixed_im(poly, d);

        // println!("poly = {} abcd = {} {} {} {}", ZZ_POLY.to_string(&poly), a, b, c, d);
        // println!(
        //     "a_vert_re = {}, a_vert_im = {}",
        //     ZZ_POLY.to_string(&a_vert_re),
        //     ZZ_POLY.to_string(&a_vert_im)
        // );
        // println!(
        //     "b_vert_re = {}, b_vert_im = {}",
        //     ZZ_POLY.to_string(&b_vert_re),
        //     ZZ_POLY.to_string(&b_vert_im)
        // );
        // println!(
        //     "c_horz_re = {}, c_horz_im = {}",
        //     ZZ_POLY.to_string(&c_horz_re),
        //     ZZ_POLY.to_string(&c_horz_im)
        // );
        // println!(
        //     "d_horz_re = {}, d_horz_im = {}",
        //     ZZ_POLY.to_string(&d_horz_re),
        //     ZZ_POLY.to_string(&d_horz_im)
        // );

        // //checks will fail - the real and imaginary parts are only up to scalar multiples
        // debug_assert_eq!(
        //     evaluate_at_rational(&a_vert_re, c),
        //     evaluate_at_rational(&c_horz_re, a)
        // );
        // debug_assert_eq!(
        //     evaluate_at_rational(&a_vert_re, d),
        //     evaluate_at_rational(&d_horz_re, a)
        // );
        // debug_assert_eq!(
        //     evaluate_at_rational(&b_vert_re, c),
        //     evaluate_at_rational(&c_horz_re, b)
        // );
        // debug_assert_eq!(
        //     evaluate_at_rational(&b_vert_re, d),
        //     evaluate_at_rational(&d_horz_re, b)
        // );
        // debug_assert_eq!(
        //     evaluate_at_rational(&a_vert_im, c),
        //     evaluate_at_rational(&c_horz_im, a)
        // );
        // debug_assert_eq!(
        //     evaluate_at_rational(&a_vert_im, d),
        //     evaluate_at_rational(&d_horz_im, a)
        // );
        // debug_assert_eq!(
        //     evaluate_at_rational(&b_vert_im, c),
        //     evaluate_at_rational(&c_horz_im, b)
        // );
        // debug_assert_eq!(
        //     evaluate_at_rational(&b_vert_im, d),
        //     evaluate_at_rational(&d_horz_im, b)
        // );

        //compute squarefree versions for when only care about the roots without multiplicity
        let a_vert_re_sqfr = self.primitive_squarefree_part(a_vert_re.clone());
        let a_vert_im_sqfr = self.primitive_squarefree_part(a_vert_im.clone());
        let b_vert_re_sqfr = self.primitive_squarefree_part(b_vert_re.clone());
        let b_vert_im_sqfr = self.primitive_squarefree_part(b_vert_im.clone());
        let c_horz_re_sqfr = self.primitive_squarefree_part(c_horz_re.clone());
        let c_horz_im_sqfr = self.primitive_squarefree_part(c_horz_im.clone());
        let d_horz_re_sqfr = self.primitive_squarefree_part(d_horz_re.clone());
        let d_horz_im_sqfr = self.primitive_squarefree_part(d_horz_im.clone());

        //trace an anticlockwise path around the box and create a list of crossings which encode what happens to the value of the polynomial
        #[derive(Debug)]
        enum Crossing {
            PosRe,
            PosIm,
            NegRe,
            NegIm,
        }

        fn crossings<const REVERSE: bool>(
            re: &Polynomial<Integer>,
            mut re_sqfr: Polynomial<Integer>,
            im: &Polynomial<Integer>,
            mut im_sqfr: Polynomial<Integer>,
            s: &Rational,
            t: &Rational,
        ) -> Option<Vec<Crossing>> {
            // println!(
            //     "REVERSE={} re={}, re_sqfr={}, im={}, im_sqfr={}",
            //     REVERSE,
            //     ZZ_POLY.to_string(&re),
            //     ZZ_POLY.to_string(&re_sqfr),
            //     ZZ_POLY.to_string(&im),
            //     ZZ_POLY.to_string(&im_sqfr)
            // );
            debug_assert_eq!(
                ZZ_POLY.equal(re, &ZZ_POLY.zero()),
                ZZ_POLY.equal(&re_sqfr, &ZZ_POLY.zero())
            );
            debug_assert_eq!(
                ZZ_POLY.equal(im, &ZZ_POLY.zero()),
                ZZ_POLY.equal(&im_sqfr, &ZZ_POLY.zero())
            );
            //because if the real and imaginary part are both constant at 0 then poly has infinitely many complex zeros which is not possible
            debug_assert!(
                !ZZ_POLY.equal(&re_sqfr, &ZZ_POLY.zero())
                    || !ZZ_POLY.equal(&im_sqfr, &ZZ_POLY.zero())
            );
            if ZZ_POLY.equal(&re_sqfr, &ZZ_POLY.zero()) {
                //the image is doing a path confied to the imaginary axis
                let roots_im = ZZ_POLY.real_roots(im, Some(s), Some(t), REVERSE, !REVERSE);
                if roots_im.len() == 0 {
                    //the image stays once side of the real axis
                    let val = evaluate_at_rational(im, s);
                    debug_assert_eq!(val > 0, evaluate_at_rational(im, t) > 0);
                    if val > 0 {
                        Some(vec![Crossing::PosIm]) //this whole line segment is a positive imaginary crossing
                    } else {
                        Some(vec![Crossing::NegIm]) //this whole line segment is a negative imaginary crossing
                    }
                } else {
                    //the image crosses the real axis and hence passes through 0
                    None
                }
            } else if ZZ_POLY.equal(&im_sqfr, &ZZ_POLY.zero()) {
                //the image is doing a path confied to the real axis
                let roots_re = ZZ_POLY.real_roots(re, Some(s), Some(t), REVERSE, !REVERSE);
                if roots_re.len() == 0 {
                    //the image stays one side of the imaginary axis
                    let val = evaluate_at_rational(re, s);
                    debug_assert_eq!(val > 0, evaluate_at_rational(re, t) > 0);
                    if val > 0 {
                        Some(vec![Crossing::PosRe]) //this whole line segment is a positive real crossing
                    } else {
                        Some(vec![Crossing::NegRe]) //this whole line segment is a negative real crossing
                    }
                } else {
                    //the image crosses the imaginary axis and hence passes through 0
                    None
                }
            } else {
                //want to isolate roots of squarefree polynomials without factoring
                //get ordered real roots in some structure
                //get ordered imag roots in some structure
                //do a merge sort pass to interleave the real and imaginary roots in the correct order
                //    if a real root equals an imaginary root then there is a root on the boundary
                //for each root of one type, compute the sign of the other part when evaluated at the root

                let mut crossings = vec![];

                //check the value of the real and imaginary part at the vertex at the start of this path
                let v = {
                    match REVERSE {
                        false => s,
                        true => t,
                    }
                };
                match (
                    evaluate_at_rational(re, v).cmp(&Rational::from(0)),
                    evaluate_at_rational(im, v).cmp(&Rational::from(0)),
                ) {
                    (std::cmp::Ordering::Equal, std::cmp::Ordering::Equal) => {
                        //the polynomial is zero at vertex v
                        return None;
                    }
                    (std::cmp::Ordering::Equal, std::cmp::Ordering::Less) => {
                        crossings.push(Crossing::NegIm);
                    }
                    (std::cmp::Ordering::Equal, std::cmp::Ordering::Greater) => {
                        crossings.push(Crossing::PosIm);
                    }
                    (std::cmp::Ordering::Less, std::cmp::Ordering::Equal) => {
                        crossings.push(Crossing::NegRe);
                    }
                    (std::cmp::Ordering::Greater, std::cmp::Ordering::Equal) => {
                        crossings.push(Crossing::PosRe);
                    }
                    (_, _) => {}
                }

                if evaluate_at_rational(&re_sqfr, s) == Rational::from(0) {
                    re_sqfr = ZZ_POLY
                        .div(
                            re_sqfr,
                            ZZ_POLY.from_coeffs(vec![-QQ.numerator(s), QQ.denominator(s)]),
                        )
                        .unwrap();
                }

                if evaluate_at_rational(&re_sqfr, t) == Rational::from(0) {
                    re_sqfr = ZZ_POLY
                        .div(
                            re_sqfr,
                            ZZ_POLY.from_coeffs(vec![-QQ.numerator(t), QQ.denominator(t)]),
                        )
                        .unwrap();
                }

                if evaluate_at_rational(&im_sqfr, s) == Rational::from(0) {
                    im_sqfr = ZZ_POLY
                        .div(
                            im_sqfr,
                            ZZ_POLY.from_coeffs(vec![-QQ.numerator(s), QQ.denominator(s)]),
                        )
                        .unwrap();
                }
                if evaluate_at_rational(&im_sqfr, t) == Rational::from(0) {
                    im_sqfr = ZZ_POLY
                        .div(
                            im_sqfr,
                            ZZ_POLY.from_coeffs(vec![-QQ.numerator(t), QQ.denominator(t)]),
                        )
                        .unwrap();
                }
                debug_assert_ne!(evaluate_at_rational(&re_sqfr, s), Rational::from(0));
                debug_assert_ne!(evaluate_at_rational(&re_sqfr, t), Rational::from(0));
                debug_assert_ne!(evaluate_at_rational(&im_sqfr, s), Rational::from(0));
                debug_assert_ne!(evaluate_at_rational(&im_sqfr, t), Rational::from(0));

                let mut re_roots = ZZ_POLY.real_roots_squarefree(
                    re_sqfr.clone(),
                    Some(s),
                    Some(t),
                    REVERSE,
                    !REVERSE,
                );
                let mut im_roots = ZZ_POLY.real_roots_squarefree(
                    im_sqfr.clone(),
                    Some(s),
                    Some(t),
                    REVERSE,
                    !REVERSE,
                );

                // println!("re_roots = {:?}", re_roots);
                // println!("im_roots = {:?}", im_roots);

                debug_assert!(re_roots.check_invariants().is_ok());
                debug_assert!(im_roots.check_invariants().is_ok());

                match SquarefreePolyRealRoots::separate(&mut re_roots, &mut im_roots) {
                    Ok(all_roots) => {
                        //the isolating intervals for re_roots and im_roots no longer overlap
                        //we can use this to our advantage...

                        for (interleave, root_idx) in {
                            match REVERSE {
                                false => all_roots,
                                true => all_roots.into_iter().rev().collect(),
                            }
                        } {
                            // println!("interleave = {:?} root_idx = {:?}", interleave, root_idx);
                            match interleave {
                                Interleave::First => {
                                    //a real root
                                    loop {
                                        let re_root = &re_roots.intervals[root_idx];
                                        match re_root {
                                            SquarefreePolyRealRootInterval::Rational(x) => {
                                                match evaluate_at_rational(&im, x)
                                                    .cmp(&Rational::from(0))
                                                {
                                                    std::cmp::Ordering::Less => {
                                                        crossings.push(Crossing::NegIm);
                                                        break;
                                                    }
                                                    std::cmp::Ordering::Equal => panic!(),
                                                    std::cmp::Ordering::Greater => {
                                                        crossings.push(Crossing::PosIm);
                                                        break;
                                                    }
                                                }
                                            }
                                            SquarefreePolyRealRootInterval::Real(a, b, _) => {
                                                match evaluate_at_rational(&im, a)
                                                    .cmp(&Rational::from(0))
                                                {
                                                    std::cmp::Ordering::Less => {
                                                        crossings.push(Crossing::NegIm);
                                                        break;
                                                    }
                                                    std::cmp::Ordering::Equal => {
                                                        //need to refine
                                                    }
                                                    std::cmp::Ordering::Greater => {
                                                        crossings.push(Crossing::PosIm);
                                                        break;
                                                    }
                                                }
                                                match evaluate_at_rational(&im, b)
                                                    .cmp(&Rational::from(0))
                                                {
                                                    std::cmp::Ordering::Less => {
                                                        crossings.push(Crossing::NegIm);
                                                        break;
                                                    }
                                                    std::cmp::Ordering::Equal => {
                                                        //need to refine
                                                    }
                                                    std::cmp::Ordering::Greater => {
                                                        crossings.push(Crossing::PosIm);
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                        re_roots.refine(root_idx);
                                    }
                                }
                                Interleave::Second => {
                                    //an imaginary root
                                    loop {
                                        let im_root = &im_roots.intervals[root_idx];
                                        match im_root {
                                            SquarefreePolyRealRootInterval::Rational(x) => {
                                                match evaluate_at_rational(&re, x)
                                                    .cmp(&Rational::from(0))
                                                {
                                                    std::cmp::Ordering::Less => {
                                                        crossings.push(Crossing::NegRe);
                                                        break;
                                                    }
                                                    std::cmp::Ordering::Equal => panic!(),
                                                    std::cmp::Ordering::Greater => {
                                                        crossings.push(Crossing::PosRe);
                                                        break;
                                                    }
                                                }
                                            }
                                            SquarefreePolyRealRootInterval::Real(a, b, _) => {
                                                match evaluate_at_rational(&re, a)
                                                    .cmp(&Rational::from(0))
                                                {
                                                    std::cmp::Ordering::Less => {
                                                        crossings.push(Crossing::NegRe);
                                                        break;
                                                    }
                                                    std::cmp::Ordering::Equal => {
                                                        //need to refine
                                                    }
                                                    std::cmp::Ordering::Greater => {
                                                        crossings.push(Crossing::PosRe);
                                                        break;
                                                    }
                                                }
                                                match evaluate_at_rational(&re, b)
                                                    .cmp(&Rational::from(0))
                                                {
                                                    std::cmp::Ordering::Less => {
                                                        crossings.push(Crossing::NegRe);
                                                        break;
                                                    }
                                                    std::cmp::Ordering::Equal => {
                                                        //need to refine
                                                    }
                                                    std::cmp::Ordering::Greater => {
                                                        crossings.push(Crossing::PosRe);
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                        im_roots.refine(root_idx);
                                    }
                                }
                            }
                        }
                        Some(crossings)
                    }
                    Err(()) => None,
                }
            }
        }

        /*
             a           b

         d   +-----------+
             |           |
             |           |
             |           |
             |           |
             |           |
         c   +-----------+

        */

        // println!("c = {:?}", crossings::<false>(&c_horz_re, c_horz_re_sqfr.clone(), &c_horz_im, c_horz_im_sqfr.clone(), a, b));
        // println!("b = {:?}", crossings::<false>(&b_vert_re, b_vert_re_sqfr.clone(), &b_vert_im, b_vert_im_sqfr.clone(), c, d));
        // println!("d = {:?}", crossings::<true>(&d_horz_re, d_horz_re_sqfr.clone(), &d_horz_im, d_horz_im_sqfr.clone(), a, b));
        // println!("a = {:?}", crossings::<true>(&a_vert_re, a_vert_re_sqfr.clone(), &a_vert_im, a_vert_im_sqfr.clone(), c, d));

        let mut winding = vec![];
        for cr in vec![
            crossings::<false>(&c_horz_re, c_horz_re_sqfr, &c_horz_im, c_horz_im_sqfr, a, b),
            crossings::<false>(&b_vert_re, b_vert_re_sqfr, &b_vert_im, b_vert_im_sqfr, c, d),
            crossings::<true>(&d_horz_re, d_horz_re_sqfr, &d_horz_im, d_horz_im_sqfr, a, b),
            crossings::<true>(&a_vert_re, a_vert_re_sqfr, &a_vert_im, a_vert_im_sqfr, c, d),
        ] {
            match cr {
                Some(mut w) => winding.append(&mut w),
                None => {
                    return None;
                }
            }
        }

        // println!("winding = {:?}", winding);

        //compute the winding number = number of roots
        if winding.len() <= 0 {
            Some(0)
        } else {
            fn axis_pair_to_num_offset(ax1: &Crossing, ax2: &Crossing) -> isize {
                match (ax1, ax2) {
                    (Crossing::PosRe, Crossing::PosRe) => 0,
                    (Crossing::PosRe, Crossing::PosIm) => 1,
                    (Crossing::PosRe, Crossing::NegRe) => panic!(),
                    (Crossing::PosRe, Crossing::NegIm) => -1,
                    (Crossing::PosIm, Crossing::PosRe) => -1,
                    (Crossing::PosIm, Crossing::PosIm) => 0,
                    (Crossing::PosIm, Crossing::NegRe) => 1,
                    (Crossing::PosIm, Crossing::NegIm) => panic!(),
                    (Crossing::NegRe, Crossing::PosRe) => panic!(),
                    (Crossing::NegRe, Crossing::PosIm) => -1,
                    (Crossing::NegRe, Crossing::NegRe) => 0,
                    (Crossing::NegRe, Crossing::NegIm) => 1,
                    (Crossing::NegIm, Crossing::PosRe) => 1,
                    (Crossing::NegIm, Crossing::PosIm) => panic!(),
                    (Crossing::NegIm, Crossing::NegRe) => -1,
                    (Crossing::NegIm, Crossing::NegIm) => 0,
                }
            }

            let mut num: isize = 0;
            num += axis_pair_to_num_offset(&winding[winding.len() - 1], &winding[0]);
            for i in 0..winding.len() - 1 {
                num += axis_pair_to_num_offset(&winding[i], &winding[i + 1]);
            }

            if num < 0 {
                panic!("winding should always be overall anti-clockwise");
            }
            let num = num as usize;
            match num % 4 {
                0 => Some(num / 4),
                _ => panic!("invalid remainder modulo four"),
            }
        }
    }

    pub fn all_complex_roots_irreducible(
        &self,
        poly: &Polynomial<Integer>,
    ) -> Vec<ComplexAlgebraicNumber> {
        debug_assert!(!ZZ_POLY.equal(poly, &self.zero()));
        debug_assert!(ZZ_POLY.is_irreducible(poly).unwrap());
        let deg = ZZ_POLY.degree(poly).unwrap();

        let mut all_roots = vec![];
        for real_root in ZZ_POLY.all_real_roots(poly) {
            all_roots.push(ComplexAlgebraicNumber::Real(real_root));
        }
        let num_real_roots = all_roots.len();

        debug_assert!(num_real_roots <= deg);
        if num_real_roots == deg {
            return all_roots;
        }

        //search the upper half plane for the complete roots with positive imaginary part
        debug_assert_eq!((deg - num_real_roots) % 2, 0);
        let target_uhp_num = (deg - num_real_roots) / 2;

        let mut a = Rational::from(-1);
        let mut b = Rational::from(1);
        let mut c = Rational::from_signeds(1, 2);
        let mut d = Rational::from(2);

        loop {
            match self.count_complex_roots(poly, &a, &b, &c, &d) {
                Some(n) => {
                    debug_assert!(n <= target_uhp_num);
                    if n == target_uhp_num {
                        break;
                    }
                }
                None => {
                    //boundary root
                }
            }
            a *= Rational::from(2);
            b *= Rational::from(2);
            c *= Rational::from_signeds(1, 2);
            d *= Rational::from(2);
        }

        fn bisect(
            poly: &Polynomial<Integer>,
            n: usize,
            a: &Rational,
            b: &Rational,
            c: &Rational,
            d: &Rational,
        ) -> Vec<ComplexAlgebraicRoot> {
            debug_assert!(a < b);
            debug_assert!(c < d);
            debug_assert_eq!(
                ZZ_POLY.count_complex_roots(poly, &a, &b, &c, &d).unwrap(),
                n
            );
            if n == 0 {
                vec![]
            } else if n == 1 {
                vec![ComplexAlgebraicRoot {
                    poly: poly.clone(),
                    tight_a: a.clone(),
                    tight_b: b.clone(),
                    tight_c: c.clone(),
                    tight_d: d.clone(),
                }]
            } else {
                let ((n1, a1, b1, c1, d1), (n2, a2, b2, c2, d2)) = bisect_box(poly, n, a, b, c, d);

                let mut roots = bisect(poly, n1, &a1, &b1, &c1, &d1);
                roots.append(&mut bisect(poly, n2, &a2, &b2, &c2, &d2));
                return roots;
            }
        }

        for complex_root in bisect(poly, target_uhp_num, &a, &b, &c, &d) {
            all_roots.push(ComplexAlgebraicNumber::Complex(complex_root.clone().conj()));
            all_roots.push(ComplexAlgebraicNumber::Complex(complex_root));
        }

        debug_assert_eq!(all_roots.len(), deg);
        all_roots
    }

    pub fn all_complex_roots(&self, poly: &Polynomial<Integer>) -> Vec<ComplexAlgebraicNumber> {
        assert!(!ZZ_POLY.equal(poly, &self.zero()));
        let factors = self.factor(&poly).unwrap();
        let mut roots = vec![];
        for (factor, k) in factors.factors() {
            for root in self.all_complex_roots_irreducible(factor) {
                let mut i = Natural::from(0u8);
                while &i < k {
                    roots.push(root.clone());
                    i += Natural::from(1u8);
                }
            }
        }
        roots
    }
}

#[derive(Debug, Clone)]
pub struct RealAlgebraicRoot {
    poly: Polynomial<Integer>, //a primitive irreducible polynomial of degree >= 2 with a unique real root between a and b
    //an arbitrarily small interval containing the root. May be mutated
    tight_a: Rational, //tight lower bound
    tight_b: Rational, //tight upper bound
    //a heuristically large interval containing the root. Should not shrink
    wide_a: LowerBound, //wide lower bound. None means -inf
    wide_b: UpperBound, //wide upper bound. None means +inf
    //false : decreasing i.e. poly(a) > poly(b), true : increasing i.e. poly(a) < poly(b)
    dir: bool,
}

impl RealAlgebraicRoot {
    pub fn check_invariants(&self) -> Result<(), &'static str> {
        if !(self.tight_a < self.tight_b) {
            return Err("tight a should be strictly less than b");
        }
        if !(self.wide_a.clone() < self.wide_b.clone()) {
            return Err("wide a should be strictly less than b");
        }
        if !ZZ_POLY.equal(
            &self.poly,
            &ZZ_POLY
                .factor_fav_assoc(ZZ_POLY.primitive_squarefree_part(self.poly.clone()))
                .1,
        ) {
            return Err("poly should be primitive and favoriate associate");
        }
        match ZZ_POLY.is_irreducible(&self.poly) {
            Some(is_irr) => {
                if !is_irr {
                    return Err("poly should be irreducible");
                }
            }
            None => {
                return Err("poly should be non-zero");
            }
        }
        if ZZ_POLY.degree(&self.poly).unwrap() < 2 {
            return Err("poly should have degree at least 2");
        }
        let at_a = self.evaluate(&self.tight_a);
        let at_b = self.evaluate(&self.tight_b);
        assert_ne!(at_a, Rational::from(0));
        assert_ne!(at_b, Rational::from(0));
        let sign_a = &at_a > &Rational::from(0);
        let sign_b = &at_b > &Rational::from(0);
        if sign_a == sign_b {
            return Err("sign at a and b should be different");
        }
        if self.dir != (sign_a == false) {
            return Err("dir is incorrect");
        }
        Ok(())
    }

    fn new_wide_bounds(poly: Polynomial<Integer>, wide_a: Rational, wide_b: Rational) -> Self {
        let dir = QQ_POLY.evaluate(
            &ZZ_POLY.apply_map(&QQ, &poly, |x| Rational::from(x)),
            &wide_a,
        ) < Rational::from(0);
        let x = Self {
            poly,
            tight_a: wide_a.clone(),
            tight_b: wide_b.clone(),
            wide_a: LowerBound::Finite(wide_a),
            wide_b: UpperBound::Finite(wide_b),
            dir,
        };
        debug_assert!(x.check_invariants().is_ok());
        x
    }

    fn evaluate(&self, val: &Rational) -> Rational {
        evaluate_at_rational(&self.poly, val)
    }

    pub fn accuracy(&self) -> Rational {
        &self.tight_b - &self.tight_a
    }

    pub fn refine(&mut self) {
        let m = (&self.tight_a + &self.tight_b) / Rational::from(2);
        let m_sign = self.evaluate(&m) > Rational::from(0);
        match self.dir == m_sign {
            true => {
                self.tight_b = m;
            }
            false => {
                self.tight_a = m;
            }
        }
    }

    pub fn refine_to_accuracy(&mut self, accuracy: &Rational) {
        while &self.accuracy() > accuracy {
            self.refine();
        }
    }

    pub fn cmp_mut(&mut self, other: &mut Self) -> std::cmp::Ordering {
        let polys_are_eq = ZZ_POLY.equal(&self.poly, &other.poly); //polys should be irreducible primitive fav-assoc so this is valid
        loop {
            //test for equality: if the tight bounds on one are within the wide bounds of the other
            if polys_are_eq {
                if other.wide_a <= self.tight_a && self.tight_b <= other.wide_b {
                    return std::cmp::Ordering::Equal;
                }
                if self.wide_a <= other.tight_a && other.tight_b <= self.wide_b {
                    return std::cmp::Ordering::Equal;
                }
            }

            //test for inequality: if the tight bounds are disjoint
            if self.tight_b <= other.tight_a {
                return std::cmp::Ordering::Less;
            }
            if other.tight_b <= self.tight_a {
                return std::cmp::Ordering::Greater;
            }

            //refine
            self.refine();
            other.refine();
        }
    }

    pub fn cmp_rat_mut(&mut self, other: &Rational) -> std::cmp::Ordering {
        loop {
            //test for inequality: other is outside the tight bounds
            if &self.tight_b <= other {
                return std::cmp::Ordering::Less;
            }
            if other <= &self.tight_a {
                return std::cmp::Ordering::Greater;
            }

            //refine
            self.refine();
        }
    }

    pub fn neg_mut(&mut self) {
        let (unit, fav_assoc) = ZZ_POLY.factor_fav_assoc(ZZ_POLY.compose(
            &self.poly,
            &ZZ_POLY.from_coeffs(vec![Integer::from(0), Integer::from(-1)]),
        ));
        if ZZ_POLY.equal(&unit, &ZZ_POLY.one()) {
            self.poly = fav_assoc;
            self.dir = !self.dir;
        } else if ZZ_POLY.equal(&unit, &ZZ_POLY.neg(ZZ_POLY.one())) {
            self.poly = fav_assoc;
        } else {
            panic!();
        }
        (self.tight_a, self.tight_b) = (-self.tight_b.clone(), -self.tight_a.clone());
        (self.wide_a, self.wide_b) = (self.wide_b.clone().neg(), self.wide_a.clone().neg());
    }
}

impl ToString for RealAlgebraicRoot {
    fn to_string(&self) -> String {
        let m = (&self.tight_a + &self.tight_b) / Rational::from(2);

        fn rat_to_string(a: Rational) -> String {
            let neg = a < Rational::from(0);
            let (mant, exp): (f64, _) = a
                .sci_mantissa_and_exponent_with_rounding(
                    malachite_base::rounding_modes::RoundingMode::Nearest,
                )
                .unwrap();
            let mut b = (2.0 as f64).powf(exp as f64) * mant;
            if neg {
                b = -b;
            }
            b.to_string()
        }

        "≈".to_owned()
            + rat_to_string(m).as_str()
            + "±"
            + rat_to_string(self.accuracy() / Rational::from(2)).as_str()
    }
}

#[derive(Debug, Clone)]
pub struct ComplexAlgebraicRoot {
    tight_a: Rational, //tight lower bound for the real part
    tight_b: Rational, //tight upper bound for the real part
    tight_c: Rational, //tight lower bound for the imaginary part
    tight_d: Rational, //tight upper bound for the imaginary part

    poly: Polynomial<Integer>, //a primitive irreducible polynomial of degree >= 2 with a unique non-real complex root in the box defined by (a, b, c, d)
}

impl ComplexAlgebraicRoot {
    pub fn check_invariants(&self) -> Result<(), &'static str> {
        if !(self.tight_a < self.tight_b) {
            return Err("tight a should be strictly less than b");
        }
        if !(self.tight_c < self.tight_d) {
            return Err("tight c should be strictly less than d");
        }
        // if !(self.wide_a < self.wide_b) {
        //     return Err("wide a should be strictly less than b");
        // }
        // if !(self.wide_c < self.wide_d) {
        //     return Err("wide c should be strictly less than d");
        // }
        match ZZ_POLY.is_irreducible(&self.poly) {
            Some(is_irr) => {
                if !is_irr {
                    return Err("poly should be irreducible");
                }
            }
            None => {
                return Err("poly should be non-zero");
            }
        }
        if ZZ_POLY.degree(&self.poly).unwrap() < 2 {
            return Err("poly should have degree at least 2");
        }
        match ZZ_POLY.count_complex_roots(
            &self.poly,
            &self.tight_a,
            &self.tight_b,
            &self.tight_c,
            &self.tight_d,
        ) {
            Some(1) => {}
            Some(_) => {
                return Err("should contain exactly 1 root with none on the boundary");
            }
            None => {
                return Err("should contain exactly 1 root with none on the boundary");
            }
        }
        Ok(())
    }

    fn conj(mut self) -> Self {
        (self.tight_c, self.tight_d) = (-self.tight_d, -self.tight_c);
        self
    }

    pub fn neg_mut(&mut self) {
        self.poly = ZZ_POLY.fav_assoc(ZZ_POLY.compose(
            &self.poly,
            &ZZ_POLY.from_coeffs(vec![Integer::from(0), Integer::from(-1)]),
        ));
        (self.tight_a, self.tight_b) = (-self.tight_b.clone(), -self.tight_a.clone());
        (self.tight_c, self.tight_d) = (-self.tight_d.clone(), -self.tight_c.clone());
    }

    pub fn refine(&mut self) {
        let ((n1, a1, b1, c1, d1), (n2, a2, b2, c2, d2)) = bisect_box(
            &self.poly,
            1,
            &self.tight_a,
            &self.tight_b,
            &self.tight_c,
            &self.tight_d,
        );

        match (n1, n2) {
            (1, 0) => {
                self.tight_a = a1;
                self.tight_b = b1;
                self.tight_c = c1;
                self.tight_d = d1;
            }
            (0, 1) => {
                self.tight_a = a2;
                self.tight_b = b2;
                self.tight_c = c2;
                self.tight_d = d2;
            }
            _ => {
                panic!();
            }
        }

        debug_assert!(self.check_invariants().is_ok());
    }
}

#[derive(Debug, Clone)]
pub enum RealAlgebraicNumber {
    Rational(Rational),
    Real(RealAlgebraicRoot),
}

impl RealAlgebraicNumber {
    pub fn check_invariants(&self) -> Result<(), &'static str> {
        match self {
            RealAlgebraicNumber::Rational(_x) => {}
            RealAlgebraicNumber::Real(x) => match x.check_invariants() {
                Ok(()) => {}
                Err(e) => {
                    return Err(e);
                }
            },
        }
        Ok(())
    }

    pub fn cmp_mut(&mut self, other: &mut Self) -> std::cmp::Ordering {
        {
            match self {
                RealAlgebraicNumber::Rational(self_rep) => match other {
                    RealAlgebraicNumber::Rational(other_rep) => self_rep.cmp(&other_rep),
                    RealAlgebraicNumber::Real(other_rep) => {
                        other_rep.cmp_rat_mut(self_rep).reverse()
                    }
                },
                RealAlgebraicNumber::Real(self_rep) => match other {
                    RealAlgebraicNumber::Rational(other_rep) => self_rep.cmp_rat_mut(other_rep),
                    RealAlgebraicNumber::Real(other_rep) => self_rep.cmp_mut(other_rep),
                },
            }
        }
    }
}

impl PartialEq for RealAlgebraicNumber {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl Eq for RealAlgebraicNumber {}

impl PartialOrd for RealAlgebraicNumber {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.clone().cmp_mut(&mut other.clone()))
    }
}

impl Ord for RealAlgebraicNumber {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Debug, Clone)]
// #[derive(Debug, Clone, PartialEq, Eq)] //todo: this
pub enum ComplexAlgebraicNumber {
    Real(RealAlgebraicNumber),
    Complex(ComplexAlgebraicRoot),
}

impl ComplexAlgebraicNumber {
    pub fn check_invariants(&self) -> Result<(), &'static str> {
        match self {
            ComplexAlgebraicNumber::Real(x) => match x.check_invariants() {
                Ok(()) => {}
                Err(e) => {
                    return Err(e);
                }
            },
            ComplexAlgebraicNumber::Complex(x) => match x.check_invariants() {
                Ok(()) => {}
                Err(e) => {
                    return Err(e);
                }
            },
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealAlgebraicField {}

impl ComRing for RealAlgebraicField {
    type ElemT = RealAlgebraicNumber;

    fn to_string(&self, elem: &Self::ElemT) -> String {
        match elem {
            RealAlgebraicNumber::Rational(a) => a.to_string(),
            RealAlgebraicNumber::Real(a) => a.to_string(),
        }
    }

    fn equal(&self, a: &Self::ElemT, b: &Self::ElemT) -> bool {
        a == b
    }

    fn zero(&self) -> Self::ElemT {
        RealAlgebraicNumber::Rational(Rational::from(0))
    }

    fn one(&self) -> Self::ElemT {
        RealAlgebraicNumber::Rational(Rational::from(1))
    }

    fn neg_mut(&self, elem: &mut Self::ElemT) {
        match elem {
            RealAlgebraicNumber::Rational(a) => a.neg_assign(),
            RealAlgebraicNumber::Real(root) => root.neg_mut(),
        }
    }

    fn add_mut(&self, elem: &mut Self::ElemT, offset: &Self::ElemT) {
        *elem = self.add(elem.clone(), offset.clone());
    }

    fn add(&self, alg1: Self::ElemT, alg2: Self::ElemT) -> Self::ElemT {
        fn add_rat(mut elem: RealAlgebraicRoot, rat: Rational) -> RealAlgebraicRoot {
            elem.tight_a += &rat;
            elem.tight_b += &rat;
            match &elem.wide_a {
                LowerBound::Inf => {}
                LowerBound::Finite(a) => {
                    elem.wide_a = LowerBound::Finite(a + &rat);
                }
            }
            match &elem.wide_b {
                UpperBound::Inf => {}
                UpperBound::Finite(b) => {
                    elem.wide_b = UpperBound::Finite(b + &rat);
                }
            }

            //compose with x - n/d = dx - n
            elem.poly = ZZ_POLY
                .primitive_part(ZZ_POLY.compose(
                    &elem.poly,
                    &ZZ_POLY.from_coeffs(vec![-QQ.numerator(&rat), QQ.denominator(&rat)]),
                ))
                .unwrap();

            debug_assert!(elem.check_invariants().is_ok());
            elem
        }

        match (alg1, alg2) {
            (RealAlgebraicNumber::Rational(rat1), RealAlgebraicNumber::Rational(rat2)) => {
                RealAlgebraicNumber::Rational(QQ.add(rat1, rat2))
            }
            (RealAlgebraicNumber::Rational(rat1), RealAlgebraicNumber::Real(alg2)) => {
                RealAlgebraicNumber::Real(add_rat(alg2, rat1))
            }
            (RealAlgebraicNumber::Real(alg1), RealAlgebraicNumber::Rational(rat2)) => {
                RealAlgebraicNumber::Real(add_rat(alg1, rat2))
            }
            (RealAlgebraicNumber::Real(mut alg1), RealAlgebraicNumber::Real(mut alg2)) => {
                let factored_rsp = ZZ_POLY
                    .factor(&root_sum_poly(&alg1.poly, &alg2.poly))
                    .unwrap();
                let polys: Vec<_> = factored_rsp.factors().iter().map(|(f, _k)| f).collect();
                //the sum of alg1 and alg2 is exactly one root of exactly one of the irreducible polynomials in polys
                //the task now is to refine alg1 and alg2 until the root is identified

                let mut root_groups: Vec<_> = polys
                    .into_iter()
                    .map(|p| ZZ_POLY.all_real_roots_squarefree(p))
                    .collect();

                //store indicies of possible roots
                let mut possible = std::collections::HashSet::new();
                for i in 0..root_groups.len() {
                    for j in 0..root_groups[i].intervals.len() {
                        possible.insert((i, j));
                    }
                }

                while possible.len() > 1 {
                    let ans_tight_a = &alg1.tight_a + &alg2.tight_a;
                    let ans_tight_b = &alg1.tight_b + &alg2.tight_b;
                    //filter out roots which dont overlap with the known range for the sum root
                    possible = possible
                        .into_iter()
                        .filter(|(i, j)| match &root_groups[*i].intervals[*j] {
                            SquarefreePolyRealRootInterval::Rational(x) => {
                                &ans_tight_a < x && x < &ans_tight_b
                            }
                            SquarefreePolyRealRootInterval::Real(ta, tb, _dir) => {
                                ta < &ans_tight_b && &ans_tight_a < tb
                            }
                        })
                        .collect();

                    alg1.refine();
                    alg2.refine();
                    for (i, j) in &possible {
                        root_groups[*i].refine(*j);
                    }
                }
                assert_eq!(possible.len(), 1);
                let (i, j) = possible.into_iter().next().unwrap();
                root_groups
                    .into_iter()
                    .nth(i)
                    .unwrap()
                    .to_real_roots()
                    .into_iter()
                    .nth(j)
                    .unwrap()
            }
        }
    }

    fn mul_mut(&self, elem: &mut Self::ElemT, mul: &Self::ElemT) {
        *elem = self.mul(elem.clone(), mul.clone());
    }

    fn mul(&self, elem1: Self::ElemT, elem2: Self::ElemT) -> Self::ElemT {
        match elem1.cmp(&self.zero()) {
            std::cmp::Ordering::Less => {
                return self.neg(self.mul(self.neg(elem1), elem2));
            }
            std::cmp::Ordering::Equal => {
                return self.zero();
            }
            std::cmp::Ordering::Greater => {}
        }

        match elem2.cmp(&self.zero()) {
            std::cmp::Ordering::Less => {
                return self.neg(self.mul(elem1, self.neg(elem2)));
            }
            std::cmp::Ordering::Equal => {
                return self.zero();
            }
            std::cmp::Ordering::Greater => {}
        }

        debug_assert!(&elem1 > &self.zero());
        debug_assert!(&elem2 > &self.zero());

        fn mul_pos_rat(mut elem: RealAlgebraicRoot, rat: Rational) -> RealAlgebraicRoot {
            debug_assert!(rat > Rational::from(0));
            elem.tight_a *= &rat;
            elem.tight_b *= &rat;
            match &elem.wide_a {
                LowerBound::Inf => {}
                LowerBound::Finite(a) => {
                    elem.wide_a = LowerBound::Finite(a * &rat);
                }
            }
            match &elem.wide_b {
                UpperBound::Inf => {}
                UpperBound::Finite(b) => {
                    elem.wide_b = UpperBound::Finite(b * &rat);
                }
            }
            //we are multiplying by a so need to replace f(x) with f(x/a)
            //e.g. f(x) = x-1 and multiply root by 3 then replace f(x) with
            //f(x/3) = 3/x-1 = x-3
            //e.g. f(x) = 1 + x + x^2 replace it with f(d/n * x) = 1 + d/n x + d^2/n^2 x^2 = n^2 + ndx + d^2 x
            elem.poly = ZZ_POLY.from_coeffs({
                let degree = ZZ_POLY.degree(&elem.poly).unwrap();
                let (n, d) = (QQ.numerator(&rat), QQ.denominator(&rat));
                let mut n_pows = vec![Integer::from(1)];
                let mut d_pows = vec![Integer::from(1)];

                {
                    let n_pow = n;
                    let d_pow = d;
                    for _i in 0..degree {
                        n_pows.push(n_pow.clone());
                        d_pows.push(d_pow.clone());
                    }
                }

                debug_assert_eq!(n_pows.len(), degree + 1);
                debug_assert_eq!(d_pows.len(), degree + 1);

                let coeffs = elem
                    .poly
                    .coeffs()
                    .iter()
                    .enumerate()
                    .map(|(i, c)| &d_pows[i] * &n_pows[degree - i] * c)
                    .collect();
                coeffs
            });
            debug_assert!(elem.check_invariants().is_ok());
            elem
        }

        match (elem1, elem2) {
            (RealAlgebraicNumber::Rational(rat1), RealAlgebraicNumber::Rational(rat2)) => {
                RealAlgebraicNumber::Rational(QQ.mul(rat1, rat2))
            }
            (RealAlgebraicNumber::Rational(rat1), RealAlgebraicNumber::Real(alg2)) => {
                RealAlgebraicNumber::Real(mul_pos_rat(alg2, rat1))
            }
            (RealAlgebraicNumber::Real(alg1), RealAlgebraicNumber::Rational(rat2)) => {
                RealAlgebraicNumber::Real(mul_pos_rat(alg1, rat2))
            }
            (RealAlgebraicNumber::Real(mut alg1), RealAlgebraicNumber::Real(mut alg2)) => {
                let factored_rsp = ZZ_POLY
                    .factor(&root_prod_poly(&alg1.poly, &alg2.poly))
                    .unwrap();
                let polys: Vec<_> = factored_rsp.factors().iter().map(|(f, _k)| f).collect();
                //the sum of alg1 and alg2 is exactly one root of exactly one of the irreducible polynomials in polys
                //the task now is to refine alg1 and alg2 until the root is identified

                let mut root_groups: Vec<_> = polys
                    .into_iter()
                    .map(|p| ZZ_POLY.all_real_roots_squarefree(p))
                    .collect();

                //store indicies of possible roots
                let mut possible = std::collections::HashSet::new();
                for i in 0..root_groups.len() {
                    for j in 0..root_groups[i].intervals.len() {
                        possible.insert((i, j));
                    }
                }

                while possible.len() > 1 {
                    let ans_tight_a = &alg1.tight_a * &alg2.tight_a;
                    let ans_tight_b = &alg1.tight_b * &alg2.tight_b;
                    //filter out roots which dont overlap with the known range for the sum root
                    possible = possible
                        .into_iter()
                        .filter(|(i, j)| match &root_groups[*i].intervals[*j] {
                            SquarefreePolyRealRootInterval::Rational(x) => {
                                &ans_tight_a < x && x < &ans_tight_b
                            }
                            SquarefreePolyRealRootInterval::Real(ta, tb, _dir) => {
                                ta < &ans_tight_b && &ans_tight_a < tb
                            }
                        })
                        .collect();

                    alg1.refine();
                    alg2.refine();
                    for (i, j) in &possible {
                        root_groups[*i].refine(*j);
                    }
                }
                assert_eq!(possible.len(), 1);
                let (i, j) = possible.into_iter().next().unwrap();
                root_groups
                    .into_iter()
                    .nth(i)
                    .unwrap()
                    .to_real_roots()
                    .into_iter()
                    .nth(j)
                    .unwrap()
            }
        }
    }

    fn div(&self, a: Self::ElemT, b: Self::ElemT) -> Result<Self::ElemT, RingDivisionError> {
        match self.inv(b) {
            Ok(b_inv) => Ok(self.mul(a, b_inv)),
            Err(err) => Err(err),
        }
    }

    fn inv(&self, mut elem: Self::ElemT) -> Result<Self::ElemT, RingDivisionError> {
        match elem.cmp_mut(&mut self.zero()) {
            std::cmp::Ordering::Less => match self.inv(self.neg(elem)) {
                Ok(neg_elem_inv) => Ok(self.neg(neg_elem_inv)),
                Err(err) => Err(err),
            },
            std::cmp::Ordering::Equal => Err(RingDivisionError::DivideByZero),
            std::cmp::Ordering::Greater => match elem {
                RealAlgebraicNumber::Rational(x) => {
                    Ok(RealAlgebraicNumber::Rational(QQ.inv(x).unwrap()))
                }
                RealAlgebraicNumber::Real(mut root) => {
                    debug_assert!(root.tight_a >= Rational::from(0));
                    while root.tight_a == Rational::from(0) {
                        root.refine();
                    }
                    debug_assert!(Rational::from(0) < root.tight_a);
                    (root.tight_a, root.tight_b) =
                        (QQ.inv(root.tight_b).unwrap(), QQ.inv(root.tight_a).unwrap());
                    (root.wide_a, root.wide_b) = (
                        {
                            match root.wide_b {
                                UpperBound::Inf => LowerBound::Finite(Rational::from(0)),
                                UpperBound::Finite(x) => match QQ.inv(x) {
                                    Ok(x_inv) => LowerBound::Finite(x_inv),
                                    Err(RingDivisionError::DivideByZero) => panic!("wide upper bound of strictly positive root should be strictly positive i.e. non-zero"),
                                    Err(_) => panic!()
                                },
                            }
                        },
                        {
                            match root.wide_a {
                                LowerBound::Inf => UpperBound::Inf,
                                LowerBound::Finite(x) => match x.cmp(&Rational::from(0)) {
                                    std::cmp::Ordering::Less => UpperBound::Inf,
                                    std::cmp::Ordering::Equal => UpperBound::Inf,
                                    std::cmp::Ordering::Greater => {
                                        UpperBound::Finite(QQ.inv(x).unwrap())
                                    }
                                },
                            }
                        },
                    );
                    let (unit, fav_assoc) = ZZ_POLY.factor_fav_assoc(
                        ZZ_POLY.from_coeffs(root.poly.coeffs().into_iter().rev().collect()),
                    );
                    if ZZ_POLY.equal(&unit, &ZZ_POLY.one()) {
                        root.poly = fav_assoc;
                        root.dir = !root.dir;
                    } else if ZZ_POLY.equal(&unit, &ZZ_POLY.neg(ZZ_POLY.one())) {
                        root.poly = fav_assoc;
                    } else {
                        panic!();
                    }
                    let ans = RealAlgebraicNumber::Real(root);
                    debug_assert!(ans.check_invariants().is_ok());
                    Ok(ans)
                }
            },
        }
    }
}

impl IntegralDomain for RealAlgebraicField {}

impl Field for RealAlgebraicField {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComplexAlgebraicField {}

impl ComRing for ComplexAlgebraicField {
    type ElemT = ComplexAlgebraicNumber;

    fn to_string(&self, elem: &Self::ElemT) -> String {
        match elem {
            ComplexAlgebraicNumber::Real(a) => RealAlgebraicField {}.to_string(a),
            ComplexAlgebraicNumber::Complex(_a) => todo!(),
        }
    }

    fn equal(&self, _a: &Self::ElemT, _b: &Self::ElemT) -> bool {
        todo!()
        // a == b //impl eq for complex roots
    }

    fn zero(&self) -> Self::ElemT {
        ComplexAlgebraicNumber::Real(RealAlgebraicNumber::Rational(Rational::from(0)))
    }

    fn one(&self) -> Self::ElemT {
        ComplexAlgebraicNumber::Real(RealAlgebraicNumber::Rational(Rational::from(1)))
    }

    fn neg_mut(&self, elem: &mut Self::ElemT) {
        match elem {
            ComplexAlgebraicNumber::Real(root) => QQ_BAR_REAL.neg_mut(root),
            ComplexAlgebraicNumber::Complex(root) => {
                root.neg_mut();
            }
        }
    }

    fn add_mut(&self, elem: &mut Self::ElemT, offset: &Self::ElemT) {
        *elem = self.add(elem.clone(), offset.clone());
    }

    fn add(&self, alg1: Self::ElemT, alg2: Self::ElemT) -> Self::ElemT {
        fn add_real(
            mut cpx: ComplexAlgebraicRoot,
            real: RealAlgebraicNumber,
        ) -> ComplexAlgebraicRoot {
            match real {
                RealAlgebraicNumber::Rational(rat) => {
                    cpx.tight_a += &rat;
                    cpx.tight_b += &rat;
                    //compose with x - n/d = dx - n
                    cpx.poly = ZZ_POLY
                        .primitive_part(ZZ_POLY.compose(
                            &cpx.poly,
                            &ZZ_POLY.from_coeffs(vec![-QQ.numerator(&rat), QQ.denominator(&rat)]),
                        ))
                        .unwrap();

                    debug_assert!(cpx.check_invariants().is_ok());
                    cpx
                }
                RealAlgebraicNumber::Real(mut real) => {
                    let mut roots: Vec<_> = ZZ_POLY
                        .all_complex_roots(
                            &ZZ_POLY
                                .primitive_squarefree_part(root_sum_poly(&cpx.poly, &real.poly)),
                        )
                        .into_iter()
                        .filter(|alg| match alg {
                            ComplexAlgebraicNumber::Real(_) => false,
                            ComplexAlgebraicNumber::Complex(_) => true,
                        })
                        .map(|alg| match alg {
                            ComplexAlgebraicNumber::Real(_) => panic!(),
                            ComplexAlgebraicNumber::Complex(cpx) => cpx,
                        })
                        .collect();

                    let mut possible: std::collections::HashSet<_> = (0..roots.len()).collect();

                    while possible.len() > 1 {
                        let ans_tight_a = &cpx.tight_a + &real.tight_a;
                        let ans_tight_b = &cpx.tight_b + &real.tight_b;
                        let ans_tight_c = cpx.tight_c.clone();
                        let ans_tight_d = cpx.tight_d.clone();
                        //filter out roots which dont overlap with the known range for the sum root

                        possible = possible
                            .into_iter()
                            .filter(|i| {
                                let possible_cpx_root = &roots[*i];
                                &possible_cpx_root.tight_a < &ans_tight_b
                                    && &ans_tight_a < &possible_cpx_root.tight_b
                                    && &possible_cpx_root.tight_c < &ans_tight_d
                                    && &ans_tight_c < &possible_cpx_root.tight_d
                            })
                            .collect();

                        cpx.refine();
                        real.refine();
                        for i in &possible {
                            roots[*i].refine();
                        }
                    }
                    assert_eq!(possible.len(), 1);
                    let i = possible.into_iter().next().unwrap();
                    roots.into_iter().nth(i).unwrap()
                }
            }
        }

        match (alg1, alg2) {
            (ComplexAlgebraicNumber::Real(real1), ComplexAlgebraicNumber::Real(real2)) => {
                ComplexAlgebraicNumber::Real(QQ_BAR_REAL.add(real1, real2))
            }
            (ComplexAlgebraicNumber::Real(real1), ComplexAlgebraicNumber::Complex(cpx2)) => {
                ComplexAlgebraicNumber::Complex(add_real(cpx2, real1))
            }
            (ComplexAlgebraicNumber::Complex(cpx1), ComplexAlgebraicNumber::Real(real2)) => {
                ComplexAlgebraicNumber::Complex(add_real(cpx1, real2))
            }
            (
                ComplexAlgebraicNumber::Complex(mut cpx1),
                ComplexAlgebraicNumber::Complex(mut cpx2),
            ) => {
                let mut roots = ZZ_POLY.all_complex_roots(&root_sum_poly(&cpx1.poly, &cpx2.poly));
                let mut possible: std::collections::HashSet<_> = (0..roots.len()).collect();

                while possible.len() > 1 {
                    let ans_tight_a = &cpx1.tight_a + &cpx2.tight_a;
                    let ans_tight_b = &cpx1.tight_b + &cpx2.tight_b;
                    let ans_tight_c = &cpx1.tight_c + &cpx2.tight_c;
                    let ans_tight_d = &cpx1.tight_d + &cpx2.tight_d;
                    //filter out roots which dont overlap with the known range for the sum root

                    possible = possible
                        .into_iter()
                        .filter(|i| match &roots[*i] {
                            ComplexAlgebraicNumber::Real(real) => match real {
                                RealAlgebraicNumber::Rational(rat) => {
                                    &ans_tight_a < rat
                                        && rat < &ans_tight_b
                                        && &ans_tight_c < &Rational::from(0)
                                        && &Rational::from(0) < &ans_tight_d
                                }
                                RealAlgebraicNumber::Real(real) => {
                                    &real.tight_a < &ans_tight_b
                                        && &ans_tight_a < &real.tight_b
                                        && &ans_tight_c < &Rational::from(0)
                                        && &Rational::from(0) < &ans_tight_d
                                }
                            },
                            ComplexAlgebraicNumber::Complex(cpx) => {
                                &cpx.tight_a < &ans_tight_b
                                    && &ans_tight_a < &cpx.tight_b
                                    && &cpx.tight_c < &ans_tight_d
                                    && &ans_tight_c < &cpx.tight_d
                            }
                        })
                        .collect();

                    cpx1.refine();
                    cpx2.refine();
                    for i in &possible {
                        match &mut roots[*i] {
                            ComplexAlgebraicNumber::Real(real) => match real {
                                RealAlgebraicNumber::Rational(_rat) => {}
                                RealAlgebraicNumber::Real(real) => real.refine(),
                            },
                            ComplexAlgebraicNumber::Complex(cpx) => {
                                cpx.refine();
                            }
                        }
                    }
                }
                assert_eq!(possible.len(), 1);
                let i = possible.into_iter().next().unwrap();
                roots.into_iter().nth(i).unwrap()
            }
        }
    }

    fn mul_mut(&self, _elem: &mut Self::ElemT, _mul: &Self::ElemT) {
        todo!()
    }

    fn div(&self, _a: Self::ElemT, _b: Self::ElemT) -> Result<Self::ElemT, RingDivisionError> {
        todo!()
    }
}

impl IntegralDomain for ComplexAlgebraicField {}

impl Field for ComplexAlgebraicField {}

#[cfg(test)]
mod tests {
    use super::super::poly::*;
    use super::*;

    #[test]
    fn test_root_sum_poly() {
        for (f, g, exp) in vec![
            (
                ZZ_POLY.from_coeffs(vec![Integer::from(0)]),
                ZZ_POLY.from_coeffs(vec![Integer::from(0)]),
                ZZ_POLY.from_coeffs(vec![Integer::from(0)]),
            ),
            (
                ZZ_POLY.from_coeffs(vec![Integer::from(0)]),
                ZZ_POLY.from_coeffs(vec![Integer::from(1)]),
                ZZ_POLY.from_coeffs(vec![Integer::from(1)]),
            ),
            (
                ZZ_POLY.from_coeffs(vec![Integer::from(1)]),
                ZZ_POLY.from_coeffs(vec![Integer::from(1)]),
                ZZ_POLY.from_coeffs(vec![Integer::from(1)]),
            ),
            (
                ZZ_POLY.from_coeffs(vec![Integer::from(-3), Integer::from(1)]),
                ZZ_POLY.from_coeffs(vec![Integer::from(-5), Integer::from(1)]),
                ZZ_POLY.from_coeffs(vec![Integer::from(-8), Integer::from(1)]),
            ),
            (
                ZZ_POLY.from_coeffs(vec![Integer::from(1)]),
                ZZ_POLY.from_coeffs(vec![Integer::from(-7), Integer::from(1)]),
                ZZ_POLY.from_coeffs(vec![Integer::from(1)]),
            ),
            (
                ZZ_POLY.from_coeffs(vec![Integer::from(-1), Integer::from(2)]),
                ZZ_POLY.from_coeffs(vec![Integer::from(-1), Integer::from(3)]),
                ZZ_POLY.from_coeffs(vec![Integer::from(-5), Integer::from(6)]),
            ),
            (
                ZZ_POLY.from_coeffs(vec![Integer::from(-1), Integer::from(-2), Integer::from(1)]),
                ZZ_POLY.from_coeffs(vec![Integer::from(-2), Integer::from(0), Integer::from(1)]),
                ZZ_POLY.from_coeffs(vec![
                    Integer::from(-7),
                    Integer::from(5),
                    Integer::from(3),
                    Integer::from(-1),
                ]),
            ),
        ] {
            println!();
            let rsp = root_sum_poly(&f, &g);
            println!("f = {}", ZZ_POLY.to_string(&f));
            println!("g = {}", ZZ_POLY.to_string(&g));
            println!(
                "exp = {}    exp_factored = {:?}",
                ZZ_POLY.to_string(&exp),
                ZZ_POLY.factorize_by_kroneckers_method(&exp)
            );
            println!(
                "rsp = {}    rsp_factored = {:?}",
                ZZ_POLY.to_string(&rsp),
                ZZ_POLY.factorize_by_kroneckers_method(&rsp)
            );
            assert!(ZZ_POLY.are_associate(&exp, &rsp));
        }
    }

    #[test]
    fn test_root_prod_poly() {
        for (f, g, exp) in vec![
            (
                ZZ_POLY.from_coeffs(vec![Integer::from(0)]),
                ZZ_POLY.from_coeffs(vec![Integer::from(0)]),
                ZZ_POLY.from_coeffs(vec![Integer::from(0)]),
            ),
            (
                ZZ_POLY.from_coeffs(vec![Integer::from(0)]),
                ZZ_POLY.from_coeffs(vec![Integer::from(1)]),
                ZZ_POLY.from_coeffs(vec![Integer::from(1)]),
            ),
            (
                ZZ_POLY.from_coeffs(vec![Integer::from(1)]),
                ZZ_POLY.from_coeffs(vec![Integer::from(1)]),
                ZZ_POLY.from_coeffs(vec![Integer::from(1)]),
            ),
            (
                ZZ_POLY.from_coeffs(vec![Integer::from(-3), Integer::from(1)]),
                ZZ_POLY.from_coeffs(vec![Integer::from(-5), Integer::from(1)]),
                ZZ_POLY.from_coeffs(vec![Integer::from(-15), Integer::from(1)]),
            ),
            (
                ZZ_POLY.from_coeffs(vec![Integer::from(1)]),
                ZZ_POLY.from_coeffs(vec![Integer::from(-7), Integer::from(1)]),
                ZZ_POLY.from_coeffs(vec![Integer::from(1)]),
            ),
            (
                ZZ_POLY.from_coeffs(vec![Integer::from(-1), Integer::from(2)]),
                ZZ_POLY.from_coeffs(vec![Integer::from(-1), Integer::from(3)]),
                ZZ_POLY.from_coeffs(vec![Integer::from(-1), Integer::from(6)]),
            ),
            (
                ZZ_POLY.from_coeffs(vec![Integer::from(-1), Integer::from(-2), Integer::from(1)]),
                ZZ_POLY.from_coeffs(vec![Integer::from(-2), Integer::from(0), Integer::from(1)]),
                ZZ_POLY.from_coeffs(vec![
                    Integer::from(4),
                    Integer::from(0),
                    Integer::from(-12),
                    Integer::from(0),
                    Integer::from(1),
                ]),
            ),
            (
                ZZ_POLY.from_coeffs(vec![Integer::from(-2), Integer::from(0), Integer::from(1)]),
                ZZ_POLY.from_coeffs(vec![Integer::from(-2), Integer::from(0), Integer::from(1)]),
                ZZ_POLY.from_coeffs(vec![Integer::from(-4), Integer::from(0), Integer::from(1)]),
            ),
        ] {
            println!();
            let rpp = root_prod_poly(&f, &g);
            println!("f = {}", ZZ_POLY.to_string(&f));
            println!("g = {}", ZZ_POLY.to_string(&g));
            println!(
                "exp = {}    exp_factored = {:?}",
                ZZ_POLY.to_string(&exp),
                ZZ_POLY.factorize_by_kroneckers_method(&exp)
            );
            println!(
                "rpp = {}    rpp_factored = {:?}",
                ZZ_POLY.to_string(&rpp),
                ZZ_POLY.factorize_by_kroneckers_method(&rpp)
            );
            assert!(ZZ_POLY.are_associate(&exp, &rpp));
        }
    }

    #[test]
    fn test_squarefree_polynomial_real_root_isolation() {
        let f = ZZ_POLY.product(vec![
            &ZZ_POLY.from_coeffs(vec![
                Integer::from(-2),
                Integer::from(-4),
                Integer::from(-2),
            ]),
            &ZZ_POLY.from_coeffs(vec![Integer::from(6), Integer::from(0), Integer::from(-3)]),
            &ZZ_POLY.from_coeffs(vec![Integer::from(1), Integer::from(-3), Integer::from(1)]),
            &ZZ_POLY.from_coeffs(vec![
                Integer::from(2),
                Integer::from(-3),
                Integer::from(0),
                Integer::from(0),
                Integer::from(0),
                Integer::from(1),
            ]),
            &ZZ_POLY.from_coeffs(vec![
                Integer::from(1),
                Integer::from(-3),
                Integer::from(0),
                Integer::from(0),
                Integer::from(0),
                Integer::from(1),
            ]),
            &ZZ_POLY.from_coeffs(vec![
                Integer::from(-1),
                Integer::from(12),
                Integer::from(-4),
                Integer::from(-15),
                Integer::from(5),
                Integer::from(3),
                Integer::from(-1),
            ]),
        ]);
        let f = ZZ_POLY.primitive_squarefree_part(f);
        //f is a squarefree polynomial with lots of roots
        println!("f = {:?}", f);
        let intervals = ZZ_POLY.real_roots_squarefree(f, None, None, false, false);
        println!("intervals = {:?}", &intervals);
        intervals.check_invariants().unwrap();

        let f = ZZ_POLY.from_coeffs(vec![Integer::from(1), Integer::from(-3), Integer::from(1)]);
        println!("f = {:?}", f);
        let mut intervals = ZZ_POLY.real_roots_squarefree(f, None, None, false, false);
        intervals.check_invariants().unwrap();
        intervals.clone().to_real_roots();
        for root in intervals.clone().to_real_roots() {
            println!("root = {:?}", root);
            root.check_invariants().unwrap();
        }
        println!("refine");
        for _i in 0..10 {
            intervals.refine_all();
        }
        for root in intervals.clone().to_real_roots() {
            println!("root = {:?}", root);
            root.check_invariants().unwrap();
        }

        let f = ZZ_POLY.product(vec![
            &ZZ_POLY.from_coeffs(vec![Integer::from(-1), Integer::from(1)]),
            &ZZ_POLY.from_coeffs(vec![Integer::from(-2), Integer::from(1)]),
            &ZZ_POLY.from_coeffs(vec![Integer::from(-3), Integer::from(1)]),
            &ZZ_POLY.from_coeffs(vec![Integer::from(-4), Integer::from(1)]),
        ]);
        assert_eq!(
            ZZ_POLY
                .real_roots_squarefree(
                    f.clone(),
                    Some(&Rational::from(1)),
                    Some(&Rational::from(4)),
                    false,
                    false
                )
                .intervals
                .len(),
            2
        );
        assert_eq!(
            ZZ_POLY
                .real_roots_squarefree(
                    f.clone(),
                    Some(&Rational::from(1)),
                    Some(&Rational::from(4)),
                    true,
                    false
                )
                .intervals
                .len(),
            3
        );
        assert_eq!(
            ZZ_POLY
                .real_roots_squarefree(
                    f.clone(),
                    Some(&Rational::from(1)),
                    Some(&Rational::from(4)),
                    false,
                    true
                )
                .intervals
                .len(),
            3
        );
        assert_eq!(
            ZZ_POLY
                .real_roots_squarefree(
                    f.clone(),
                    Some(&Rational::from(1)),
                    Some(&Rational::from(4)),
                    true,
                    true
                )
                .intervals
                .len(),
            4
        );
    }

    #[test]
    fn test_real_root_irreducible_count() {
        assert_eq!(
            ZZ_POLY
                .real_roots_irreducible(
                    &ZZ_POLY.from_coeffs(vec![
                        Integer::from(3),
                        Integer::from(-3),
                        Integer::from(0),
                        Integer::from(0),
                        Integer::from(0),
                        Integer::from(1)
                    ]),
                    None,
                    None,
                    false,
                    false
                )
                .len(),
            1
        );
        assert_eq!(
            ZZ_POLY
                .real_roots_irreducible(
                    &ZZ_POLY.from_coeffs(vec![
                        Integer::from(1),
                        Integer::from(-3),
                        Integer::from(0),
                        Integer::from(0),
                        Integer::from(0),
                        Integer::from(1)
                    ]),
                    None,
                    None,
                    false,
                    false
                )
                .len(),
            3
        );
    }

    #[test]
    fn test_real_algebraic_ordering() {
        let mut all_roots = vec![];
        for f in vec![
            ZZ_POLY.from_coeffs(vec![
                Integer::from(-2),
                Integer::from(-4),
                Integer::from(-2),
            ]),
            ZZ_POLY.from_coeffs(vec![Integer::from(6), Integer::from(0), Integer::from(-3)]),
            ZZ_POLY.from_coeffs(vec![Integer::from(1), Integer::from(-3), Integer::from(1)]),
            ZZ_POLY.from_coeffs(vec![
                Integer::from(2),
                Integer::from(-3),
                Integer::from(0),
                Integer::from(0),
                Integer::from(0),
                Integer::from(1),
            ]),
            ZZ_POLY.from_coeffs(vec![
                Integer::from(1),
                Integer::from(-3),
                Integer::from(0),
                Integer::from(0),
                Integer::from(0),
                Integer::from(1),
            ]),
            ZZ_POLY.from_coeffs(vec![
                Integer::from(-1),
                Integer::from(12),
                Integer::from(-4),
                Integer::from(-15),
                Integer::from(5),
                Integer::from(3),
                Integer::from(-1),
            ]),
        ] {
            for root in ZZ_POLY.real_roots(&f, None, None, false, false) {
                all_roots.push(root.clone());
            }
        }

        all_roots.sort();

        for mut root in &mut all_roots {
            root.check_invariants().unwrap();
            match &mut root {
                RealAlgebraicNumber::Rational(_a) => {}
                RealAlgebraicNumber::Real(a) => {
                    a.refine_to_accuracy(&Rational::from_signeds(1, i64::MAX))
                }
            }
            println!("    {} {:?}", QQ_BAR_REAL.to_string(&root), root);
        }

        let mut all_roots_sorted_by_lower_tight_bound = all_roots.clone();
        all_roots_sorted_by_lower_tight_bound.sort_by_key(|root| match root {
            RealAlgebraicNumber::Rational(a) => a.clone(),
            RealAlgebraicNumber::Real(r) => r.tight_a.clone(),
        });
        assert_eq!(all_roots, all_roots_sorted_by_lower_tight_bound);
    }

    #[test]
    fn test_at_fixed_re_and_im() {
        let f = ZZ_POLY.from_coeffs(vec![
            Integer::from(-1),
            Integer::from(3),
            Integer::from(0),
            Integer::from(1),
        ]);

        println!("f = {}", ZZ_POLY.to_string(&f));

        let (vert_re_f, vert_im_f) = ZZ_POLY.at_fixed_re(&f, &Rational::from(2));
        println!("re = {}", ZZ_POLY.to_string(&vert_re_f));
        println!("im = {}", ZZ_POLY.to_string(&vert_im_f));
        // f(z) = z^3 + 3z - 1
        // f(2 + xi) = (2 + xi)^3 + 3(2 + xi) - 1
        //           = 8 + 12xi - 6x^2 - x^3i + 6 + 3xi - 1
        //           = 13 + 15ix - 6x^2 - ix^3
        debug_assert!(ZZ_POLY.equal(
            &vert_re_f,
            &ZZ_POLY.from_coeffs(vec![Integer::from(13), Integer::from(0), Integer::from(-6)])
        ));
        debug_assert!(ZZ_POLY.equal(
            &vert_im_f,
            &ZZ_POLY.from_coeffs(vec![
                Integer::from(0),
                Integer::from(15),
                Integer::from(0),
                Integer::from(-1)
            ])
        ));

        let (vert_re_f, vert_im_f) = ZZ_POLY.at_fixed_re(&f, &Rational::from_signeds(1, 2));
        println!("re = {}", ZZ_POLY.to_string(&vert_re_f));
        println!("im = {}", ZZ_POLY.to_string(&vert_im_f));
        // f(z) = z^3 + 3z - 1
        // f(1/2 + xi) = 5 + 30ix - 12x^2 - 8ix^3
        debug_assert!(ZZ_POLY.equal(
            &vert_re_f,
            &ZZ_POLY.from_coeffs(vec![Integer::from(5), Integer::from(0), Integer::from(-12)])
        ));
        debug_assert!(ZZ_POLY.equal(
            &vert_im_f,
            &ZZ_POLY.from_coeffs(vec![
                Integer::from(0),
                Integer::from(30),
                Integer::from(0),
                Integer::from(-8)
            ])
        ));

        let (vert_re_f, vert_im_f) = ZZ_POLY.at_fixed_im(&f, &Rational::from(2));
        println!("re = {}", ZZ_POLY.to_string(&vert_re_f));
        println!("im = {}", ZZ_POLY.to_string(&vert_im_f));
        // f(z) = z^3 + 3z - 1
        // f(x + 2i) = -1 -2i -9x + 6ix^2 + x^3
        debug_assert!(ZZ_POLY.equal(
            &vert_re_f,
            &ZZ_POLY.from_coeffs(vec![
                Integer::from(-1),
                Integer::from(-9),
                Integer::from(0),
                Integer::from(1)
            ])
        ));
        debug_assert!(ZZ_POLY.equal(
            &vert_im_f,
            &ZZ_POLY.from_coeffs(vec![Integer::from(-2), Integer::from(0), Integer::from(6),])
        ));

        let (vert_re_f, vert_im_f) = ZZ_POLY.at_fixed_im(&f, &Rational::from_signeds(1, 2));
        println!("re = {}", ZZ_POLY.to_string(&vert_re_f));
        println!("im = {}", ZZ_POLY.to_string(&vert_im_f));
        // f(z) = z^3 + 3z - 1
        // f(x + 1/2i) = -8 +11i + 18x + 12ix^2 + 8x^3
        debug_assert!(ZZ_POLY.equal(
            &vert_re_f,
            &ZZ_POLY.from_coeffs(vec![
                Integer::from(-8),
                Integer::from(18),
                Integer::from(0),
                Integer::from(8)
            ])
        ));
        debug_assert!(ZZ_POLY.equal(
            &vert_im_f,
            &ZZ_POLY.from_coeffs(vec![Integer::from(11), Integer::from(0), Integer::from(12)])
        ));
    }

    #[test]
    fn test_count_complex_roots() {
        //cyclotomic polynomials in a box of sidelength 4
        for k in 1..19 {
            let f = ZZ_POLY.add(ZZ_POLY.var_pow(k), ZZ_POLY.neg(ZZ_POLY.one()));
            let n = ZZ_POLY
                .count_complex_roots(
                    &f,
                    &Rational::from(-2),
                    &Rational::from(2),
                    &Rational::from(-2),
                    &Rational::from(2),
                )
                .unwrap();
            assert_eq!(n, k);
        }

        //cyclotomic polynomials in a box with a boundary root iff k=0 mod 2
        for k in 1..19 {
            let f = ZZ_POLY.add(ZZ_POLY.var_pow(k), ZZ_POLY.neg(ZZ_POLY.one()));
            let n = ZZ_POLY.count_complex_roots(
                &f,
                &Rational::from(-1),
                &Rational::from(3),
                &Rational::from(-3),
                &Rational::from(3),
            );
            if k % 2 == 0 {
                assert!(n.is_none());
            } else {
                assert_eq!(n.unwrap(), k);
            }
        }

        //cyclotomic polynomials in a box with a boundary root iff k=0 mod 4
        for k in 1..19 {
            let f = ZZ_POLY.add(ZZ_POLY.var_pow(k), ZZ_POLY.neg(ZZ_POLY.one()));
            let n = ZZ_POLY.count_complex_roots(
                &f,
                &Rational::from(-2),
                &Rational::from(2),
                &Rational::from(-1),
                &Rational::from(1),
            );
            if k % 4 == 0 {
                assert!(n.is_none());
            } else {
                assert_eq!(n.unwrap(), k);
            }
        }

        //other test cases
        assert_eq!(
            Some(1),
            ZZ_POLY.count_complex_roots(
                &ZZ_POLY.from_coeffs(vec![
                    Integer::from(2),
                    Integer::from(-8),
                    Integer::from(1),
                    Integer::from(-4),
                    Integer::from(0),
                    Integer::from(1),
                ]),
                &Rational::from(-1),
                &Rational::from(1),
                &Rational::from(1),
                &Rational::from(2),
            )
        );

        assert_eq!(
            Some(3),
            ZZ_POLY.count_complex_roots(
                &ZZ_POLY.from_coeffs(vec![
                    Integer::from(2),
                    Integer::from(-8),
                    Integer::from(1),
                    Integer::from(-4),
                    Integer::from(0),
                    Integer::from(1),
                ]),
                &Rational::from(-3),
                &Rational::from(3),
                &Rational::from(-1),
                &Rational::from(1),
            )
        );

        //polynomial with roots 2+3i, 2-3i and counting box with 2+3i as a vertex
        assert_eq!(
            None,
            ZZ_POLY.count_complex_roots(
                &ZZ_POLY.from_coeffs(vec![Integer::from(13), Integer::from(-4), Integer::from(1),]),
                &Rational::from(2),
                &Rational::from(3),
                &Rational::from(3),
                &Rational::from(4),
            )
        );

        //x^2-x+1
        let f = ZZ_POLY.from_coeffs(vec![Integer::from(1), Integer::from(-1), Integer::from(1)]);
        let n = ZZ_POLY
            .count_complex_roots(
                &f,
                &Rational::from(-1),
                &Rational::from(1),
                &Rational::from(-1),
                &Rational::from(1),
            )
            .unwrap();
        assert_eq!(n, 2);
    }

    #[test]
    fn test_real_neg() {
        {
            let f =
                ZZ_POLY.from_coeffs(vec![Integer::from(-2), Integer::from(0), Integer::from(1)]);
            let roots = ZZ_POLY.all_real_roots(&f);

            assert_eq!(roots.len(), 2);
            let a = &roots[0];
            let b = &roots[1];

            let a_neg = QQ_BAR_REAL.neg_ref(a);
            let b_neg = QQ_BAR_REAL.neg_ref(b);

            a_neg.check_invariants().unwrap();
            b_neg.check_invariants().unwrap();

            println!("a = {}", QQ_BAR_REAL.to_string(a));
            println!("b = {}", QQ_BAR_REAL.to_string(b));
            println!("a_neg = {}", QQ_BAR_REAL.to_string(&a_neg));
            println!("b_neg = {}", QQ_BAR_REAL.to_string(&b_neg));

            assert_ne!(a, b);
            assert_eq!(a, &b_neg);
            assert_eq!(b, &a_neg);
        }
        {
            let f = ZZ_POLY.from_coeffs(vec![
                Integer::from(-1),
                Integer::from(0),
                Integer::from(0),
                Integer::from(0),
                Integer::from(0),
                Integer::from(0),
                Integer::from(3),
                Integer::from(1),
            ]);
            let roots = ZZ_POLY.all_real_roots(&f);

            assert_eq!(roots.len(), 3);
            for root in roots {
                QQ_BAR_REAL.neg(root).check_invariants().unwrap();
            }
        }
        {
            //example where f(g(x)) is not primitive even though f and g are
            let f =
                ZZ_POLY.from_coeffs(vec![Integer::from(-4), Integer::from(-1), Integer::from(1)]);
            let roots = ZZ_POLY.all_real_roots(&f);
            for root in roots {
                let root2 = QQ_BAR_REAL.add(
                    root,
                    QQ_BAR_REAL.from_rat(&Rational::from_signeds(1, 2)).unwrap(),
                );
                root2.check_invariants().unwrap();
            }
        }
    }

    #[test]
    fn test_real_add() {
        let f = ZZ_POLY.from_coeffs(vec![Integer::from(-2), Integer::from(0), Integer::from(3)]);
        let roots = ZZ_POLY.all_real_roots(&f);
        let a = QQ_BAR_REAL.sum(roots.iter().collect());
        assert_eq!(a, QQ_BAR_REAL.zero());

        let f = ZZ_POLY.from_coeffs(vec![
            Integer::from(-7),
            Integer::from(0),
            Integer::from(100),
        ]);
        let roots = ZZ_POLY.all_real_roots(&f);
        let a = QQ_BAR_REAL.sum(roots.iter().collect());
        assert_eq!(a, QQ_BAR_REAL.zero());

        let f = ZZ_POLY.from_coeffs(vec![
            Integer::from(-100),
            Integer::from(0),
            Integer::from(7),
        ]);
        let roots = ZZ_POLY.all_real_roots(&f);
        let a = QQ_BAR_REAL.sum(roots.iter().collect());
        assert_eq!(a, QQ_BAR_REAL.zero());
    }

    #[test]
    fn test_real_mul() {
        let f = ZZ_POLY.from_coeffs(vec![
            Integer::from(-100),
            Integer::from(0),
            Integer::from(7),
        ]);
        // (x-a)(x-b) = x^2 - 100/7
        // so ab=-100/7
        let roots = ZZ_POLY.all_real_roots(&f);
        let a = QQ_BAR_REAL.product(roots.iter().collect());
        assert_eq!(
            a,
            QQ_BAR_REAL
                .from_rat(&Rational::from_signeds(-100, 7))
                .unwrap()
        );
    }

    #[test]
    fn test_all_complex_roots() {
        let f = ZZ_POLY.from_coeffs(vec![
            Integer::from(-1),
            Integer::from(-1),
            Integer::from(0),
            Integer::from(0),
            Integer::from(0),
            Integer::from(1),
        ]);
        let roots = ZZ_POLY.all_complex_roots(&f);
        assert_eq!(roots.len(), ZZ_POLY.degree(&f).unwrap());
        for root in &roots {
            root.check_invariants().unwrap();
        }
    }

    #[test]
    fn test_complex_add() {
        // let f = ZZ_POLY.from_coeffs(vec![Integer::from(-2), Integer::from(0), Integer::from(3)]);
        // let roots = ZZ_POLY.all_real_roots(&f);
        // let a = QQ_BAR_REAL.sum(roots);
        // assert_eq!(a, QQ_BAR_REAL.zero());

        // let f = ZZ_POLY.from_coeffs(vec![
        //     Integer::from(-7),
        //     Integer::from(0),
        //     Integer::from(100),
        // ]);
        // let roots = ZZ_POLY.all_real_roots(&f);
        // let a = QQ_BAR_REAL.sum(roots);
        // assert_eq!(a, QQ_BAR_REAL.zero());

        // let f = ZZ_POLY.from_coeffs(vec![
        //     Integer::from(-100),
        //     Integer::from(0),
        //     Integer::from(7),
        // ]);
        // let roots = ZZ_POLY.all_real_roots(&f);
        // let a = QQ_BAR_REAL.sum(roots);
        // assert_eq!(a, QQ_BAR_REAL.zero());
    }
}
