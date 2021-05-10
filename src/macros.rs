/// Relative assert on two arrays
///
/// Checks:
///
/// - Are lengths exactly equal?
/// - Are paired values approximately equal?
#[macro_export]
macro_rules! assert_array_relative_eq {
    ($lhs:expr, $rhs:expr) => {
        // Size
        assert_eq!($lhs.len(), $rhs.len());

        // Values
        let it = $lhs.iter().zip($rhs.iter());
        for (l, r) in it {
            approx::assert_relative_eq!(l, r);
        }
    };
}

#[cfg(test)]
mod test_assert_array_relative_eq_macro {
    #[test]
    fn pass_on_empty() {
        let expected: Vec<f64> = vec![];
        let result: Vec<f64> = vec![];
        assert_array_relative_eq!(expected, result);
    }

    #[test]
    fn pass_on_same() {
        let expected: Vec<f64> = vec![1., 2., 3.];
        let result: Vec<f64> = vec![1., 2., 3.];
        assert_array_relative_eq!(expected, result);
    }

    #[test]
    #[should_panic]
    fn fail_on_different() {
        let expected: Vec<f64> = vec![1., 2., 3.];
        let result: Vec<f64> = vec![1., 2., 0.];
        assert_array_relative_eq!(expected, result);
    }

    #[test]
    #[should_panic]
    fn fail_on_different_size() {
        let expected: Vec<f64> = vec![1., 2., 3.];
        let result: Vec<f64> = vec![1., 2.];
        assert_array_relative_eq!(expected, result);
    }
}

/// Relative assert on two arrays of complex numbers
///
/// Checks:
///
/// - Are lengths exactly equal?
/// - Are paired values approximately equal in real and imag?
#[macro_export]
macro_rules! assert_complex_array_relative_eq {
    ($lhs:expr, $rhs:expr) => {
        // Size
        assert_eq!($lhs.len(), $rhs.len());

        // Values
        for (l, r) in $lhs.iter().zip($rhs.iter()) {
            approx::assert_relative_eq!(l.re, r.re);
            approx::assert_relative_eq!(l.im, r.im);
        }
    };
}

#[cfg(test)]
mod test_assert_complex_array_relative_eq {
    use num_complex::Complex;

    #[test]
    fn pass_on_empty() {
        let expected: Vec<Complex<f64>> = vec![];
        let result: Vec<Complex<f64>> = vec![];
        assert_complex_array_relative_eq!(expected, result);
    }

    #[test]
    fn pass_on_same() {
        let expected: Vec<Complex<f64>> = vec![Complex { re: 1., im: 2. }];
        let result: Vec<Complex<f64>> = vec![Complex { re: 1., im: 2. }];
        assert_complex_array_relative_eq!(expected, result);
    }

    #[test]
    #[should_panic]
    fn fail_on_real_different() {
        let expected: Vec<Complex<f64>> = vec![Complex { re: 1., im: 2. }];
        let result: Vec<Complex<f64>> = vec![Complex { re: 2., im: 2. }];
        assert_complex_array_relative_eq!(expected, result);
    }

    #[test]
    #[should_panic]
    fn fail_on_imag_different() {
        let expected: Vec<Complex<f64>> = vec![Complex { re: 1., im: 2. }];
        let result: Vec<Complex<f64>> = vec![Complex { re: 1., im: 1. }];
        assert_complex_array_relative_eq!(expected, result);
    }

    #[test]
    #[should_panic]
    fn fail_on_different_size() {
        let expected: Vec<Complex<f64>> =
            vec![Complex { re: 1., im: 2. }, Complex { re: 1., im: 2. }];
        let result: Vec<Complex<f64>> = vec![Complex { re: 1., im: 2. }];
        assert_complex_array_relative_eq!(expected, result);
    }
}
