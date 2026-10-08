// The plugin's install script downloads the release tagged with the version in
// plugin.json, so it must match the crate version being released.
#[test]
fn plugin_version_matches_cargo() {
    let manifest = std::fs::read_to_string("plugin/.claude-plugin/plugin.json").unwrap();
    let manifest: serde_json::Value = serde_json::from_str(&manifest).unwrap();
    assert_eq!(manifest["version"], env!("CARGO_PKG_VERSION"));
}
