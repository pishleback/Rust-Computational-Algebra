#![allow(dead_code)]

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

#[derive(Debug)]
enum SquarefreePolyRealRootInterval {
    Rational(Rational),
    //lower bound, upper bound, increasing
    //increasing = false : decreasing i.e. poly(a) > poly(b), true : increasing i.e. poly(a) < poly(b)
    Real(Rational, Rational, bool),
}

#[derive(Debug)]
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

        //check leading coefficient
        if ZZ_POLY.coeff(&self.poly_sqfr, ZZ_POLY.degree(&self.poly_sqfr).unwrap())
            <= Rational::from(0)
        {
            return Err("poly should have positive leading coefficient");
        }

        //check the isolating intervals
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

    fn to_real_roots(&self) -> Vec<RealAlgebraicNumber> {
        debug_assert!(ZZ_POLY.is_irreducible(&self.poly_sqfr).unwrap());
        let deg = ZZ_POLY.degree(&self.poly_sqfr).unwrap();
        if deg == 0 {
            vec![]
        } else if deg == 1 {
            debug_assert_eq!(self.intervals.len(), 1);
            match &self.intervals[0] {
                SquarefreePolyRealRootInterval::Rational(a) => vec![RealAlgebraicNumber::Rational(a.clone())],
                SquarefreePolyRealRootInterval::Real(_, _, _) => panic!(),
            }
        } else {
            todo!();
        }
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

        let mut n = self.degree(&poly).unwrap();
        let mut l = vec![(Natural::from(0u8), 0, poly.clone())];
        let mut isol = vec![];
        while l.len() != 0 {
            let (c, k, mut q) = l.pop().unwrap();
            if ZZ_POLY.evaluate(&q, &Integer::from(0)) == Integer::from(0) {
                //q = q/x
                q = self.div(q, self.var()).unwrap();
                n -= 1;
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
                    coeff * Integer::from(2) << (n - i)
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

    //isolate all real roots of a squarefree (no repeated roots) polynomial
    fn real_roots_squarefree(
        &self,
        poly: Polynomial<Integer>,
        opt_a: Option<&Rational>,
        opt_b: Option<&Rational>,
    ) -> SquarefreePolyRealRoots {
        assert_ne!(poly, self.zero());
        //poly should be squarefree
        debug_assert_eq!(
            self.degree(&self.primitive_squarefree_part(poly.clone()))
                .unwrap(),
            self.degree(&poly).unwrap()
        );

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
            if opt_a.is_some() {
                if !(opt_a.unwrap() < &root) {
                    return SquarefreePolyRealRoots {
                        poly_sqfr: poly,
                        intervals: vec![],
                    };
                }
            }
            if opt_b.is_some() {
                if !(&root < opt_b.unwrap()) {
                    return SquarefreePolyRealRoots {
                        poly_sqfr: poly,
                        intervals: vec![],
                    };
                }
            }
            SquarefreePolyRealRoots {
                poly_sqfr: poly,
                intervals: vec![SquarefreePolyRealRootInterval::Rational(root)],
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
                        None => self.real_roots_squarefree(poly, Some(a_val), Some(&m)),
                    },
                    None => match opt_b {
                        Some(b_val) => self.real_roots_squarefree(poly, Some(&-m), Some(b_val)),
                        None => {
                            let neg_m = -m.clone();
                            self.real_roots_squarefree(poly, Some(&neg_m), Some(&m))
                        }
                    },
                };
            }
            let (a, b) = (opt_a.unwrap(), opt_b.unwrap());
            assert!(a < b);

            //there are no roots A < r < B if A >= B
            if a >= b {
                return SquarefreePolyRealRoots {
                    poly_sqfr: poly,
                    intervals: vec![],
                };
            }

            //condition for collin_akritas. might need to handle this case
            debug_assert_ne!(evaluate_at_rational(&poly, a), Rational::from(0));
            debug_assert_ne!(evaluate_at_rational(&poly, b), Rational::from(0));

            //apply a transformation to p so that its roots in (a, b) are moved to roots in (0, 1)
            let (_, trans_poly) = QQ_POLY.factor_primitive_fof(&QQ_POLY.compose(
                &ZZ_POLY.apply_map(&QQ, &poly, |c| Rational::from(c)),
                &QQ_POLY.from_coeffs(vec![a.clone(), b.clone() - a.clone()]),
            ));

            let mut intervals = vec![];
            for (c, k, h) in self.isolate_real_roots_by_collin_akritas(&trans_poly) {
                let d = Natural::from(1u8) << k;
                let interval_a = (b - a) * Rational::from_naturals(c.clone(), d.clone()) + a;
                if h {
                    let interval_b =
                        (b - a) * Rational::from_naturals(&c + Natural::from(1u8), d.clone()) + a;
                    let sign_b = evaluate_at_rational(&poly, &interval_b) > Rational::from(0);
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
            SquarefreePolyRealRoots {
                poly_sqfr: poly,
                intervals,
            }
        }
    }

    //isolate all real roots of the irreducible poly in the open interval (a, b)
    fn real_roots_irreducible(
        &self,
        poly: &Polynomial<Integer>,
        opt_a: Option<&Rational>,
        opt_b: Option<&Rational>,
    ) -> Vec<RealAlgebraicNumber> {
        assert_ne!(poly, &self.zero());
        debug_assert!(self.is_irreducible(&poly).unwrap());

        // self.real_roots_squarefree(poly.clone(), opt_a, opt_b)
        //     .to_real_roots()

        let d = ZZ_POLY.degree(&poly).unwrap();
        if d == 0 {
            //constant polynomial has no roots
            vec![]
        } else if d == 1 {
            //poly = a+bx
            //root = -a/b
            let root = -Rational::from(self.coeff(&poly, 0)) / Rational::from(self.coeff(&poly, 1));
            if opt_a.is_some() {
                if !(opt_a.unwrap() < &root) {
                    return vec![];
                }
            }
            if opt_b.is_some() {
                if !(&root < opt_b.unwrap()) {
                    return vec![];
                }
            }
            vec![RealAlgebraicNumber::Rational(root)]
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
                        None => self.real_roots_irreducible(poly, Some(a_val), Some(&m)),
                    },
                    None => match opt_b {
                        Some(b_val) => self.real_roots_irreducible(poly, Some(&-m), Some(b_val)),
                        None => {
                            let neg_m = -m.clone();
                            self.real_roots_irreducible(poly, Some(&neg_m), Some(&m))
                        }
                    },
                };
            }
            let (a, b) = (opt_a.unwrap(), opt_b.unwrap());

            assert!(a < b);

            //there are no roots A < r < B if A >= B
            if a >= b {
                return vec![];
            }

            //apply a transformation to p so that its roots in (a, b) are moved to roots in (0, 1)
            let (_, trans_poly) = QQ_POLY.factor_primitive_fof(&QQ_POLY.compose(
                &ZZ_POLY.apply_map(&QQ, &poly, |c| Rational::from(c)),
                &QQ_POLY.from_coeffs(vec![a.clone(), b.clone() - a.clone()]),
            ));

            let mut roots = vec![];
            for (c, k, h) in self.isolate_real_roots_by_collin_akritas(&trans_poly) {
                assert!(h); //should not isolate any rational roots since poly is irreducible with degree >= 2
                let d = Natural::from(1u8) << k;
                roots.push(RealAlgebraicNumber::Real(
                    RealAlgebraicRoot::new_wide_bounds(
                        poly.clone(),
                        (b - a) * Rational::from_naturals(c.clone(), d.clone()) + a,
                        (b - a) * Rational::from_naturals(&c + Natural::from(1u8), d.clone()) + a,
                    ),
                ));
            }
            roots
        }
    }

    //get the real roots with multiplicity of poly
    pub fn real_roots(
        &self,
        poly: &Polynomial<Integer>,
        a: Option<&Rational>,
        b: Option<&Rational>,
    ) -> Vec<RealAlgebraicNumber> {
        assert_ne!(poly, &self.zero());
        let factors = self.factor(&poly).unwrap();
        let mut roots = vec![];
        for (factor, k) in factors.factors() {
            for root in self.real_roots_irreducible(factor, a, b) {
                let mut i = Natural::from(0u8);
                while &i < k {
                    roots.push(root.clone());
                    i += Natural::from(1u8);
                }
            }
        }
        roots
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

        //checks
        debug_assert_eq!(
            evaluate_at_rational(&a_vert_re, c),
            evaluate_at_rational(&c_horz_re, a)
        );
        debug_assert_eq!(
            evaluate_at_rational(&a_vert_re, d),
            evaluate_at_rational(&d_horz_re, a)
        );
        debug_assert_eq!(
            evaluate_at_rational(&b_vert_re, c),
            evaluate_at_rational(&c_horz_re, b)
        );
        debug_assert_eq!(
            evaluate_at_rational(&b_vert_re, d),
            evaluate_at_rational(&d_horz_re, b)
        );
        debug_assert_eq!(
            evaluate_at_rational(&a_vert_im, c),
            evaluate_at_rational(&c_horz_im, a)
        );
        debug_assert_eq!(
            evaluate_at_rational(&a_vert_im, d),
            evaluate_at_rational(&d_horz_im, a)
        );
        debug_assert_eq!(
            evaluate_at_rational(&b_vert_im, c),
            evaluate_at_rational(&c_horz_im, b)
        );
        debug_assert_eq!(
            evaluate_at_rational(&b_vert_im, d),
            evaluate_at_rational(&d_horz_im, b)
        );

        //compute squarefree versions for when only care about the roots without multiplicity
        let a_vert_re_sqfr = self.primitive_squarefree_part(a_vert_re.clone());
        let a_vert_im_sqfr = self.primitive_squarefree_part(a_vert_im.clone());
        let b_vert_re_sqfr = self.primitive_squarefree_part(b_vert_re.clone());
        let b_vert_im_sqfr = self.primitive_squarefree_part(b_vert_im.clone());
        let c_horz_re_sqfr = self.primitive_squarefree_part(c_horz_re.clone());
        let c_horz_im_sqfr = self.primitive_squarefree_part(c_horz_im.clone());
        let d_horz_re_sqfr = self.primitive_squarefree_part(d_horz_re.clone());
        let d_horz_im_sqfr = self.primitive_squarefree_part(d_horz_im.clone());

        //evaluate the real and imaginary parts at the verticies of the box
        let z1r = evaluate_at_rational(&a_vert_re_sqfr, c) == Rational::from(0);
        debug_assert_eq!(
            z1r,
            evaluate_at_rational(&c_horz_re_sqfr, a) == Rational::from(0)
        );
        let z2r = evaluate_at_rational(&a_vert_re_sqfr, d) == Rational::from(0);
        debug_assert_eq!(
            z2r,
            evaluate_at_rational(&d_horz_re_sqfr, a) == Rational::from(0)
        );
        let z3r = evaluate_at_rational(&b_vert_re_sqfr, c) == Rational::from(0);
        debug_assert_eq!(
            z3r,
            evaluate_at_rational(&c_horz_re_sqfr, b) == Rational::from(0)
        );
        let z4r = evaluate_at_rational(&b_vert_re_sqfr, d) == Rational::from(0);
        debug_assert_eq!(
            z4r,
            evaluate_at_rational(&d_horz_re_sqfr, b) == Rational::from(0)
        );

        let z1i = evaluate_at_rational(&a_vert_im_sqfr, c) == Rational::from(0);
        debug_assert_eq!(
            z1i,
            evaluate_at_rational(&c_horz_im_sqfr, a) == Rational::from(0)
        );
        let z2i = evaluate_at_rational(&a_vert_im_sqfr, d) == Rational::from(0);
        debug_assert_eq!(
            z2i,
            evaluate_at_rational(&d_horz_im_sqfr, a) == Rational::from(0)
        );
        let z3i = evaluate_at_rational(&b_vert_im_sqfr, c) == Rational::from(0);
        debug_assert_eq!(
            z3i,
            evaluate_at_rational(&c_horz_im_sqfr, b) == Rational::from(0)
        );
        let z4i = evaluate_at_rational(&b_vert_im_sqfr, d) == Rational::from(0);
        debug_assert_eq!(
            z4i,
            evaluate_at_rational(&d_horz_im_sqfr, b) == Rational::from(0)
        );

        //if the polynomial has a root at any vertix then we must give up
        for (zr, zi) in vec![(z1r, z1i), (z2r, z2i), (z3r, z3i), (z4r, z4i)] {
            if zr && zi {
                return None;
            }
        }

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
            re_sqfr: &Polynomial<Integer>,
            im: &Polynomial<Integer>,
            im_sqfr: &Polynomial<Integer>,
            s: &Rational,
            t: &Rational,
        ) -> Option<Vec<Crossing>> {
            debug_assert_eq!(re == &ZZ_POLY.zero(), re_sqfr == &ZZ_POLY.zero());
            debug_assert_eq!(im == &ZZ_POLY.zero(), im_sqfr == &ZZ_POLY.zero());
            //because if the real and imaginary part are both constant at 0 then poly has infinitely many complex zeros which is not possible
            debug_assert_ne!((re_sqfr, im_sqfr), (&ZZ_POLY.zero(), &ZZ_POLY.zero()));
            if re_sqfr == &ZZ_POLY.zero() {
                //the image is doing a path confied to the imaginary axis
                let roots_im = ZZ_POLY.real_roots(im, Some(s), Some(t));
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
            } else if im_sqfr == &ZZ_POLY.zero() {
                //the image is doing a path confied to the real axis
                let roots_re = ZZ_POLY.real_roots(re, Some(s), Some(t));
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
                /*
                roots_re = real_roots(re, s, t)
                roots_im = real_roots(im, s, t)
                for x, y in itertools.product(roots_re, roots_im):
                    if x == y:
                        raise BoundaryRoot("Edge Root")
                    RealRoot.separate(x, y)

                def sign_at(poly, root):
                    at_a = Poly.evaluate(poly, root.a)
                    at_b = Poly.evaluate(poly, root.b)
                    assert at_a != 0 and at_b != 0
                    sign_a = at_a > 0
                    sign_b = at_b > 0
                    if sign_a and sign_b:
                        return True
                    elif not sign_a and not sign_b:
                        return False
                    else:
                        #because the real and imaginary roots have been seperated
                        #for example, the interval around each real root contains no root of the imaginary part
                        #thus the sign of the imaginary part evaluated at the location of the real root is constant
                        #thus the sign is equal to the sign at either end point
                        assert False

                roots = [(x, sign_at(im, x), False) for x in roots_re] + [(x, sign_at(re, x), True) for x in roots_im]

                def to_mod4(sign, reim):
                    return {(True, True) : 0, (True, False) : 1, (False, True) : 2, (False, False) : 3}[(sign, reim)]

                return [to_mod4(tri[1], tri[2]) for tri in sorted(roots, key = lambda tri : tri[0])]

                 */

                //want to isolate roots of squarefree polynomials without factoring
                //get ordered real roots in some structure
                //get ordered imag roots in some structure
                //do a merge sort pass to interleave the real and imaginary roots in the correct order
                //    if a real root equals an imaginary root then there is a root on the boundary
                //for each root of one type, compute the sign of the other part when evaluated at the root

                println!("{:?} {:?} {:?} {:?}", re, im, s, t);
                todo!()
            }
        }

        let mut winding = vec![];
        for cr in vec![
            crossings::<false>(
                &a_vert_re,
                &a_vert_re_sqfr,
                &a_vert_im,
                &a_vert_im_sqfr,
                c,
                d,
            ),
            crossings::<false>(
                &d_horz_re,
                &d_horz_re_sqfr,
                &d_horz_im,
                &d_horz_im_sqfr,
                a,
                b,
            ),
            crossings::<true>(
                &b_vert_re,
                &b_vert_re_sqfr,
                &b_vert_im,
                &b_vert_im_sqfr,
                c,
                d,
            ),
            crossings::<true>(
                &c_horz_re,
                &c_horz_re_sqfr,
                &c_horz_im,
                &c_horz_im_sqfr,
                a,
                b,
            ),
        ] {
            match cr {
                Some(mut w) => winding.append(&mut w),
                None => {
                    return None;
                }
            }
        }

        println!("winding = {:?}", winding);

        //compute the winding number = number of roots
        if winding.len() == 0 {
            Some(0)
        } else {
            todo!();
        }
    }
}

#[derive(Debug, Clone, Hash)]
pub struct RealAlgebraicRoot {
    poly: Polynomial<Integer>, //a primitive irreducible polynomial of degree >= 2 with a unique real root between a and b
    //an arbitrarily small interval containing the root. May be mutated
    tight_a: Rational, //tight lower bound
    tight_b: Rational, //tight upper bound
    //a heuristically large interval containing the root. Should not shrink
    wide_a: Option<Rational>, //wide lower bound. None means -inf
    wide_b: Option<Rational>, //wide upper bound. None means +inf
    //false : decreasing i.e. poly(a) > poly(b), true : increasing i.e. poly(a) < poly(b)
    dir: bool,
}

impl RealAlgebraicRoot {
    pub fn check_invariants(&self) -> Result<(), &'static str> {
        if !(self.tight_a < self.tight_b) {
            return Err("tight a should be strictly less than b");
        }
        if !(self.wide_a < self.wide_b) {
            return Err("wide a should be strictly less than b");
        }
        if self.poly
            != ZZ_POLY
                .factor_fav_assoc(ZZ_POLY.primitive_squarefree_part(self.poly.clone()))
                .1
        {
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
            wide_a: Some(wide_a),
            wide_b: Some(wide_b),
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
        let eq_poly = self.poly == other.poly; //polys should be irreducible primitive fav-assoc so this is valid
        loop {
            //test for equality: if the tight bounds on one are within the wide bounds of the other
            if eq_poly {
                if {
                    match &other.wide_a {
                        Some(wa) => wa <= &self.tight_a,
                        None => true,
                    }
                } && {
                    match &other.wide_b {
                        Some(wb) => &self.tight_b <= wb,
                        None => true,
                    }
                } {
                    return std::cmp::Ordering::Equal;
                }
                if {
                    match &self.wide_a {
                        Some(wa) => wa <= &other.tight_a,
                        None => true,
                    }
                } && {
                    match &self.wide_b {
                        Some(wb) => &other.tight_b <= wb,
                        None => true,
                    }
                } {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComplexAlgebraicRoot {
    tight_a: Rational, //tight lower bound for the real part
    tight_b: Rational, //tight upper bound for the real part
    tight_c: Rational, //tight lower bound for the imaginary part
    tight_d: Rational, //tight upper bound for the imaginary part

    wide_a: Rational, //wide lower bound for the real part
    wide_b: Rational, //wide upper bound for the real part
    wide_c: Rational, //wide lower bound for the imaginary part
    wide_d: Rational, //wide upper bound for the imaginary part

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
        if !(self.wide_a < self.wide_b) {
            return Err("wide a should be strictly less than b");
        }
        if !(self.wide_c < self.wide_d) {
            return Err("wide c should be strictly less than d");
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
        Ok(())
    }
}

#[derive(Debug, Clone, Hash)]
pub enum RealAlgebraicNumber {
    Rational(Rational),
    Real(RealAlgebraicRoot),
}

impl RealAlgebraicNumber {
    pub fn check_invariants(&self) -> Result<(), &'static str> {
        match self {
            RealAlgebraicNumber::Rational(x) => {}
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ComplexAlgebraicNumber {
    Real(RealAlgebraicNumber),
    Complex(ComplexAlgebraicRoot),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RealAlgebraicField {}

impl ComRing for RealAlgebraicField {
    type ElemT = RealAlgebraicNumber;

    fn to_string(&self, elem: &Self::ElemT) -> String {
        match elem {
            RealAlgebraicNumber::Rational(a) => a.to_string(),
            RealAlgebraicNumber::Real(a) => a.to_string(),
        }
    }

    fn zero(&self) -> Self::ElemT {
        RealAlgebraicNumber::Rational(Rational::from(0))
    }

    fn one(&self) -> Self::ElemT {
        RealAlgebraicNumber::Rational(Rational::from(1))
    }

    fn neg_mut(&self, elem: &mut Self::ElemT) {
        todo!()
    }

    fn add_mut(&self, elem: &mut Self::ElemT, offset: &Self::ElemT) {
        todo!()
    }

    fn mul_mut(&self, elem: &mut Self::ElemT, mul: &Self::ElemT) {
        todo!()
    }

    fn div(&self, a: Self::ElemT, b: Self::ElemT) -> Result<Self::ElemT, RingDivisionError> {
        todo!()
    }
}

impl IntegralDomain for RealAlgebraicField {}

impl Field for RealAlgebraicField {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComplexAlgebraicField {}

impl ComRing for ComplexAlgebraicField {
    type ElemT = ComplexAlgebraicNumber;

    fn to_string(&self, elem: &Self::ElemT) -> String {
        match elem {
            ComplexAlgebraicNumber::Real(a) => RealAlgebraicField {}.to_string(a),
            ComplexAlgebraicNumber::Complex(a) => todo!(),
        }
    }

    fn zero(&self) -> Self::ElemT {
        ComplexAlgebraicNumber::Real(RealAlgebraicNumber::Rational(Rational::from(0)))
    }

    fn one(&self) -> Self::ElemT {
        ComplexAlgebraicNumber::Real(RealAlgebraicNumber::Rational(Rational::from(1)))
    }

    fn neg_mut(&self, elem: &mut Self::ElemT) {
        todo!()
    }

    fn add_mut(&self, elem: &mut Self::ElemT, offset: &Self::ElemT) {
        todo!()
    }

    fn mul_mut(&self, elem: &mut Self::ElemT, mul: &Self::ElemT) {
        todo!()
    }

    fn div(&self, a: Self::ElemT, b: Self::ElemT) -> Result<Self::ElemT, RingDivisionError> {
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
        ]);
        let f = ZZ_POLY.primitive_squarefree_part(f);
        //f is a squarefree polynomial with lots of roots
        println!("f = {:?}", f);
        let intervals = ZZ_POLY.real_roots_squarefree(f, None, None);
        println!("intervals = {:?}", intervals);
        intervals.check_invariants().unwrap();
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
                    None
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
                    None
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
            for root in ZZ_POLY.real_roots(&f, None, None) {
                all_roots.push(root.clone());
            }
        }

        all_roots.sort();

        for mut root in &mut all_roots {
            root.check_invariants().unwrap();
            match &mut root {
                RealAlgebraicNumber::Rational(a) => {}
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
        debug_assert_eq!(
            vert_re_f,
            ZZ_POLY.from_coeffs(vec![Integer::from(13), Integer::from(0), Integer::from(-6)])
        );
        debug_assert_eq!(
            vert_im_f,
            ZZ_POLY.from_coeffs(vec![
                Integer::from(0),
                Integer::from(15),
                Integer::from(0),
                Integer::from(-1)
            ])
        );

        let (vert_re_f, vert_im_f) = ZZ_POLY.at_fixed_re(&f, &Rational::from_signeds(1, 2));
        println!("re = {}", ZZ_POLY.to_string(&vert_re_f));
        println!("im = {}", ZZ_POLY.to_string(&vert_im_f));
        // f(z) = z^3 + 3z - 1
        // f(1/2 + xi) = 5 + 30ix - 12x^2 - 8ix^3
        debug_assert_eq!(
            vert_re_f,
            ZZ_POLY.from_coeffs(vec![Integer::from(5), Integer::from(0), Integer::from(-12)])
        );
        debug_assert_eq!(
            vert_im_f,
            ZZ_POLY.from_coeffs(vec![
                Integer::from(0),
                Integer::from(30),
                Integer::from(0),
                Integer::from(-8)
            ])
        );

        let (vert_re_f, vert_im_f) = ZZ_POLY.at_fixed_im(&f, &Rational::from(2));
        println!("re = {}", ZZ_POLY.to_string(&vert_re_f));
        println!("im = {}", ZZ_POLY.to_string(&vert_im_f));
        // f(z) = z^3 + 3z - 1
        // f(x + 2i) = -1 -2i -9x + 6ix^2 + x^3
        debug_assert_eq!(
            vert_re_f,
            ZZ_POLY.from_coeffs(vec![
                Integer::from(-1),
                Integer::from(-9),
                Integer::from(0),
                Integer::from(1)
            ])
        );
        debug_assert_eq!(
            vert_im_f,
            ZZ_POLY.from_coeffs(vec![Integer::from(-2), Integer::from(0), Integer::from(6),])
        );

        let (vert_re_f, vert_im_f) = ZZ_POLY.at_fixed_im(&f, &Rational::from_signeds(1, 2));
        println!("re = {}", ZZ_POLY.to_string(&vert_re_f));
        println!("im = {}", ZZ_POLY.to_string(&vert_im_f));
        // f(z) = z^3 + 3z - 1
        // f(x + 1/2i) = -8 +11i + 18x + 12ix^2 + 8x^3
        debug_assert_eq!(
            vert_re_f,
            ZZ_POLY.from_coeffs(vec![
                Integer::from(-8),
                Integer::from(18),
                Integer::from(0),
                Integer::from(8)
            ])
        );
        debug_assert_eq!(
            vert_im_f,
            ZZ_POLY.from_coeffs(vec![Integer::from(11), Integer::from(0), Integer::from(12),])
        );
    }
}