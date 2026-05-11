use archidoc_types::ir::ArchitectureIR;

/// Serialize an `ArchitectureIR` to pretty-printed JSON.
pub fn serialize_ir(ir: &ArchitectureIR) -> String {
    serde_json::to_string_pretty(ir).expect("failed to serialize ArchitectureIR to JSON")
}

/// Deserialize an `ArchitectureIR` from JSON.
pub fn deserialize_ir(json: &str) -> Result<ArchitectureIR, String> {
    serde_json::from_str(json).map_err(|e| format!("invalid IR: {}", e))
}
