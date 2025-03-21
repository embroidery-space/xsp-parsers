pub mod oxs;

pub type AttributesMap = std::collections::HashMap<String, String>;
pub fn process_attributes(attributes: quick_xml::events::attributes::Attributes) -> anyhow::Result<AttributesMap> {
  let mut map = std::collections::HashMap::new();
  for attr in attributes {
    let attr = attr?;
    let key = String::from_utf8(attr.key.as_ref().to_vec())?;
    let value = String::from_utf8(attr.value.to_vec())?;
    map.insert(key, value);
  }
  Ok(map)
}
