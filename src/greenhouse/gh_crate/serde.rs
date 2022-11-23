use bevy_reflect::{TypeRegistry, serde::TypedReflectDeserializer};
use serde::{Serialize, ser::{SerializeMap, SerializeStruct}, de::{DeserializeSeed, Visitor}, Deserialize};

use crate::greenhouse::gh_crate::ReflectCropData;

use super::{Fields, Crate};

impl<'a> Serialize for FieldSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        let mut map = serializer.serialize_map(Some(self.gh_crate.0.len()))?;
        for item in self.gh_crate.0.iter() {
            let val = bevy_reflect::serde::TypedReflectSerializer::new(item.as_reflect(), self.type_registry);
            map.serialize_entry(item.type_name(), &val)?;
        }
        map.end()
    }
}

struct FieldSerializer<'a> {
    gh_crate: &'a Fields,
    type_registry: &'a TypeRegistry,
}

pub struct CrateSerializer<'a> {
    gh_crate: &'a Crate,
    type_registry: &'a TypeRegistry,
}

impl<'a> CrateSerializer<'a> {
    pub fn new(gh_crate: &'a Crate, type_registry: &'a TypeRegistry) -> Self {
        CrateSerializer { gh_crate, type_registry }
    }
}

impl<'a> Serialize for CrateSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        let mut map = serializer.serialize_struct("Crate", 2)?;
        map.serialize_field("crop", &self.gh_crate.crop)?;
        map.serialize_field("fields", &FieldSerializer{gh_crate: &self.gh_crate.fields, type_registry: self.type_registry})?;
        map.end()
    }
}

struct FieldsDeserializer<'a> {
    type_registry: &'a TypeRegistry,
}

struct FieldsVisitor<'a> {
    type_registry: &'a TypeRegistry,
}

impl<'de, 'a> DeserializeSeed<'de> for FieldsDeserializer<'a> {
    type Value = Fields;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de> {
        deserializer.deserialize_map(FieldsVisitor{type_registry: self.type_registry})
    }
}

impl<'de, 'a> Visitor<'de> for FieldsVisitor<'a> {
    type Value = Fields;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Fields Map")
    }
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>, {
                use serde::de::Error;
        let mut fields = Vec::new();
        while let Some(key) = map.next_key::<&str>()? {
            let Some(reg) = self.type_registry.get_with_name(key) else {return Err(Error::custom(format!("{} is not in type registry", key)));};
            let val = map.next_value_seed(TypedReflectDeserializer::new(reg, self.type_registry))?;
            let Some(cd) = reg.data::<ReflectCropData>() else {return Err(Error::custom(format!("{} does not reflect CropData", key)));};
            let crop = cd.get_boxed(val).expect("val to be Cropdata");
            fields.push(crop);
        }
        Ok(Fields(fields))
    }
}

pub struct CrateDeserializer<'a> {
    pub type_registry: &'a TypeRegistry,
}

struct CrateVisitor<'a> {
    type_registry: &'a TypeRegistry,
}

impl<'a, 'de> DeserializeSeed<'de> for CrateDeserializer<'a> {
    type Value = Crate;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de> {
        deserializer.deserialize_struct("Crate", &["crop", "fields"],CrateVisitor{type_registry: self.type_registry})
    }
}

#[derive(Debug, Deserialize)]
#[serde(field_identifier, rename_all = "lowercase")]
enum CrateFields {
    Crop,
    Fields,
}

impl<'de, 'a> Visitor<'de> for CrateVisitor<'a> {
    type Value = Crate;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Crate Struct")
    }
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>, {
        use serde::de::Error;
        let mut fields = None;
        let mut crop = None;
        while let Some(key) = map.next_key()? {
            match key {
                CrateFields::Crop => {
                    if let None = crop {crop = Some(map.next_value()?)}
                    else {return Err(Error::duplicate_field("Crop"));};
                },
                CrateFields::Fields => {
                    if let None = fields {fields = Some(map.next_value_seed(FieldsDeserializer{type_registry: self.type_registry})?)}
                    else {return Err(Error::duplicate_field("Fields"));};
                }
            }
        }
        let Some(fields) = fields else {return Err(Error::missing_field("Fields"));};
        let Some(crop) = crop else {return Err(Error::missing_field("Crop"));};
        Ok(Crate{
            id: crate::ItemId::default(),
            fields,
            crop
        })
    }
}
