//! Module for utility functions that may be useful across the entire application.

use rand::{distributions::Alphanumeric, Rng};

use std::iter;

/// Generates a random alphanumeric string of a certain length.
///
/// # Arguments
///
/// * `length` - The length of the random string to generate
/// * `only_uppercase` - If true, returns only upper case letters. Otherwise, returns
///                    mixed-case strings.
///
/// # Returns
///
/// * `String` - A random alphanumeric string
pub fn generate_random_string(length: usize, only_uppercase: bool) -> String {
    let mut rng = rand::thread_rng();
    let random_string = iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(length)
        .collect::<String>();
    if only_uppercase {
        return random_string.to_uppercase();
    }
    random_string
}
