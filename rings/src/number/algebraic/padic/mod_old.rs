use std::rc::Rc;

use algebraeon_sets::structure::*;
use malachite_base::num::arithmetic::traits::UnsignedAbs;
use malachite_base::num::{
    arithmetic::traits::{DivMod, Mod, Pow},
    basic::traits::{One, Zero},
};
use malachite_nz::{integer::Integer, natural::Natural};
use malachite_q::Rational;

use crate::number::algebraic::isolated_roots::poly_tools::root_sum_poly;
use crate::{
    number::natural::primes::is_prime,
    polynomial::polynomial::*,
    structure::{quotient::QuotientStructure, structure::*},
};

fn pos_int_to_nat(x: Integer) -> Natural {
    debug_assert!(x >= Integer::ZERO);
    x.unsigned_abs()
}

fn padic_int_valuation(p: &Natural, mut n: Integer) -> Option<usize> {
    debug_assert!(is_prime(p));
    let p = Integer::from(p);
    if n == Natural::ZERO {
        None
    } else {
        let mut k = 0;
        let mut r;
        loop {
            (n, r) = n.div_mod(&p);
            if r == Natural::ZERO {
                k += 1;
                continue;
            } else {
                break;
            }
        }
        Some(k)
    }
}

fn padic_rat_valuation(p: &Natural, r: Rational) -> Option<isize> {
    if r == Rational::ZERO {
        None
    } else {
        let (n, d) = r.into_numerator_and_denominator();
        Some(
            (padic_int_valuation(p, Integer::from(n)).unwrap() as isize)
                - (padic_int_valuation(p, Integer::from(d)).unwrap() as isize),
        )
    }
}

fn padic_digits(p: &Natural, mut n: Natural) -> Vec<Natural> {
    debug_assert!(is_prime(p));
    let mut digits = vec![];
    let mut r;
    while n != 0 {
        (n, r) = n.div_mod(p);
        digits.push(r);
    }
    digits
}

#[derive(Debug, Clone)]
struct PAdicIntegerAlgebraicRoot {
    p: Natural,                 // a prime number
    poly: Polynomial<Integer>,  // a primitive irreducible degree >= 2 polynomial
    dpoly: Polynomial<Integer>, // the derivative of poly
    dpoly_valuation: usize, // f'(a) where a is the approximate root OR equivelently the lifted root.
    approx_root: Integer,   // an approximation to the root represented by this struct modulo p^k
    k: usize,
    // Furthermore approx_root must satisfy unique lifting to p-adic integers. By hensels lemma, this is the case whenever
    // |f(a)|_p < |f'(a)|^2
    // where f = poly and a = approx root
    // The true root b will agree with the approximate root a in the first dpoly_valuation+1 digits since |a-b| < |f'(a)| = |f'(b)|
}

impl PAdicIntegerAlgebraicRoot {
    fn modulus(&self) -> Natural {
        self.p.clone().pow(self.k as u64)
    }

    fn check(&self) -> Result<(), &'static str> {
        let pk = self.modulus();
        if !is_prime(&self.p) {
            return Err("p not prime");
        }
        match self.poly.degree() {
            Some(d) => {
                if d <= 1 {
                    return Err("deg(poly) <= 1");
                }
            }
            None => {
                return Err("poly = 0");
            }
        }
        if !Polynomial::is_irreducible(&self.poly) {
            return Err("poly is not irreducible");
        }
        if self.poly != self.poly.clone().primitive_part().unwrap() {
            return Err("f is not primitive");
        }
        if self.dpoly != self.poly.clone().derivative() {
            return Err("dpoly is not the derivative of poly");
        }
        if self.approx_root >= pk {
            return Err("approx root >= p^k");
        }

        let poly_mod_pk = PolynomialStructure::new(
            QuotientStructure::new_ring(Integer::structure(), Integer::from(pk)).into(),
        );

        let vfa = padic_int_valuation(&self.p, poly_mod_pk.evaluate(&self.poly, &self.approx_root));
        let vdfa = padic_int_valuation(
            &self.p,
            poly_mod_pk.evaluate(&self.dpoly, &self.approx_root),
        );
        match (vfa, vdfa) {
            (None, None) => {
                return Err("f(a) = f'(a) = 0");
            }
            (None, Some(_)) => {}
            (Some(_), None) => {
                return Err("f(a) != 0 and f'(a) = 0");
            }
            (Some(poly_val), Some(dpoly_val)) => {
                if !(poly_val > 2 * dpoly_val) {
                    return Err("|f(a)|_p < |f'(a)|^2 does not hold");
                }
            }
        }
        if vdfa.unwrap() != self.dpoly_valuation {
            return Err("v(f'(a)) does not match true value");
        }
        Ok(())
    }

    fn refine(&mut self) {
        self.k += 1;
        // p^{k+1}
        let pk1 = self.modulus();
        // Z/p^{k+1}Z
        let mod_pk1 = Rc::new(QuotientStructure::new_ring(
            Integer::structure(),
            Integer::from(&pk1),
        ));
        // Z/p^{k+1}Z[x]
        let poly_mod_pk1 = PolynomialStructure::new(mod_pk1.clone());

        // Update approximate root by Newtons method:
        // a <- a - f(a)/f'(a)
        let mut fa = poly_mod_pk1.evaluate(&self.poly, &self.approx_root); // f(a)
        let mut dfa = poly_mod_pk1.evaluate(&self.dpoly, &self.approx_root); // f'(a)
        let m = Integer::from(self.p.clone().pow(self.dpoly_valuation as u64)); // m is a common divisor of the top and bottom of f(a)/f'(a)
        (fa, dfa) = (fa / &m, dfa / &m);
        // now dfa != 0 mod p so we can find an inverse modulo p^{k+1}
        let (g, _, inv_dfa) = Integer::xgcd(&Integer::from(pk1), &dfa);
        debug_assert_eq!(g, Integer::ONE);
        self.approx_root =
            mod_pk1.add(&self.approx_root, &mod_pk1.neg(&mod_pk1.mul(&fa, &inv_dfa)));
    }

    fn refine_to_valuation(&mut self, k: usize) {
        while self.correct_approx_valuation() < k {
            self.refine();
        }
    }

    fn correct_approx_valuation(&self) -> usize {
        let pk = self.p.clone().pow(self.k as u64);
        let mod_pk = Rc::new(QuotientStructure::new_ring(
            Integer::structure(),
            Integer::from(&pk),
        ));
        let poly_mod_pk = PolynomialStructure::new(mod_pk.clone());
        // a = a-f(a)/f'(a) each iteration
        // v(f(a)) increases by at least v(f(a))-2v(f'(a)) each refinement
        // so the first v(f(a))-v(f'(a)) digits are correct
        let fa = poly_mod_pk.evaluate(&self.poly, &self.approx_root);
        let vfa = padic_int_valuation(&self.p, fa).unwrap_or(self.k);
        vfa - self.dpoly_valuation
    }

    fn reduce_modulo_valuation(&mut self, k: usize) -> Natural {
        self.refine_to_valuation(k);
        let pk = Integer::from(self.p.clone().pow(k as u64));
        pos_int_to_nat(Integer::rem(&self.approx_root, &pk))
    }

    fn equal_mut(&mut self, other: &mut Self) -> bool {
        let p = &self.p;
        if p != &other.p {
            return false;
        }
        if self.poly != other.poly {
            return false;
        }
        let dpoly_valuation = self.dpoly_valuation;
        if dpoly_valuation != other.dpoly_valuation {
            return false;
        }
        self.reduce_modulo_valuation(dpoly_valuation + 1)
            == other.reduce_modulo_valuation(dpoly_valuation + 1)
    }

    fn rightshift(&mut self) -> Option<Self> {
        if self.reduce_modulo_valuation(1) != Integer::ZERO {
            None
        } else {
            let (mul, poly) = self
                .poly
                .apply_map_with_powers(|(power, coeff)| {
                    coeff * Integer::from(&self.p).nat_pow(&Natural::from(power))
                })
                .factor_primitive()
                .unwrap();
            let dpoly = poly.clone().derivative();
            let approx_root = &self.approx_root / Integer::from(&self.p);
            let dpoly_valuation =
                self.dpoly_valuation + 1 - padic_int_valuation(&self.p, mul).unwrap();
            let ans = Self {
                p: self.p.clone(),
                poly,
                dpoly,
                dpoly_valuation,
                approx_root,
                k: self.k - 1,
            };
            #[cfg(debug_assertions)]
            ans.check().unwrap();
            Some(ans)
        }
    }

    /// Divide by the largest power of p possible, so that self != 0 mod p, and return the power
    fn fully_rightshift(&mut self) -> (Self, usize) {
        let mut rshifted = self.clone();
        let mut k = 0;
        loop {
            match rshifted.rightshift() {
                Some(new_rshifted) => {
                    rshifted = new_rshifted;
                    k += 1;
                }
                None => {
                    return (rshifted, k);
                }
            }
        }
    }

    fn leftshift(mut self, k: usize) -> Self {
        let deg = self.poly.degree().unwrap();
        // Refine so that self.k is greater then what self.dpoly_valuation will be
        while self.k <= k * (deg - 1) {
            self.refine();
        }
        // Replace self.poly(x) with p^{k*deg} * self.poly(x / p^{k*deg}})
        let (mul, poly) = self
            .poly
            .apply_map_with_powers(|(power, coeff)| {
                coeff * Integer::from(&self.p).nat_pow(&Natural::from(k * (deg - power)))
            })
            .factor_primitive()
            .unwrap();
        self.poly = poly;
        self.dpoly = self.poly.clone().derivative();
        self.approx_root *= Integer::from(&self.p).nat_pow(&Natural::from(k));
        self.dpoly_valuation += k * (deg - 1) - padic_int_valuation(&self.p, mul).unwrap();
        self.k += k;
        #[cfg(debug_assertions)]
        self.check().unwrap();
        self
    }

    fn neg(mut self) -> Self {
        self.poly = Polynomial::compose(
            &self.poly,
            &Polynomial::from_coeffs(vec![Integer::from(0), Integer::from(-1)]),
        )
        .fav_assoc();
        self.dpoly = Polynomial::compose(
            &self.dpoly,
            &Polynomial::from_coeffs(vec![Integer::from(0), Integer::from(-1)]),
        )
        .fav_assoc();
        self.approx_root = QuotientStructure::new_ring(
            Integer::structure(),
            self.p.nat_pow(&Natural::from(self.k)).into(),
        )
        .neg(&self.approx_root);
        #[cfg(debug_assertions)]
        self.check().unwrap();
        self
    }

    fn add_rat(mut self, rat: &Rational) -> Self {
        #[cfg(debug_assertions)]
        if let Some(rat_valuation) = padic_rat_valuation(&self.p, rat.clone()) {
            debug_assert!(rat_valuation >= 0);
        }

        self.poly = Polynomial::compose(
            &self.poly.apply_map(|c| Rational::from(c)),
            &Polynomial::from_coeffs(vec![-rat, Rational::ONE]),
        )
        .primitive_part_fof();
        self.dpoly = self.poly.clone().derivative();

        let padic_rat = PAdicRational {
            p: self.p.clone(),
            rat: rat.clone(),
        };
        let padic_rat_value = padic_rat.reduce_modulo_valuation(self.k as isize);
        let (value, shift) = padic_rat_value.natural_and_shift();
        debug_assert!(shift >= 0);
        self.approx_root +=
            Integer::from(value) * Integer::from(&self.p).nat_pow(&Natural::from(shift as usize));
        self.approx_root =
            self.approx_root % Integer::from(&self.p).nat_pow(&Natural::from(self.k));

        #[cfg(debug_assertions)]
        self.check().unwrap();
        self
    }

    fn add_mut(a: &mut Self, b: &mut Self) -> PAdicAlgebraic {
        let p = a.p.clone();
        debug_assert_eq!(p, b.p);
        let mut candidates = root_sum_poly(&a.poly, &b.poly)
            .primitive_squarefree_part()
            .all_padic_roots(&p);
        let mut k = usize::min(a.correct_approx_valuation(), b.correct_approx_valuation());
        let mut pk = p.nat_pow(&Natural::from(k));
        while candidates.len() > 1 {
            candidates = candidates
                .into_iter()
                .filter_map(|mut s| {
                    if s.reduce_modulo_valuation(k as isize).natural().unwrap()
                        == (a.reduce_modulo_valuation(k) + b.reduce_modulo_valuation(k)).mod_op(&pk)
                    {
                        Some(s)
                    } else {
                        None
                    }
                })
                .collect();
            k += 1;
            pk *= &p;
        }
        candidates.into_iter().next().unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct PAdicAlgebraicRoot {
    // Multiply int_root by p^k to get the p-adic root represented by this struct
    root: PAdicIntegerAlgebraicRoot, // should be non-zero modulo p
    shift: isize,                    // how much to left shift by
}

impl From<PAdicIntegerAlgebraicRoot> for PAdicAlgebraicRoot {
    fn from(mut value: PAdicIntegerAlgebraicRoot) -> Self {
        let (root, k) = value.fully_rightshift();
        PAdicAlgebraicRoot {
            root,
            shift: k as isize,
        }
    }
}

impl PAdicAlgebraicRoot {
    fn check(&mut self) -> Result<(), &'static str> {
        if self.root.reduce_modulo_valuation(1) == Integer::ZERO {
            return Err("self.root = 0 mod p");
        }
        Ok(())
    }

    fn shift_by(mut self, k: isize) -> Self {
        self.shift += k;
        self
    }

    fn reduce_modulo_valuation<'a>(&'a mut self, k: isize) -> PAdicTerminatingRational<'a> {
        if k < self.shift {
            PAdicTerminatingRational {
                p: &self.root.p,
                value: Natural::ZERO,
                k,
                shift: 0,
            }
        } else {
            let num_digits = (k - self.shift) as usize;
            let value = self.root.reduce_modulo_valuation(num_digits);
            PAdicTerminatingRational {
                p: &self.root.p,
                value,
                k,
                shift: self.shift,
            }
        }
    }

    fn unwrap(self) -> (PAdicIntegerAlgebraicRoot, isize) {
        (self.root, self.shift)
    }
}

#[derive(Debug, Clone)]
pub struct PAdicRational {
    p: Natural, // a prime number
    rat: Rational,
}
impl PAdicRational {
    fn reduce_modulo_valuation<'a>(&'a self, k: isize) -> PAdicTerminatingRational<'a> {
        match padic_rat_valuation(&self.p, self.rat.clone()) {
            Some(shift) => {
                let shifted_rat = &self.rat
                    * Rational::from(&self.p)
                        .int_pow(&Integer::from(-shift))
                        .unwrap();
                let (n, d) = (shifted_rat.numerator(), shifted_rat.denominator());
                debug_assert_eq!(padic_int_valuation(&self.p, n.clone()).unwrap(), 0);
                debug_assert_eq!(padic_int_valuation(&self.p, d.clone()).unwrap(), 0);
                if k < shift {
                    PAdicTerminatingRational {
                        p: &self.p,
                        value: Natural::ZERO,
                        k,
                        shift: 0,
                    }
                } else {
                    let num_digits = (k - shift) as usize;
                    let pn = Integer::from(&self.p).nat_pow(&Natural::from(num_digits)); // p^{num_digits}
                    let (g, _, d_inv) = Integer::xgcd(&pn, &d);
                    debug_assert_eq!(g, Integer::ONE);
                    let value = pos_int_to_nat(Integer::rem(&(n * d_inv), &pn));
                    PAdicTerminatingRational {
                        p: &self.p,
                        value,
                        k,
                        shift,
                    }
                }
            }
            None => PAdicTerminatingRational {
                p: &self.p,
                value: Natural::ZERO,
                k,
                shift: 0,
            },
        }
    }

    fn shift_by(mut self, k: isize) -> Self {
        self.rat *= Rational::from(&self.p).int_pow(&Integer::from(k)).unwrap();
        self
    }
}

#[derive(Debug, Clone)]
pub struct PAdicTerminatingRational<'a> {
    p: &'a Natural, // A prime number
    value: Natural, // A value modulo p^{k-shift}
    k: isize,
    shift: isize, // A power of p
                  //Together represents p^shift * value
}
impl<'a> PAdicTerminatingRational<'a> {
    pub fn digits(&mut self) -> (Vec<Natural>, isize) {
        let mut digits = padic_digits(self.p, self.value.clone());
        while (digits.len() as isize) < (self.k as isize) - self.shift {
            digits.push(Natural::ZERO);
        }
        (digits, self.shift)
    }

    pub fn natural_and_shift(&self) -> (&Natural, isize) {
        (&self.value, self.shift)
    }

    pub fn natural(&self) -> Result<Natural, ()> {
        let (n, s) = self.natural_and_shift();
        if s < 0 {
            Err(())
        } else {
            let s = s as usize;
            let m = n * self.p.nat_pow(&Natural::from(s));
            Ok(m)
        }
    }
}

/// Store an algebraic p-adic number
#[derive(Debug, Clone)]
pub enum PAdicAlgebraic {
    Rational(PAdicRational),
    Algebraic(PAdicAlgebraicRoot),
}
impl From<PAdicIntegerAlgebraicRoot> for PAdicAlgebraic {
    fn from(value: PAdicIntegerAlgebraicRoot) -> Self {
        PAdicAlgebraic::Algebraic(value.into())
    }
}
impl From<PAdicAlgebraicRoot> for PAdicAlgebraic {
    fn from(value: PAdicAlgebraicRoot) -> Self {
        PAdicAlgebraic::Algebraic(value)
    }
}
impl PAdicAlgebraic {
    pub fn p(&self) -> &Natural {
        match self {
            PAdicAlgebraic::Rational(padic_rational) => &padic_rational.p,
            PAdicAlgebraic::Algebraic(padic_algebraic_root) => &padic_algebraic_root.root.p,
        }
    }

    pub fn degree(&self) -> usize {
        match self {
            PAdicAlgebraic::Rational(_) => 1,
            PAdicAlgebraic::Algebraic(root) => root.root.poly.degree().unwrap(),
        }
    }

    pub fn valuation(&self) -> Option<isize> {
        match self {
            PAdicAlgebraic::Rational(x) => padic_rat_valuation(&x.p, x.rat.clone()),
            PAdicAlgebraic::Algebraic(x) => Some(x.shift),
        }
    }

    pub fn shift_by(self, k: isize) -> Self {
        match self {
            PAdicAlgebraic::Rational(padic_rational) => {
                PAdicAlgebraic::Rational(padic_rational.shift_by(k))
            }
            PAdicAlgebraic::Algebraic(padic_algebraic_root) => {
                PAdicAlgebraic::Algebraic(padic_algebraic_root.shift_by(k))
            }
        }
    }

    pub fn reduce_modulo_valuation<'a>(&'a mut self, k: isize) -> PAdicTerminatingRational<'a> {
        match self {
            PAdicAlgebraic::Rational(rational) => rational.reduce_modulo_valuation(k),
            PAdicAlgebraic::Algebraic(root) => root.reduce_modulo_valuation(k),
        }
    }

    pub fn string_repr(&mut self, num_digits: usize) -> String {
        use std::fmt::Write;
        let seps = self.p() >= &Natural::from(10u32);
        let (digits, mut shift) = self.reduce_modulo_valuation(num_digits as isize).digits();
        let mut digits = digits.into_iter().rev().collect::<Vec<_>>();
        while shift > 0 {
            digits.push(Natural::ZERO);
            shift -= 1;
        }
        debug_assert!(shift <= 0);
        let shift = (-shift) as usize;
        let mut s = String::new();
        write!(&mut s, "...").unwrap();
        for (i, d) in digits.into_iter().rev().enumerate().rev() {
            write!(&mut s, "{}", d).unwrap();
            if i != 0 {
                if seps {
                    if i == shift {
                        write!(&mut s, ";").unwrap();
                    } else {
                        write!(&mut s, ",").unwrap();
                    }
                } else {
                    if i == shift {
                        write!(&mut s, ".").unwrap();
                    }
                }
            }
        }
        s
    }
}

impl std::fmt::Display for PAdicAlgebraic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let p = self.p();
        let n = if p < &Natural::from(10u32) { 6 } else { 3 };
        write!(f, "{}", self.clone().string_repr(n))
    }
}

impl Polynomial<Integer> {
    fn all_padic_roots_irreducible(&self, p: &Natural) -> Vec<PAdicAlgebraic> {
        debug_assert!(is_prime(p));
        debug_assert!(self.is_irreducible());

        let f = self.clone();
        let d = f.degree().unwrap();

        debug_assert!(d > 0);
        if d == 1 {
            // Rational root
            let a = f.coeff(1);
            let b = f.coeff(0);
            // f(x) = ax + b
            // root = -b/a
            vec![PAdicAlgebraic::Rational(PAdicRational {
                p: p.clone(),
                rat: -Rational::from_integers(b, a),
            })]
        } else {
            // Algebraic root

            // Apply f(x) -> f(x/p) until the leading coefficient is not divisible by p i.e. is a p-adic integer.
            // Call the resulting polynomial g(x)
            let mut shift = padic_int_valuation(p, f.leading_coeff().unwrap()).unwrap();
            let mut g = f
                .apply_map_with_powers(|(power, coeff)| {
                    coeff * Integer::from(p).nat_pow(&Natural::from(shift * (d - power)))
                })
                .primitive_part()
                .unwrap();

            println!(
                "{:?}",
                (0..(g.degree().unwrap() + 1))
                    .map(|i| padic_int_valuation(p, g.coeff(i)))
                    .collect::<Vec<_>>()
            );

            let dg = g.clone().derivative();

            println!("g = {}", g);

            // Manually lift roots until they uniquely lift to the p-adic integers
            let mut k = 0;
            let mut pk = Natural::ONE;
            let mut ununique_liftable_roots = vec![Natural::ZERO];
            let mut unique_liftable_roots = vec![];
            while !ununique_liftable_roots.is_empty() {
                println!("{:?}", ununique_liftable_roots.len());

                let mut lifted_roots = vec![];
                let mod_pk1 =
                    QuotientStructure::new_ring(Integer::structure(), Integer::from(&pk * p));
                let poly_mod_pk1 = PolynomialStructure::new(mod_pk1.into());
                for root in ununique_liftable_roots {
                    let mut offset = Natural::ZERO;
                    while offset < *p {
                        let possible_lifted_root = &root + &pk * &offset;
                        let g_eval =
                            poly_mod_pk1.evaluate(&g, &Integer::from(&possible_lifted_root));
                        if g_eval == Integer::ZERO {
                            let g_valuation = k + 1;
                            let dg_eval =
                                poly_mod_pk1.evaluate(&dg, &Integer::from(&possible_lifted_root));
                            let dg_valuation = padic_int_valuation(p, dg_eval).unwrap_or(k + 1);
                            if g_valuation > 2 * dg_valuation {
                                let mut padic_int_root = PAdicIntegerAlgebraicRoot {
                                    p: p.clone(),
                                    poly: g.clone(),
                                    dpoly: dg.clone(),
                                    dpoly_valuation: dg_valuation,
                                    approx_root: Integer::from(possible_lifted_root),
                                    k: k + 1,
                                };
                                #[cfg(debug_assertions)]
                                padic_int_root.check().unwrap();
                                if !unique_liftable_roots
                                    .iter_mut()
                                    .any(|unique_liftable_root| {
                                        PAdicIntegerAlgebraicRoot::equal_mut(
                                            unique_liftable_root,
                                            &mut padic_int_root,
                                        )
                                    })
                                {
                                    unique_liftable_roots.push(padic_int_root);
                                }
                            } else {
                                lifted_roots.push(possible_lifted_root);
                            }
                        }
                        offset += Natural::ONE;
                    }
                }
                ununique_liftable_roots = lifted_roots;
                k += 1;
                pk *= p;
            }

            unique_liftable_roots
                .into_iter()
                .map(|root| {
                    PAdicAlgebraicRoot::from(root)
                        .shift_by(-(shift as isize))
                        .into()
                })
                .collect()
        }
    }
    pub fn all_padic_roots(&self, p: &Natural) -> Vec<PAdicAlgebraic> {
        debug_assert!(is_prime(p));
        assert_ne!(self, &Self::zero());
        let factors = self.factor().unwrap();
        let mut roots = vec![];
        for (factor, k) in factors.factors() {
            for root in factor.all_padic_roots_irreducible(p) {
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

impl PAdicRational {
    fn neg(mut self) -> Self {
        self.rat = -&self.rat;
        self
    }
}

impl PAdicAlgebraicRoot {
    fn neg(mut self) -> Self {
        self.root = self.root.neg();
        self
    }

    fn add_rat(mut self, rat: &Rational) -> Self {
        match padic_rat_valuation(&self.root.p, rat.clone()) {
            Some(rat_valuation) => {
                /*
                  a + p^i b
                = p^-i p^i a + p^i b
                = p^i (p^-i a + b)
                */
                let p = Rational::from(&self.root.p);
                let shifted_rat = rat * p.int_pow(&Integer::from(-self.shift)).unwrap();
                // Want to add shifted_rat to self.root
                let shifted_rat_valuation = rat_valuation - self.shift;
                if shifted_rat_valuation < 0 {
                    self.shift += shifted_rat_valuation;
                    self.root = self.root.leftshift((-shifted_rat_valuation) as usize);
                    self.root = self.root.add_rat(
                        &(shifted_rat * p.int_pow(&Integer::from(-shifted_rat_valuation)).unwrap()),
                    );
                } else {
                    self.root = self.root.add_rat(&shifted_rat);
                }
            }
            None => {
                debug_assert_eq!(rat, &Rational::ZERO);
            }
        }
        self
    }
}

impl PAdicAlgebraic {
    pub fn neg(mut self) -> Self {
        self = match self {
            PAdicAlgebraic::Rational(padic_rational) => {
                PAdicAlgebraic::Rational(padic_rational.neg())
            }
            PAdicAlgebraic::Algebraic(padic_algebraic_root) => {
                PAdicAlgebraic::Algebraic(padic_algebraic_root.neg())
            }
        };
        self
    }
}

impl FieldStructure for PAdicAlgebraicStructure {}

#[cfg(test)]
mod tests {
    use crate::structure::elements::IntoErgonomic;

    use super::*;


    #[test]
    fn test_valid_padic_root_and_refine() {
        // Set up a 7-adic square-root of 2
        let poly =
            Polynomial::from_coeffs(vec![Integer::from(-2), Integer::from(0), Integer::from(1)]);
        let mut root = PAdicIntegerAlgebraicRoot {
            p: Natural::from(7u32),
            poly: poly.clone(),
            dpoly: poly.derivative(),
            dpoly_valuation: 0,
            approx_root: Integer::from(3),
            k: 1,
        };
        // Test refining it
        root.check().unwrap();
        root.refine();
        root.check().unwrap();
        root.refine();
        root.check().unwrap();
        root.refine();
        root.check().unwrap();
        root.refine();
        root.check().unwrap();
        println!("{:?}", root);

        debug_assert_eq!(root.reduce_modulo_valuation(0), Integer::from(0));
        debug_assert_eq!(root.reduce_modulo_valuation(1), Integer::from(3));
        debug_assert_eq!(root.reduce_modulo_valuation(2), Integer::from(10));
        debug_assert_eq!(root.reduce_modulo_valuation(3), Integer::from(108));
        debug_assert_eq!(
            root.reduce_modulo_valuation(20),
            Integer::from(75182500718243698u64)
        );
    }

    #[test]
    fn test_padic_root_rightshift() {
        let poly =
            Polynomial::from_coeffs(vec![Integer::from(-98), Integer::from(0), Integer::from(1)]);
        let mut root = PAdicIntegerAlgebraicRoot {
            p: Natural::from(7u32),
            poly: poly.clone(),
            dpoly: poly.derivative(),
            dpoly_valuation: 1,
            approx_root: Integer::from(21),
            k: 2,
        };

        let x = root.reduce_modulo_valuation(10);
        println!("{:?}", padic_digits(&root.p, x));
        root.check().unwrap();
        assert_eq!(root.reduce_modulo_valuation(0), Integer::from(0));
        assert_eq!(root.reduce_modulo_valuation(1), Integer::from(0));
        assert_eq!(root.reduce_modulo_valuation(2), Integer::from(21));
        assert_eq!(root.reduce_modulo_valuation(3), Integer::from(70));
        assert_eq!(root.reduce_modulo_valuation(4), Integer::from(756));
        root.check().unwrap();
        assert_eq!(
            PAdicAlgebraic::from(root.clone())
                .reduce_modulo_valuation(10)
                .digits(),
            (
                vec![3, 1, 2, 6, 1, 2, 1, 2, 4]
                    .into_iter()
                    .map(|x| (x as u32).into())
                    .collect(),
                1
            )
        );

        let (mut rshift_root, k) = root.clone().fully_rightshift();
        assert_eq!(k, 1);

        let x = rshift_root.reduce_modulo_valuation(10);
        println!("{:?}", padic_digits(&rshift_root.p, x));
        rshift_root.check().unwrap();
        assert_eq!(rshift_root.reduce_modulo_valuation(0), Integer::from(0));
        assert_eq!(rshift_root.reduce_modulo_valuation(1), Integer::from(3));
        assert_eq!(rshift_root.reduce_modulo_valuation(2), Integer::from(10));
        assert_eq!(rshift_root.reduce_modulo_valuation(3), Integer::from(108));
        assert_eq!(rshift_root.reduce_modulo_valuation(4), Integer::from(2166));
        rshift_root.check().unwrap();

        assert!(rshift_root.rightshift().is_none());
    }



}
