use std::collections::HashMap;

/// Parses Farley-style string parameters.
///
/// Input: `&["module: bus", "pattern: Mediator"]`
/// Output: HashMap with `"module" -> "bus"`, `"pattern" -> "Mediator"`
pub struct Params {
    map: HashMap<String, String>,
}

impl Params {
    pub fn parse(args: &[&str]) -> Self {
        let mut map = HashMap::new();
        for arg in args {
            if let Some((key, value)) = arg.split_once(':') {
                map.insert(key.trim().to_string(), value.trim().to_string());
            }
        }
        Self { map }
    }

    pub fn get(&self, key: &str) -> String {
        self.map
            .get(key)
            .unwrap_or_else(|| panic!("missing required parameter: {}", key))
            .clone()
    }

    pub fn get_opt(&self, key: &str) -> Option<String> {
        self.map.get(key).cloned()
    }

    pub fn get_usize(&self, key: &str) -> usize {
        self.get(key)
            .parse()
            .unwrap_or_else(|_| panic!("parameter '{}' is not a valid usize", key))
    }
}
