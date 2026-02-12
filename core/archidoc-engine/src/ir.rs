use archidoc_types::ModuleDoc;

/// Serialize a slice of ModuleDocs to JSON IR.
///
/// This produces the portable intermediate representation that bridges
/// language adapters and the core generator.
pub fn serialize(docs: &[ModuleDoc]) -> String {
    serde_json::to_string_pretty(docs).expect("failed to serialize ModuleDoc to JSON")
}

/// Deserialize JSON IR into ModuleDocs.
///
/// Returns an error message if the JSON is malformed or does not
/// conform to the ModuleDoc[] schema.
pub fn deserialize(json: &str) -> Result<Vec<ModuleDoc>, String> {
    serde_json::from_str(json).map_err(|e| format!("invalid IR: {}", e))
}

/// Validate JSON IR without deserializing into a full result.
///
/// Returns Ok(()) if the JSON conforms to the ModuleDoc[] schema,
/// or Err with a description of what's wrong.
pub fn validate(json: &str) -> Result<(), String> {
    let _: Vec<ModuleDoc> = serde_json::from_str(json)
        .map_err(|e| format!("IR validation failed: {}", e))?;
    Ok(())
}
