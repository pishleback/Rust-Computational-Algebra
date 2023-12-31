use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;
use std::sync::atomic::AtomicUsize;

use super::nzq::*;
use super::poly::*;
use super::ring::*;

pub const ZZ_MULTIPOLY: MultiPolynomialRing<IntegerRing> = MultiPolynomialRing { ring: &ZZ };
pub const QQ_MULTIPOLY: MultiPolynomialRing<RationalField> = MultiPolynomialRing { ring: &QQ };

#[derive(Debug, Hash, Clone)]
pub struct Variable {
    ident: usize,
    name: String,
}

impl PartialEq for Variable {
    fn eq(&self, other: &Self) -> bool {
        self.ident == other.ident
    }
}

impl Eq for Variable {}

impl Variable {
    pub fn new(name: String) -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        assert!(name.len() >= 1);
        Self {
            ident: COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            name,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VariablePower {
    var: Variable,
    pow: usize,
}

#[derive(Debug, Clone)]
pub struct Monomial {
    prod: Vec<VariablePower>,            //should be sorted by variable ident
    ident_lookup: HashMap<usize, usize>, //point from variable ident to index of that variables power in self.prod
}

impl PartialEq for Monomial {
    fn eq(&self, other: &Self) -> bool {
        self.prod == other.prod
    }
}

impl Eq for Monomial {}

impl Hash for Monomial {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.prod.hash(state);
    }
}

impl ToString for Monomial {
    fn to_string(&self) -> String {
        if self.prod.len() == 0 {
            String::from("1")
        } else {
            let mut ans = String::from("");
            for VariablePower { var, pow } in &self.prod {
                ans += &var.name;
                ans += "^";
                ans += pow.to_string().as_str();
            }
            ans
        }
    }
}

impl Monomial {
    fn check_invariants(&self) -> Result<(), &'static str> {
        let mut vars = HashSet::new();
        for VariablePower { var, pow } in &self.prod {
            if pow == &0 {
                return Err("shouldn't have a variable to the power of zero");
            }
            if vars.contains(var) {
                return Err("each var should appear at most once");
            }
            vars.insert(var);
        }
        for (ident, idx) in &self.ident_lookup {
            if &self.prod[*idx].var.ident != ident {
                return Err("bad ident_lookup");
            }
        }
        let mut ordered_prod = self.prod.clone();
        ordered_prod.sort_by_key(|VariablePower { var, pow: _pow }| var.ident);
        if self.prod != ordered_prod {
            return Err("var powers are not sorted");
        }
        Ok(())
    }

    fn new(mut prod: Vec<VariablePower>) -> Self {
        prod = prod
            .into_iter()
            .filter(|VariablePower { var: _var, pow }| pow != &0)
            .collect();
        prod.sort_by_key(|vpow| vpow.var.ident);
        let mut ident_lookup = HashMap::new();
        for (idx, VariablePower { var, pow: _pow }) in prod.iter().enumerate() {
            ident_lookup.insert(var.ident, idx);
        }
        Self { prod, ident_lookup }
    }

    fn one() -> Self {
        Monomial {
            prod: vec![],
            ident_lookup: HashMap::new(),
        }
    }

    fn degree(&self) -> usize {
        let mut d = 0;
        for VariablePower { var: _var, pow } in &self.prod {
            d += pow;
        }
        d
    }

    fn homogenize(&self, target_degree: usize, v: &Variable) -> Self {
        let self_degree = self.degree();
        if target_degree >= self_degree {
            let mut prod = self.prod.clone();
            prod.push(VariablePower {
                var: v.clone(),
                pow: target_degree - self_degree,
            });
            Self::new(prod)
        } else {
            panic!();
        }
    }

    fn get_var_pow(&self, v: &Variable) -> usize {
        if self.ident_lookup.contains_key(&v.ident) {
            self.prod[*self.ident_lookup.get(&v.ident).unwrap()].pow
        } else {
            0
        }
    }

    fn free_vars(&self) -> HashSet<Variable> {
        self.prod
            .iter()
            .map(|VariablePower { var, pow: _pow }| var.clone())
            .collect()
    }

    fn mul(a: &Self, b: &Self) -> Self {
        Self::new({
            let mut prod = HashMap::new();
            for VariablePower { var: v, pow: k } in &a.prod {
                *prod.entry(v.clone()).or_insert(0) += k;
            }
            for VariablePower { var: v, pow: k } in &b.prod {
                *prod.entry(v.clone()).or_insert(0) += k;
            }
            let prod: Vec<VariablePower> = prod
                .into_iter()
                .map(|(v, k)| VariablePower { var: v, pow: k })
                .collect();
            prod
        })
    }

    fn lexicographic_order(a: &Self, b: &Self) -> std::cmp::Ordering {
        let mut i = 0;
        while i < std::cmp::min(a.prod.len(), b.prod.len()) {
            if a.prod[i].var.ident < b.prod[i].var.ident {
                return std::cmp::Ordering::Less;
            } else if a.prod[i].var.ident > b.prod[i].var.ident {
                return std::cmp::Ordering::Greater;
            } else {
                if a.prod[i].pow > b.prod[i].pow {
                    return std::cmp::Ordering::Less;
                } else if a.prod[i].pow < b.prod[i].pow {
                    return std::cmp::Ordering::Greater;
                } else {
                    i += 1;
                }
            }
        }
        if a.prod.len() > b.prod.len() {
            return std::cmp::Ordering::Less;
        } else if a.prod.len() < b.prod.len() {
            return std::cmp::Ordering::Greater;
        } else {
            return std::cmp::Ordering::Equal;
        }
    }

    fn graded_lexicographic_order(a: &Self, b: &Self) -> std::cmp::Ordering {
        if a.degree() < b.degree() {
            std::cmp::Ordering::Greater
        } else if a.degree() > b.degree() {
            std::cmp::Ordering::Less
        } else {
            Self::lexicographic_order(a, b)
        }
    }
}

#[derive(Debug, Clone)]
pub struct Term<ElemT: Clone> {
    coeff: ElemT,
    monomial: Monomial,
}

impl<ElemT: Clone> Term<ElemT> {
    fn check_invariants(&self) -> Result<(), &'static str> {
        self.monomial.check_invariants()
    }
}

#[derive(Debug, Clone)]
pub struct MultiPolynomial<ElemT: Clone> {
    terms: Vec<Term<ElemT>>, //sorted by monomial ordering
}

impl<ElemT: Clone> MultiPolynomial<ElemT> {
    fn new(mut terms: Vec<Term<ElemT>>) -> Self {
        terms.sort_by(|t1, t2| Monomial::lexicographic_order(&t1.monomial, &t2.monomial));
        Self { terms }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MultiPolynomialRing<'a, R: ComRing> {
    ring: &'a R,
}

impl<'a, R: ComRing> MultiPolynomialRing<'a, R> {}

impl<'a, R: ComRing> ComRing for MultiPolynomialRing<'a, R> {
    type ElemT = MultiPolynomial<R::ElemT>;

    fn to_string(&self, elem: &Self::ElemT) -> String {
        if elem.terms.len() == 0 {
            String::from("0")
        } else {
            let mut ans = String::from("");
            for (idx, term) in elem.terms.iter().enumerate() {
                if idx != 0 {
                    ans += "+";
                }
                ans += "(";
                ans += self.ring.to_string(&term.coeff).as_str();
                ans += ")";
                ans += term.monomial.to_string().as_str();
            }
            ans
        }
    }

    fn equal(&self, a: &Self::ElemT, b: &Self::ElemT) -> bool {
        let n = a.terms.len();

        if n != b.terms.len() {
            false
        } else {
            (0..n).all(|i| {
                self.ring.equal(&a.terms[i].coeff, &b.terms[i].coeff)
                    && a.terms[i].monomial == b.terms[i].monomial
            })
        }
    }

    fn zero(&self) -> Self::ElemT {
        MultiPolynomial { terms: vec![] }
    }

    fn one(&self) -> Self::ElemT {
        MultiPolynomial {
            terms: vec![Term {
                coeff: self.ring.one(),
                monomial: Monomial::one(),
            }],
        }
    }

    fn neg_mut(&self, elem: &mut Self::ElemT) {
        for Term {
            coeff,
            monomial: _monomial,
        } in &mut elem.terms
        {
            self.ring.neg_mut(coeff);
        }
    }

    fn add_mut(&self, elem: &mut Self::ElemT, offset: &Self::ElemT) {
        elem.clone_from(&self.add(elem.clone(), offset.clone()))
    }

    fn add(&self, mut elem: Self::ElemT, offset: Self::ElemT) -> Self::ElemT {
        let mut existing_monomials: HashMap<Monomial, usize> = HashMap::new(); //the index of each monomial
        for (
            idx,
            Term {
                coeff: _coeff,
                monomial,
            },
        ) in elem.terms.clone().into_iter().enumerate()
        {
            existing_monomials.insert(monomial, idx);
        }
        for Term { coeff, monomial } in offset.terms {
            if existing_monomials.contains_key(&monomial) {
                self.ring.add_mut(
                    &mut elem.terms[*existing_monomials.get(&monomial).unwrap()].coeff,
                    &coeff,
                );
            } else {
                elem.terms.push(Term { coeff, monomial });
            }
        }
        MultiPolynomial::new(
            elem.terms
                .into_iter()
                .filter(|term| !self.ring.equal(&term.coeff, &self.ring.zero()))
                .collect(),
        ) //sort the coeffs
    }

    fn mul_mut(&self, elem: &mut Self::ElemT, offset: &Self::ElemT) {
        elem.clone_from(&self.mul_refs(&elem, &offset))
    }

    fn mul_refs(&self, a: &Self::ElemT, b: &Self::ElemT) -> Self::ElemT {
        let mut terms: HashMap<Monomial, R::ElemT> = HashMap::new();
        for Term {
            coeff: a_coeff,
            monomial: a_monomial,
        } in &a.terms
        {
            for Term {
                coeff: b_coeff,
                monomial: b_monomial,
            } in &b.terms
            {
                let mon = Monomial::mul(a_monomial, b_monomial);
                let coeff = self.ring.mul_refs(a_coeff, b_coeff);
                self.ring
                    .add_mut(terms.entry(mon).or_insert(self.ring.zero()), &coeff);
            }
        }
        MultiPolynomial::new(
            terms
                .into_iter()
                .filter(|(_monomial, coeff)| !self.ring.equal(coeff, &self.ring.zero()))
                .map(|(monomial, coeff)| Term { coeff, monomial })
                .collect(),
        )
    }

    fn div(&self, a: Self::ElemT, b: Self::ElemT) -> Result<Self::ElemT, RingDivisionError> {
        let mut vars = HashSet::new();
        vars.extend(self.free_vars(&a));
        vars.extend(self.free_vars(&b));
        if vars.len() == 0 {
            //a and b are constants
            debug_assert!(a.terms.len() <= 1);
            debug_assert!(b.terms.len() <= 1);
            if b.terms.len() == 0 {
                Err(RingDivisionError::DivideByZero)
            } else if a.terms.len() == 0 {
                Ok(self.zero())
            } else {
                debug_assert!(a.terms.len() == 1);
                debug_assert!(b.terms.len() == 1);
                match self.ring.div_refs(&a.terms[0].coeff, &b.terms[0].coeff) {
                    Ok(c) => Ok(self.constant(c)),
                    Err(RingDivisionError::NotDivisible) => Err(RingDivisionError::NotDivisible),
                    Err(RingDivisionError::DivideByZero) => panic!(),
                }
            }
        } else {
            let var = vars.iter().next().unwrap();
            let a_poly = self.expand(&a, var);
            let b_poly = self.expand(&b, var);
            match PolynomialRing::new(self).div(a_poly, b_poly) {
                Ok(c_poly) => {
                    Ok(PolynomialRing::new(self).evaluate(&c_poly, &self.var(var.clone())))
                }

                Err(e) => Err(e),
            }
        }
    }
}

impl<'a, R: IntegralDomain> IntegralDomain for MultiPolynomialRing<'a, R> {}

impl<'a, R: ComRing> MultiPolynomialRing<'a, R> {
    fn check_invariants(&self, poly: MultiPolynomial<R::ElemT>) -> Result<(), &'static str> {
        for term in &poly.terms {
            match term.check_invariants() {
                Ok(()) => {}
                Err(e) => {
                    return Err(e);
                }
            }
            if self.ring.equal(&term.coeff, &self.ring.zero()) {
                return Err("coeff should not be zero");
            }
        }

        if !(0..poly.terms.len() - 1).all(|i| {
            Monomial::lexicographic_order(&poly.terms[i].monomial, &poly.terms[i + 1].monomial)
                .is_le()
        }) {
            return Err("terms are not sorted");
        }

        Ok(())
    }

    pub fn new(ring: &'a R) -> Self {
        Self { ring }
    }

    pub fn var_pow(&self, v: Variable, k: usize) -> MultiPolynomial<R::ElemT> {
        MultiPolynomial {
            terms: vec![Term {
                coeff: self.ring.one(),
                monomial: Monomial::new(vec![VariablePower { var: v, pow: k }]),
            }],
        }
    }

    pub fn var(&self, v: Variable) -> MultiPolynomial<R::ElemT> {
        self.var_pow(v, 1)
    }

    pub fn constant(&self, c: R::ElemT) -> MultiPolynomial<R::ElemT> {
        if self.ring.equal(&c, &self.ring.zero()) {
            self.zero()
        } else {
            MultiPolynomial {
                terms: vec![Term {
                    coeff: c,
                    monomial: Monomial::one(),
                }],
            }
        }
    }

    pub fn as_constant(&self, poly: &MultiPolynomial<R::ElemT>) -> Option<R::ElemT> {
        if poly.terms.len() == 0 {
            Some(self.ring.zero())
        } else if poly.terms.len() == 1 {
            let Term { coeff, monomial } = &poly.terms[0];
            if monomial == &Monomial::one() {
                Some(coeff.clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn degree(&self, poly: &MultiPolynomial<R::ElemT>) -> Option<usize> {
        if poly.terms.len() == 0 {
            None
        } else {
            let mut d = 0;
            for Term {
                coeff: _coeff,
                monomial,
            } in &poly.terms
            {
                d = std::cmp::max(d, monomial.degree())
            }
            Some(d)
        }
    }

    pub fn term(&self, t: Term<R::ElemT>) -> MultiPolynomial<R::ElemT> {
        MultiPolynomial { terms: vec![t] }
    }

    pub fn free_vars(&self, a: &MultiPolynomial<R::ElemT>) -> HashSet<Variable> {
        let mut vars = HashSet::new();
        for term in &a.terms {
            vars.extend(term.monomial.free_vars());
        }
        vars
    }

    pub fn homogenize(
        &self,
        poly: &MultiPolynomial<R::ElemT>,
        v: &Variable,
    ) -> MultiPolynomial<R::ElemT> {
        if self.equal(poly, &self.zero()) {
            self.zero()
        } else {
            let d = self.degree(poly).unwrap();
            let h = MultiPolynomial::new(
                poly.terms
                    .iter()
                    .map(|Term { coeff, monomial }| Term {
                        coeff: coeff.clone(),
                        monomial: monomial.homogenize(d, v),
                    })
                    .collect(),
            );
            debug_assert!(self.check_invariants(h.clone()).is_ok());
            h
        }
    }

    pub fn expand(
        &self,
        a: &MultiPolynomial<R::ElemT>,
        v: &Variable,
    ) -> Polynomial<MultiPolynomial<R::ElemT>> {
        let mut coeffs = vec![];
        for Term { coeff, monomial } in &a.terms {
            let k = monomial.get_var_pow(v);
            while coeffs.len() <= k {
                coeffs.push(self.zero())
            }
            self.add_mut(
                &mut coeffs[k],
                &MultiPolynomial {
                    terms: vec![Term {
                        coeff: coeff.clone(),
                        monomial: Monomial::new(
                            monomial
                                .clone()
                                .prod
                                .into_iter()
                                .filter(|VariablePower { var, pow: _pow }| var != v)
                                .collect(),
                        ),
                    }],
                },
            );
        }
        PolynomialRing::new(self).from_coeffs(coeffs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monomial_ordering() {
        let xv = Variable::new(String::from("x"));
        let yv = Variable::new(String::from("y"));

        let xx = Monomial::new(vec![VariablePower {
            var: xv.clone(),
            pow: 2,
        }]);
        let yy = Monomial::new(vec![VariablePower {
            var: yv.clone(),
            pow: 2,
        }]);
        let xy = Monomial::new(vec![
            VariablePower {
                var: xv.clone(),
                pow: 1,
            },
            VariablePower {
                var: yv.clone(),
                pow: 1,
            },
        ]);
        let x = Monomial::new(vec![VariablePower {
            var: xv.clone(),
            pow: 1,
        }]);
        let y = Monomial::new(vec![VariablePower {
            var: yv.clone(),
            pow: 1,
        }]);
        let one = Monomial::one();

        {
            let terms = vec![
                xx.clone(),
                xy.clone(),
                x.clone(),
                yy.clone(),
                y.clone(),
                one.clone(),
            ];
            let mut sorted_terms = terms.clone();
            sorted_terms.sort_by(|a, b| Monomial::lexicographic_order(a, b));

            assert_eq!(terms, sorted_terms);
        }

        {
            let terms = vec![
                xx.clone(),
                xy.clone(),
                yy.clone(),
                x.clone(),
                y.clone(),
                one.clone(),
            ];
            let mut sorted_terms = terms.clone();
            sorted_terms.sort_by(|a, b| Monomial::graded_lexicographic_order(a, b));

            assert_eq!(terms, sorted_terms);
        }
    }

    #[test]
    fn test_reduction() {
        let x = ZZ_MULTIPOLY.var(Variable::new(String::from("x")));
        let f = ZZ_MULTIPOLY.sum(vec![&x, &ZZ_MULTIPOLY.neg(x.clone())]);
        assert_eq!(f.terms.len(), 0);

        let x = ZZ_MULTIPOLY.var(Variable::new(String::from("x")));
        let y = ZZ_MULTIPOLY.var(Variable::new(String::from("y")));
        let f = ZZ_MULTIPOLY.product(vec![
            &ZZ_MULTIPOLY.sum(vec![&x, &y]),
            &ZZ_MULTIPOLY.sum(vec![&x, &ZZ_MULTIPOLY.neg_ref(&y)]),
        ]);
        println!("{}", ZZ_MULTIPOLY.to_string(&f));
        assert_eq!(f.terms.len(), 2);
    }

    #[test]
    fn test_division() {
        let x = ZZ_MULTIPOLY.var(Variable::new(String::from("x")));
        let y = ZZ_MULTIPOLY.var(Variable::new(String::from("y")));

        let f = ZZ_MULTIPOLY.sum(vec![
            &ZZ_MULTIPOLY.product(vec![&x, &x]),
            &ZZ_MULTIPOLY.neg(ZZ_MULTIPOLY.product(vec![&y, &y])),
        ]);
        let g = ZZ_MULTIPOLY.sum(vec![&x, &ZZ_MULTIPOLY.neg_ref(&y)]);
        match ZZ_MULTIPOLY.div_refs(&f, &g) {
            Ok(h) => {
                assert!(ZZ_MULTIPOLY.equal(&f, &ZZ_MULTIPOLY.mul_refs(&g, &h)));
            }
            Err(RingDivisionError::NotDivisible) => panic!(),
            Err(RingDivisionError::DivideByZero) => panic!(),
        }

        let f = ZZ_MULTIPOLY.sum(vec![
            &ZZ_MULTIPOLY.product(vec![&x, &x]),
            &ZZ_MULTIPOLY.neg(ZZ_MULTIPOLY.product(vec![&y, &y])),
        ]);
        let g = ZZ_MULTIPOLY.sum(vec![]);
        match ZZ_MULTIPOLY.div_refs(&f, &g) {
            Ok(_) => panic!(),
            Err(RingDivisionError::NotDivisible) => panic!(),
            Err(RingDivisionError::DivideByZero) => {}
        }

        let f = ZZ_MULTIPOLY.sum(vec![
            &ZZ_MULTIPOLY.product(vec![&x, &x]),
            &ZZ_MULTIPOLY.neg(ZZ_MULTIPOLY.product(vec![&y, &y])),
        ]);
        let g = ZZ_MULTIPOLY.sum(vec![&x]);
        match ZZ_MULTIPOLY.div_refs(&f, &g) {
            Ok(_) => panic!(),
            Err(RingDivisionError::NotDivisible) => {}
            Err(RingDivisionError::DivideByZero) => panic!(),
        }
    }
}
