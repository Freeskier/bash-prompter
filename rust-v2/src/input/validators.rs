use crate::input::Validator;

/// Walidator wymagający niepustej wartości
pub fn required() -> Validator {
    Box::new(|value: &str| {
        if value.trim().is_empty() {
            Err("This field is required".to_string())
        } else {
            Ok(())
        }
    })
}

/// Walidator minimalnej długości
pub fn min_length(n: usize) -> Validator {
    Box::new(move |value: &str| {
        if value.chars().count() < n {
            Err(format!("Minimum {} characters required", n))
        } else {
            Ok(())
        }
    })
}

/// Walidator maksymalnej długości
pub fn max_length(n: usize) -> Validator {
    Box::new(move |value: &str| {
        if value.chars().count() > n {
            Err(format!("Maximum {} characters allowed", n))
        } else {
            Ok(())
        }
    })
}

/// Walidator zakresu długości
pub fn length_range(min: usize, max: usize) -> Validator {
    Box::new(move |value: &str| {
        let len = value.chars().count();
        if len < min {
            Err(format!("Minimum {} characters required", min))
        } else if len > max {
            Err(format!("Maximum {} characters allowed", max))
        } else {
            Ok(())
        }
    })
}

// Walidator wyrażenia regularnego - wymaga dodania `regex` do dependencies w Cargo.toml
// #[cfg(feature = "regex")]
// pub fn regex(pattern: &str) -> Validator {
//     let re = regex::Regex::new(pattern).expect("Invalid regex pattern");
//     Box::new(move |value: &str| {
//         if re.is_match(value) {
//             Ok(())
//         } else {
//             Err("Invalid format".to_string())
//         }
//     })
// }

/// Walidator emaila (prosty)
pub fn email() -> Validator {
    Box::new(|value: &str| {
        if value.contains('@') && value.contains('.') {
            Ok(())
        } else {
            Err("Invalid email address".to_string())
        }
    })
}

/// Walidator numeryczny
pub fn numeric() -> Validator {
    Box::new(|value: &str| {
        if value.is_empty() || value.chars().all(|c| c.is_numeric()) {
            Ok(())
        } else {
            Err("Only numbers allowed".to_string())
        }
    })
}

/// Walidator alfanumeryczny
pub fn alphanumeric() -> Validator {
    Box::new(|value: &str| {
        if value.is_empty() || value.chars().all(|c| c.is_alphanumeric()) {
            Ok(())
        } else {
            Err("Only letters and numbers allowed".to_string())
        }
    })
}

/// Walidator dostosowany (custom)
pub fn custom<F>(f: F, error_message: impl Into<String>) -> Validator
where
    F: Fn(&str) -> bool + 'static,
{
    let msg = error_message.into();
    Box::new(
        move |value: &str| {
            if f(value) { Ok(()) } else { Err(msg.clone()) }
        },
    )
}
