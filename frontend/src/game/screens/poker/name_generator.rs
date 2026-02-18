pub struct NameGenerator;

impl NameGenerator {
    pub fn generate_random_name(existing_names: &[String]) -> String {
        let random_names = Self::get_random_name_pool();
        let existing_set: std::collections::HashSet<&str> =
            existing_names.iter().map(|s| s.as_str()).collect();

        // Try to find a name that's not already used
        if let Some(name) = Self::find_unused_name(&random_names, &existing_set) {
            return name;
        }

        // If all names are used, append a number
        if let Some(name) = Self::find_available_name_with_number(&random_names, &existing_set) {
            return name;
        }

        // Fallback: use a timestamp-based name
        Self::generate_timestamp_name()
    }

    fn get_random_name_pool() -> [&'static str; 48] {
        [
            "Alice", "Bob", "Charlie", "Diana", "Eve", "Frank", "Grace", "Henry", "Iris", "Jack",
            "Kate", "Leo", "Mia", "Noah", "Olivia", "Peter", "Quinn", "Rose", "Sam", "Tina", "Uma",
            "Victor", "Wendy", "Xander", "Yara", "Zoe", "Alex", "Blake", "Casey", "Dylan", "Erin",
            "Finn", "Gabe", "Holly", "Ian", "Jade", "Kyle", "Luna", "Max", "Nora", "Owen", "Piper",
            "Ryan", "Sage", "Tyler", "Violet", "Wyatt", "Zara",
        ]
    }

    fn find_unused_name(
        random_names: &[&str],
        existing_names: &std::collections::HashSet<&str>,
    ) -> Option<String> {
        for &name in random_names {
            if !existing_names.contains(name) {
                return Some(name.to_string());
            }
        }
        None
    }

    fn find_available_name_with_number(
        random_names: &[&str],
        existing_names: &std::collections::HashSet<&str>,
    ) -> Option<String> {
        for &base_name in random_names {
            for i in 2..100 {
                // Try numbers 2-99
                let candidate = format!("{} {}", base_name, i);
                if !existing_names.contains(candidate.as_str()) {
                    return Some(candidate);
                }
            }
        }
        None
    }

    fn generate_timestamp_name() -> String {
        format!(
            "Player {}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        )
    }
}
