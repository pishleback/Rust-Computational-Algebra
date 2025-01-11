## Crates
Algebraeon is published under the four separate crates.
 - [algebraeon-sets](https://crates.io/crates/algebraeon-sets)
 - [algebraeon-groups](https://crates.io/crates/algebraeon-groups)
 - [algebraeon-rings](https://crates.io/crates/algebraeon-rings)
 - [algebraeon-geometry](https://crates.io/crates/algebraeon-geometry)

# Algebraeon
Algebraeon is a computer algebra system written purely in Rust. It implements algorithms for working with matrices, polynomials, algebraic numbers, factorizations, etc. The focus is on exact algebraic computations over approximate numerical solutions. Algebraeon is in early stages of development and the API is currently highly unstable and subject to change.

As a taste for the sorts of problems Algebraeon solves, it already implements the following:
 - Euclids algorithm for GCD and the extended version for obtaining Bezout coefficients.
 - Polynomial GCD computations using subresultant pseudo-remainder sequences.
 - AKS algorithm for natural number primality testing.
 - Matrix algorithms including:
   - Putting a matrix into Hermite normal form. In particular putting it into echelon form.
   - Putting a matrix into Smith normal form.
   - Gram–Schmidt algorithm for orthogonalization and orthonormalization.
   - Putting a matrix into Jordan normal.
   - Finding the general solution to a linear or affine system of equations.
 - Polynomial factoring algorithms including:
   - Kronecker's method for factoring polynomials over the integers (slow).
   - Berlekamp-Zassenhaus algorithm for factoring polynomials over the integers.
   - Berlekamp's algorithm for factoring polynomials over finite fields.
   - Trager's algorithm for factoring polynomials over algebraic number fields.
 - Expressing symmetric polynomials in terms of elementary symmetric polynomials.
 - Computations with algebraic numbers:
   - Real root isolation and arithmetic.
   - Complex root isolation and arithmetic.
 - Computations with multiplication tables for small finite groups.
 - Todd-Coxeter algorithm for the enumeration of finite index cosets of a finitely generated groups.

# Example Usage
## Factoring a Polynomial
Factor the polynomials $x^2 - 5x + 6$ and $x^{15} - 1$.
```
use algebraeon_rings::{
    polynomial::polynomial::*,
    ring_structure::{elements::*, structure::*},
};
use malachite_nz::integer::Integer;

let x = &Polynomial::<Integer>::var().into_ergonomic();
let f = (x.pow(2) - 5*x + 6).into_verbose();
println!("f = {}", f.factor().unwrap());
/*
Output:
    f = 1 * ((-2)+λ) * ((-3)+λ)
*/

let f = (x.pow(15) - 1).into_verbose();
println!("f = {}", f.factor().unwrap());
/*
Output:
    f = 1 * ((-1)+λ) * (1+λ+λ^2) * (1+λ+λ^2+λ^3+λ^4) * (1+(-1)λ+λ^3+(-1)λ^4+λ^5+(-1)λ^7+λ^8)
*/
```
so
```math
x^2 - 5x + 6 = (x-2)(x-3)
```
and
```math
x^{15}-1 = (x-1)(x^2+x+1)(x^4+x^3+x^2+x+1)(x^8-x^7+x^5-x^4+x^3-x+1)
```

## Jordan Normal Form of a Matrix
```
use algebraeon_rings::{linear::matrix::*, number::algebraic::isolated_roots::complex::*};
use algebraeon_sets::structure::*;
use malachite_q::Rational;
// Construct a matrix
let a = Matrix::<Rational>::from_rows(vec![
    vec![5, 4, 2, 1],
    vec![0, 1, -1, -1],
    vec![-1, -1, 3, 0],
    vec![1, 1, -1, 2],
]);
// Put it into Jordan Normal Form
let j = MatrixStructure::new(ComplexAlgebraic::structure()).jordan_normal_form(&a);
j.pprint();
/*
Output:
    / 2    0    0    0 \
    | 0    1    0    0 |
    | 0    0    4    1 |
    \ 0    0    0    4 /
*/
```

## Integer Linear Systems
Find the general solution to the linear system
```math
a \begin{pmatrix}3 \\ 4 \\ 1\end{pmatrix} + b \begin{pmatrix}2 \\ 1 \\ 2\end{pmatrix} + c \begin{pmatrix}1 \\ 3 \\ -1\end{pmatrix} = \begin{pmatrix}5 \\ 5 \\ 3\end{pmatrix}
```
for integers $a$, $b$ and $c$.
```
use algebraeon_rings::linear::matrix::Matrix;
use malachite_nz::integer::Integer;
let a = Matrix::<Integer>::from_rows(vec![vec![3, 4, 1], vec![2, 1, 2], vec![1, 3, -1]]);
let b = Matrix::<Integer>::from_rows(vec![vec![5, 5, 3]]);
let s = a.row_solution_lattice(b);
s.pprint();
/*
Output:
    Start Affine Lattice
    Offset
    ( 2    0    -1 )
    Start Linear Lattice
    ( 1    -1    -1 )
    End Linear Lattice
    End Affine Lattice
*/
```
so the general solution is all $a$, $b$, $c$ such that
```math
\begin{pmatrix}a \\ b \\ c\end{pmatrix} = \begin{pmatrix}2 \\ 0 \\ -1\end{pmatrix} + t\begin{pmatrix}1 \\ -1 \\ -1\end{pmatrix}
```
for some integer $t$.


# Getting Help
If you have questions, concerns, bug reports, etc, please file an issue in this repository's Issue Tracker.

# Contributing
Contributions are welcome. There are two primary ways to contribute:

## Using the issue tracker
Use the issue tracker to suggest feature requests, report bugs, and ask questions.

## Changing the code-base
You should fork this repository, make changes in your own fork, and then submit a pull request. New code should have associated unit tests that validate implemented features and the presence or lack of defects.

Algebraeon is organized as a [cargo workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html). Run `cargo test` in the root directory to build and run all tests.

A suggested workflow for testing new features:
 - Create a new binary in `examples/src/bin`, for example `my_main.rs`.
 - To run, use `cargo run --bin my_main.rs` in the root directory.
 - Test any changes to the codebase with unit tests and/or using `my_main.rs`.
