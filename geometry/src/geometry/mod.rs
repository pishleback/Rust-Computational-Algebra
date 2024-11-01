use std::borrow::Borrow;
use std::hash::Hash;
use std::rc::Rc;

use orthoclase_rings::ring_structure::structure::{FieldStructure, OrderedRingStructure};

mod coordinates;
pub use coordinates::*;

mod linear_space;
pub use linear_space::*;

mod affine_subspace;
pub use affine_subspace::*;

pub mod simplexes;

// pub mod drawing;