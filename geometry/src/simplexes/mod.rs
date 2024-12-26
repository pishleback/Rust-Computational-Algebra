use super::*;

// It is helpful for computational reasons to put an ordering on the vectors
// so that the points of a simplex can be ordered
impl<FS: OrderedRingStructure + FieldStructure, SP: Borrow<AffineSpace<FS>> + Clone> PartialOrd
    for Vector<FS, SP>
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let space = common_space(
            self.ambient_space().borrow(),
            other.ambient_space().borrow(),
        )?;
        for i in 0..space.linear_dimension().unwrap() {
            match space
                .ordered_field()
                .ring_cmp(self.coordinate(i), other.coordinate(i))
            {
                std::cmp::Ordering::Less => {
                    return Some(std::cmp::Ordering::Less);
                }
                std::cmp::Ordering::Equal => {}
                std::cmp::Ordering::Greater => {
                    return Some(std::cmp::Ordering::Greater);
                }
            }
        }
        Some(std::cmp::Ordering::Equal)
    }
}
impl<FS: OrderedRingStructure + FieldStructure, SP: Borrow<AffineSpace<FS>> + Clone> Ord
    for Vector<FS, SP>
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.partial_cmp(other) {
            Some(ans) => ans,
            None => panic!(),
        }
    }
}

mod simplex;
pub use simplex::*;

mod convex_hull;
pub use convex_hull::*;

mod simplicial_complex;
pub use simplicial_complex::*;