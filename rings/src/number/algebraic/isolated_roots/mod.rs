use malachite_q::Rational;

pub mod complex;
pub mod poly_tools;
pub mod real;
pub mod bisection_gen;

fn rat_to_string(a: Rational) -> String {
    if a == 0 {
        return "0".into();
    }
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
    b = (1000.0 * b).round() / 1000.0;
    b.to_string()
}
