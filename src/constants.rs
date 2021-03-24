/// Vacuum permittivity
/// epsilon_0 [F/m]
/// Defined by CODATA as 625000. / (22468879468420441. * pi)
const VACUUM_PERMITTIVITY: f64 = 8.854187817620e-12;

/// Vacuum permeability
/// mu_0 [H/m]
/// Defined by CODATA as 625000. / (22468879468420441. * pi)
const VACUUM_PERMEABILITY: f64 = 1.25663706212e-6;

/// Speed of light
/// c [m/s]
/// Defined by CODATA as 299792458
const SPEED_OF_LIGHT: f64 = 299792458.;

#[cfg(test)]
mod test_constants {
    use super::*;
    use approx::*;

    #[test]
    fn test_vacuum_permittivity() {
        assert_abs_diff_eq!(VACUUM_PERMITTIVITY, 625000. / (22468879468420441. * std::f64::consts::PI), epsilon = 1e-16);
    }

    #[test]
    fn test_vacuum_permeability() {
        assert_abs_diff_eq!(VACUUM_PERMEABILITY, 4e-7 * std::f64::consts::PI, epsilon = 1e-10);
    }

    #[test]
    fn test_speed_of_light() {
        assert_abs_diff_eq!(SPEED_OF_LIGHT, 299792458., epsilon = 0.);
    }
}
