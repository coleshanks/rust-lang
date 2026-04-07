pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

pub fn greeting(name: &str) -> String {
    format!("Hello {name}!")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exploration() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
    
    #[test]
    fn greeting_contains_name() {
        let result = greeting("Emma");
        assert!(
            result.contains("Carol"),
            "Greeting did not contain \"Carol!\", result was {result}"
        );
    }

}
