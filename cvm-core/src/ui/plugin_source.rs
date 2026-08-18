use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ItemSource {
    Native,
    Plugin { marketplace: String, plugin: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_native_variant_with_kind_tag() {
        let json = serde_json::to_value(ItemSource::Native).unwrap();
        assert_eq!(json, serde_json::json!({"kind": "native"}));
    }

    #[test]
    fn serializes_plugin_variant_with_marketplace_and_plugin() {
        let source = ItemSource::Plugin {
            marketplace: "acme".to_string(),
            plugin: "tool".to_string(),
        };
        let json = serde_json::to_value(source).unwrap();
        assert_eq!(
            json,
            serde_json::json!({"kind": "plugin", "marketplace": "acme", "plugin": "tool"})
        );
    }
}
