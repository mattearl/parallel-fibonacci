use num_bigint::BigUint;
use num_traits::{One, Zero};

// Matrix structure for 2x2 matrices
#[derive(Clone)]
pub struct Matrix {
    pub a: BigUint,
    pub b: BigUint,
    pub c: BigUint,
    pub d: BigUint,
}

// Matrix multiplication for 2x2 matrices
pub fn matrix_mult(m1: &Matrix, m2: &Matrix) -> Matrix {
    Matrix {
        a: &m1.a * &m2.a + &m1.b * &m2.c,
        b: &m1.a * &m2.b + &m1.b * &m2.d,
        c: &m1.c * &m2.a + &m1.d * &m2.c,
        d: &m1.c * &m2.b + &m1.d * &m2.d,
    }
}

// Matrix exponentiation using squaring (O(log n))
pub fn matrix_pow(mut base: Matrix, mut exp: usize) -> Matrix {
    let mut result = Matrix {
        a: BigUint::one(),
        b: BigUint::zero(),
        c: BigUint::zero(),
        d: BigUint::one(),
    }; // Identity matrix

    while exp > 0 {
        if exp % 2 == 1 {
            result = matrix_mult(&result, &base);
        }
        base = matrix_mult(&base, &base);
        exp /= 2;
    }

    result
}
