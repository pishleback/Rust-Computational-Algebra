use std::collections::BTreeSet;

use super::normal_subgroup::*;
use super::subgroup::*;
use super::group::*;
use std::hash::Hash;

pub struct Subset<'a> {
    pub group: &'a Group,
    pub elems: BTreeSet<usize>,
}

impl<'a> IntoIterator for Subset<'a> {
    type Item = usize;
    type IntoIter = <BTreeSet<usize> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.elems.into_iter()
    }
}

impl<'a> PartialEq for Subset<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.elems == other.elems
    }
}

impl<'a> Eq for Subset<'a> {}

impl<'a> Hash for Subset<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.elems.hash(state);
    }
}

impl<'a> Clone for Subset<'a> {
    fn clone(&self) -> Self {
        Self {
            group: self.group,
            elems: self.elems.clone(),
        }
    }
}

impl<'a> Subset<'a> {
    pub fn check_state(&self) -> Result<(), &'static str> {
        for x in &self.elems {
            if !(*x < self.group.n) {
                return Err("invalid subset element");
            }
        }

        Ok(())
    }

    pub fn size(&self) -> usize {
        self.elems.len()
    }

    pub fn add_elem(&mut self, x: usize) -> Result<(), ()> {
        if self.elems.contains(&x) {
            return Ok(());
        } else if !(x < self.group.n) {
            return Err(());
        }
        self.elems.insert(x);
        Ok(())
    }

    pub fn left_mul(&self, g: usize) -> Result<Subset, ()> {
        if !(g < self.group.n) {
            return Err(());
        }
        Ok(Subset {
            group: self.group,
            elems: self.elems.iter().map(|h| self.group.mul[g][*h]).collect(),
        })
    }

    pub fn right_mul(&self, g: usize) -> Result<Subset, ()> {
        if !(g < self.group.n) {
            return Err(());
        }
        Ok(Subset {
            group: self.group,
            elems: self.elems.iter().map(|h| self.group.mul[*h][g]).collect(),
        })
    }

    pub fn is_subgroup(&self) -> bool {
        for x in &self.elems {
            for y in &self.elems {
                if !self.elems.contains(&self.group.mul[*x][*y]) {
                    return false;
                }
            }
        }
        true
    }

    pub fn to_subgroup(&self) -> Option<Subgroup> {
        match self.is_subgroup() {
            true => Some(Subgroup {
                subset: self.clone(),
            }),
            false => None,
        }
    }

    pub fn generated_subgroup(&self) -> Result<Subgroup<'a>, ()> {
        for g in &self.elems {
            if !(*g < self.group.size()) {
                return Err(());
            }
        }

        //generate subgroup by adding all generated elements
        let mut sg = BTreeSet::new();
        sg.insert(self.group.ident);

        let mut boundary = vec![self.group.ident];
        let mut next_boundary = vec![];
        let mut y;
        while boundary.len() > 0 {
            for x in &boundary {
                for g in &self.elems {
                    y = self.group.mul[*x][*g];
                    if !sg.contains(&y) {
                        sg.insert(y);
                        next_boundary.push(y);
                    }
                }
            }
            boundary = next_boundary.clone();
            next_boundary = vec![];
        }
        Ok(Subgroup {
            subset: Subset {
                group: self.group,
                elems: sg,
            },
        })
    }

    pub fn normal_closure(&self) -> Result<NormalSubgroup<'a>, &'static str> {
        for g in &self.elems {
            if !(*g < self.group.size()) {
                return Err("gen out of range");
            }
        }

        let conj_info = self.group.conjugacy_classes().state;

        //generate subgroup by adding all generated elements
        let mut sg = BTreeSet::new();
        sg.insert(self.group.ident);

        let mut boundary = vec![self.group.ident];
        let mut next_boundary = vec![];
        while boundary.len() > 0 {
            for x in &boundary {
                for g in &self.elems {
                    for y in &conj_info.classes[conj_info.lookup[self.group.mul[*x][*g]]] {
                        if !sg.contains(y) {
                            sg.insert(*y);
                            next_boundary.push(*y);
                        }
                    }
                }
            }
            boundary = next_boundary.clone();
            next_boundary = vec![];
        }
        Ok(NormalSubgroup {
            subgroup: Subgroup {
                subset: Subset {
                    group: self.group,
                    elems: sg,
                },
            },
        })
    }
}

#[cfg(test)]
mod subset_tests {
    use super::super::super::permutations::*;
    use super::*;

    #[test]
    fn subset_left_mul() {
        let (grp, _perms, elems) = symmetric_group_structure(4);

        let subset = Subset {
            group: &grp,
            elems: vec![
                elems[&Permutation::new(vec![1, 2, 3, 0]).unwrap()],
                elems[&Permutation::new(vec![0, 1, 3, 2]).unwrap()],
                elems[&Permutation::new(vec![3, 2, 1, 0]).unwrap()],
            ]
            .into_iter()
            .collect(),
        };
        let left_mul_subset = subset
            .left_mul(elems[&Permutation::new(vec![1, 2, 0, 3]).unwrap()])
            .unwrap();
        assert_eq!(
            left_mul_subset.elems,
            Subset {
                group: &grp,
                elems: vec![
                    elems[&Permutation::new(vec![2, 0, 3, 1]).unwrap()],
                    elems[&Permutation::new(vec![1, 2, 3, 0]).unwrap()],
                    elems[&Permutation::new(vec![3, 0, 2, 1]).unwrap()]
                ]
                .into_iter()
                .collect()
            }
            .elems
        );
    }

    #[test]
    fn subset_right_mul() {
        let (grp, _perms, elems) = symmetric_group_structure(4);

        let subset = Subset {
            group: &grp,
            elems: vec![
                elems[&Permutation::new(vec![1, 2, 3, 0]).unwrap()],
                elems[&Permutation::new(vec![0, 1, 3, 2]).unwrap()],
                elems[&Permutation::new(vec![3, 2, 1, 0]).unwrap()],
            ]
            .into_iter()
            .collect(),
        };
        let left_mul_subset = subset
            .right_mul(elems[&Permutation::new(vec![1, 2, 0, 3]).unwrap()])
            .unwrap();
        assert_eq!(
            left_mul_subset.elems,
            Subset {
                group: &grp,
                elems: vec![
                    elems[&Permutation::new(vec![2, 3, 1, 0]).unwrap()],
                    elems[&Permutation::new(vec![1, 3, 0, 2]).unwrap()],
                    elems[&Permutation::new(vec![2, 1, 3, 0]).unwrap()]
                ]
                .into_iter()
                .collect()
            }
            .elems
        );
    }
}
