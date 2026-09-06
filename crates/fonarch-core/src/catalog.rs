use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::error::{Error, Result};

#[derive(Debug, Clone)]
pub struct FontMeta {
    pub id: String,
    pub family_name: String,
    pub variation_name: String,
    pub full_name: String,
    pub is_variable: bool,
}

/// Child-tag catalog under `<typekitSyncState>`. Adobe stores names as
/// elements (`<familyName>`), not attributes — the Python prototype read
/// attributes and got empty ids.
pub fn parse_entitlements(path: &Path) -> Result<BTreeMap<String, FontMeta>> {
    let text = fs::read_to_string(path)?;
    parse_entitlements_str(path, &text)
}

pub fn parse_entitlements_str(path: &Path, text: &str) -> Result<BTreeMap<String, FontMeta>> {
    let doc = roxmltree::Document::parse(text).map_err(|e| Error::Xml(e.to_string()))?;
    let root = doc.root_element();
    if root.tag_name().name() != "typekitSyncState" {
        return Err(Error::BadCatalog(path.to_path_buf()));
    }

    let mut map = BTreeMap::new();
    for font in root.descendants().filter(|n| n.has_tag_name("font")) {
        let Some(meta) = read_font(font) else {
            continue;
        };
        map.insert(meta.id.clone(), meta);
    }
    Ok(map)
}

fn child_text(node: roxmltree::Node<'_, '_>, tag: &str) -> Option<String> {
    node.children()
        .find(|n| n.has_tag_name(tag))
        .and_then(|n| n.text())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn read_font(font: roxmltree::Node<'_, '_>) -> Option<FontMeta> {
    let id = child_text(font, "id")?;
    let props = font.children().find(|n| n.has_tag_name("properties"))?;
    let family_name = child_text(props, "familyName")?;
    let full_name = child_text(props, "fullName").unwrap_or_else(|| family_name.clone());
    let variation_name = child_text(props, "variationName").unwrap_or_else(|| "Regular".into());
    let is_variable = child_text(props, "isVariable")
        .map(|s| s.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    Some(FontMeta {
        id,
        family_name,
        variation_name,
        full_name,
        is_variable,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    const FIXTURE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<typekitSyncState>
  <state>abc</state>
  <fonts type="array">
    <font>
      <id>169</id>
      <properties>
        <fullName>Proxima Nova Extrabold</fullName>
        <familyName>Proxima Nova</familyName>
        <variationName>Extrabold</variationName>
        <isVariable>false</isVariable>
      </properties>
    </font>
    <font>
      <id>1</id>
      <properties>
        <fullName>Noto Sans Variable</fullName>
        <familyName>Noto Sans</familyName>
        <variationName>Regular</variationName>
        <isVariable>true</isVariable>
      </properties>
    </font>
  </fonts>
</typekitSyncState>
"#;

    #[test]
    fn child_tags_not_attributes() {
        let map = parse_entitlements_str(Path::new("fixture.xml"), FIXTURE).unwrap();
        assert_eq!(map.len(), 2);
        let p = &map["169"];
        assert_eq!(p.family_name, "Proxima Nova");
        assert_eq!(p.variation_name, "Extrabold");
        assert_eq!(p.full_name, "Proxima Nova Extrabold");
        assert!(!p.is_variable);
        assert!(map["1"].is_variable);
    }

    #[test]
    fn attributes_would_be_empty() {
        // Same shape the Python script expected — must not produce a catalog.
        let attr_xml = r#"<typekitSyncState><font id="169" familyName="Nope"/></typekitSyncState>"#;
        let map = parse_entitlements_str(Path::new("a.xml"), attr_xml).unwrap();
        assert!(map.is_empty());
    }

    #[test]
    fn rejects_other_roots() {
        let err = parse_entitlements_str(Path::new("x.xml"), "<html></html>").unwrap_err();
        assert!(matches!(err, Error::BadCatalog(_)));
    }
}
