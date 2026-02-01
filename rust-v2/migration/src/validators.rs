use regex::Regex;

/// Validator function type - returns Ok(()) if valid, Err(message) if invalid
pub type Validator = Box<dyn Fn(&str) -> Result<(), String> + Send>;

/// Creates a validator that checks if value is not empty
pub fn required() -> Validator {
    Box::new(|value: &str| {
        if value.trim().is_empty() {
            Err("This field is required".to_string())
        } else {
            Ok(())
        }
    })
}

/// Creates a validator that checks minimum length
pub fn min_length(min: usize) -> Validator {
    Box::new(move |value: &str| {
        if value.chars().count() < min {
            Err(format!("Minimum length is {}", min))
        } else {
            Ok(())
        }
    })
}

/// Creates a validator that checks maximum length
pub fn max_length(max: usize) -> Validator {
    Box::new(move |value: &str| {
        if value.chars().count() > max {
            Err(format!("Maximum length is {}", max))
        } else {
            Ok(())
        }
    })
}

/// Creates a validator that checks against a regex pattern
pub fn regex(pattern: &str) -> Validator {
    let re = Regex::new(pattern).expect("Invalid regex pattern");
    Box::new(move |value: &str| {
        if re.is_match(value) {
            Ok(())
        } else {
            Err(format!("Value must match pattern: {}", re.as_str()))
        }
    })
}

/// Creates an email validator
pub fn email() -> Validator {
    regex(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
}

/// Creates a validator that checks if value contains only alphanumeric characters
pub fn alphanumeric() -> Validator {
    Box::new(|value: &str| {
        if value.chars().all(|c| c.is_alphanumeric()) {
            Ok(())
        } else {
            Err("Only alphanumeric characters allowed".to_string())
        }
    })
}

/// Creates a custom validator with a specific error message
pub fn custom<F>(f: F, message: impl Into<String>) -> Validator
where
    F: Fn(&str) -> bool + Send + 'static,
{
    let msg = message.into();
    Box::new(
        move |value: &str| {
            if f(value) { Ok(()) } else { Err(msg.clone()) }
        },
    )
}
