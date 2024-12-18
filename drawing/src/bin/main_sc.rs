#![allow(dead_code, warnings, unused)]

use std::rc::Rc;

use geometry::AffineSpace;
use geometry::Vector;
use malachite_nz::integer::Integer;
use malachite_nz::natural::Natural;
use malachite_q::arithmetic::traits::Approximate;
use malachite_q::Rational;
use orthoclase_drawing::canvas::canvas2d::*;
use orthoclase_drawing::canvas::Canvas;
use orthoclase_geometry::geometry::simplexes::ConvexHull;
use orthoclase_geometry::geometry::simplexes::OrientationSide;
use orthoclase_geometry::geometry::simplexes::OrientedSimplex;
use orthoclase_geometry::geometry::simplexes::Simplex;
use orthoclase_geometry::geometry::simplexes::SimplicialDisjointUnion;
use orthoclase_geometry::geometry::simplexes::VennResult;
use orthoclase_geometry::*;
use orthoclase_rings::ring_structure::cannonical::*;
use orthoclase_rings::structure::CannonicalStructure;
use orthoclase_rings::structure::StructuredType;
use rand::Rng;

fn main() {
    // let space = AffineSpace::new_linear(Rational::structure(), 2);
    // let p1 = Vector::new(&space, vec![Rational::from(0), Rational::from(0)]);
    // let p2 = Vector::new(&space, vec![Rational::from(1), Rational::from(0)]);
    // let p3 = Vector::new(&space, vec![Rational::from(0), Rational::from(1)]);

    // let s1 = Simplex::new(&space, vec![p1.clone()]).unwrap();
    // let s2 = Simplex::new(&space, vec![p1.clone(), p2.clone()]).unwrap();
    // let s3 = Simplex::new(&space, vec![p1.clone(), p2.clone(), p3.clone()]).unwrap();

    let space = Rc::new(AffineSpace::new_linear(Rational::structure(), 2));

    fn random_point(
        space: Rc<AffineSpace<CannonicalStructure<Rational>>>,
        rad: f64,
    ) -> Vector<CannonicalStructure<Rational>, Rc<AffineSpace<CannonicalStructure<Rational>>>> {
        let mut rng = rand::thread_rng();
        Vector::construct(space.clone(), |i| {
            Rational::from_f64_approx(rng.gen_range(-rad..rad)).approximate(&Natural::from(3u64))
        })
    }

    // let pt1 = Vector::new(space.clone(), vec![Rational::from(0), Rational::from(0)]);
    // let pt2 = Vector::new(space.clone(), vec![Rational::from(0), Rational::from(-1)]);
    // let pt3 = Vector::new(space.clone(), vec![Rational::from(0), Rational::from(1)]);
    // let pt4 = Vector::new(space.clone(), vec![Rational::from(1), Rational::from(0)]);

    // let spx1 = Simplex::new(space.clone(), vec![pt1]).unwrap();
    // let spx2 = Simplex::new(space.clone(), vec![pt2, pt3, pt4]).unwrap();

    // let VennResult {
    //     left: a,
    //     middle: b,
    //     right: c,
    // } = spx1.venn(&spx2);

    let ch1 = ConvexHull::new(
        space.clone(),
        (0..6)
            .map(|i| random_point(space.clone(), (i + 1) as f64))
            .collect(),
    );
    let ch2 = ConvexHull::new(
        space.clone(),
        (0..6)
            .map(|i| random_point(space.clone(), (i + 1) as f64))
            .collect(),
    );
    // let ch3 = ch1.intersect(&ch2);

    let sc1 = ch1.as_simplicial_complex().entire;
    let sc2 = ch2.as_simplicial_complex().entire;
    // let sc3 = ch3.as_simplicial_complex().entire;

    // let VennResult {
    //     left: a,
    //     middle: b,
    //     right: c,0

    // }

    let sc4 = SimplicialDisjointUnion::union(&(&sc1).into(), &(&sc2).into());
    println!("done union");
    let sc5 = sc4
        .clone()
        .refine_to_partial_simplicial_complex()
        .closure_as_simplicial_complex();
    println!("done to sc");
    let sc6 = sc5.clone().simplify();
    println!("done simplify");

    orthoclase_drawing::canvas::canvas2d::Diagram2dCanvas::run(|canvas| {
        canvas.draw(&sc1, (1.0, 0.0, 1.0));
        canvas.draw(&sc2, (0.0, 1.0, 1.0));
        // canvas.draw(&sc3, (1.0, 1.0, 0.0));

        // canvas.draw(&sc5, (1.0, 0.0, 0.0));
        canvas.draw(&sc6, (0.0, 1.0, 0.0));
    });
}
