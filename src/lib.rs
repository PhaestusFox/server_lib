use bevy_reflect::prelude::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

pub use prelude::*;
mod prelude {
    pub use crate::date::Date;
    pub use crate::items::ItemData;
    pub use crate::items::EventData;
    pub use crate::items::EventId;
    pub use crate::items::ItemId;
    pub use crate::items::Item;
    #[cfg(feature = "yew")]
    pub use crate::items::{YewObj, ObjList, yew_impl::ObjMsg, /*yew_impl::LoadedItems,*/ ObjView};
    //#[cfg(feature = "yew")]
    //pub use crate::events:;
}
pub mod items;
pub mod date;
pub mod worms;
pub mod plants;
pub mod greenhouse;
#[cfg(feature = "yew")]
pub mod components;

#[toml_cfg::toml_config]
pub (crate) struct Config {
    #[default("http://192.168.0.100")]
    server_id: &'static str,
    #[default(&Uuid::from_u128_le(17975531051516076104489778949011204464))]
    greenhouse_namespace: &'static Uuid,
}

#[test]
fn print_uuid() {
    use std::str::FromStr;
    let uuid = uuid::Uuid::from_str("70e96686-393e-4dba-9909-6f271bf6850d").unwrap();
    assert_eq!(uuid, *CONFIG.greenhouse_namespace);
    print!("{}", uuid.to_u128_le())
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Note {
    pub metadata: Metadata,
    pub content: String,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Metadata {
    pub tital: String,
    pub content_id: Uuid,
}

pub struct Database {
    type_registry: bevy_reflect::TypeRegistry,
    db: sled::Db,
    events: sled::Tree,
    type_tree: sled::Tree,
}

impl Database {
    fn get_next_key(&self, date: Date) -> Result<EventId, DbError> {
        use bincode::Options;
        let key = EventId::date_key(date);
        let options = bincode::options().with_big_endian();
        let next = self.db.update_and_fetch(key, |v| {
            let mut next = match v {
                Some(raw) => {
                    options.deserialize::<u64>(raw).unwrap_or(0)
                },
                None => 0,
            };
            next += 1;
            Some(options.serialize(&next).unwrap())
        })?.expect("will return 1 if it did not find a pre exising num");
        let id: u64 = options.deserialize(&next).unwrap();
        Ok(key.with_id(id))
    }
    #[inline(always)]
    pub fn add_event(&self, event: &EventData) -> Result<EventId, DbError> {
        let key = self.get_next_key(event.date)?;
        self.type_tree.insert(key, event.type_name.as_str())?;
        self.events.insert(key, event.data.as_str())?;
        Ok(key)
    }
    pub fn get_event_obj(&self, key: EventId) -> anyhow::Result<Box<dyn Reflect>> {
        let type_name = if let Some(name) = self.type_tree.get(key)? {
            String::from_utf8(name.to_vec())?
        } else {
            return Err(DbError::NoTypeName.into());
        };
        let registration = match self.type_registry.get_with_name(&type_name) {
            Some(r) => r,
            None => {return Err(DbError::TypeNotRegistered(type_name).into())}
        };
        let ser = match registration.data::<ReflectDeserialize>() {
            Some(s) => s,
            None => return Err(DbError::NoReflectDeSerialize(registration.type_name()).into()),
        };
        let data = if let Some(data) = self.events.get(key)? {
            println!("{}", String::from_utf8_lossy(&data));
            data
        } else {
            return Err(DbError::NoData.into());
        };
        let mut de = ron::Deserializer::from_bytes(&data)?;
        //let mut de = bincode::Deserializer::from_slice(&data, bincode::options());
        Ok(ser.deserialize(&mut de)?)
    }
    pub fn add_event_obj(&self, event: &dyn Reflect, date: Date) -> anyhow::Result<EventId> {
        let key = self.get_next_key(date)?;
        let registration = match self.type_registry.get(event.type_id()) {
            Some(r) => r,
            None => {return Err(DbError::TypeNotRegistered(event.type_name().to_string()).into())}
        };
        let ser = match registration.data::<ReflectSerialize>() {
            Some(s) => s,
            None => return Err(DbError::NoReflectSerialize(registration.type_name()).into()),
        };
        let ser = match ser.get_serializable(event.as_reflect()) {
            bevy_reflect::serde::Serializable::Owned(s) => ron::to_string(&s),
            bevy_reflect::serde::Serializable::Borrowed(s) => ron::to_string(s),
        }?;
        self.type_tree.insert(key, registration.type_name())?;
        self.events.insert(key, ser.as_str())?;
        Ok(key)
    }
    pub fn insert<T: Serialize + Reflect>(&self, item: T) -> anyhow::Result<ItemId> {
        let uuid = Uuid::new_v4();
        let registration = match self.type_registry.get(item.type_id()) {
            Some(r) => r,
            None => {return Err(DbError::TypeNotRegistered(item.type_name().to_string()).into())}
        };
        let ser = match registration.data::<ReflectSerialize>() {
            Some(s) => s,
            None => return Err(DbError::NoReflectSerialize(registration.type_name()).into()),
        };
        let ser = match ser.get_serializable(item.as_reflect()) {
            bevy_reflect::serde::Serializable::Owned(s) => ron::to_string(&s),
            bevy_reflect::serde::Serializable::Borrowed(s) => ron::to_string(s),
        }?;
        self.type_tree.insert(uuid, registration.type_name())?;
        self.events.insert(uuid, ser.as_str())?;
        Ok(ItemId(uuid))
    }
    pub fn get_obj(&self, key: ItemId) -> anyhow::Result<Box<dyn Reflect>> {
        let type_name = if let Some(name) = self.type_tree.get(key)? {
            String::from_utf8(name.to_vec())?
        } else {
            return Err(DbError::NoTypeName.into());
        };
        let registration = match self.type_registry.get_with_name(type_name.as_str()) {
            Some(r) => r,
            None => {return Err(DbError::TypeNotRegistered(type_name.into()).into())}
        };
        let ser = match registration.data::<ReflectDeserialize>() {
            Some(s) => s,
            None => return Err(DbError::NoReflectDeSerialize(registration.type_name()).into()),
        };
        let data = if let Some(data) = self.events.get(key)? {
            data
        } else {
            return Err(DbError::NoData.into());
        };
        let mut de = ron::Deserializer::from_bytes(&data)?;
        //let mut de = bincode::Deserializer::from_slice(&data, bincode::options());
        Ok(ser.deserialize(&mut de).unwrap())
    }
    #[inline(always)]
    pub fn get_item<T: serde::de::DeserializeOwned>(&self, key: ItemId) -> anyhow::Result<T> {
        let type_name = String::from_utf8(self.type_tree.get(key)?.ok_or_else(|| DbError::NoTypeName)?.to_vec())?;
        if type_name != std::any::type_name::<T>() {anyhow::bail!("{} != {}", type_name, std::any::type_name::<T>());}
        let data = String::from_utf8(self.db.get(key)?.ok_or_else(|| DbError::NoData)?.to_vec())?;
        Ok(ron::from_str(&data)?)
    }
    pub fn get_item_data(&self, key: ItemId) -> Result<ItemData, DbError> {
        let name = if let Some(v) = self.type_tree.get(key)? {
            String::from_utf8(v.to_vec())?
        } else {return Err(DbError::NoTypeName);};
        let data = if let Some(v) = self.db.get(key)? {
            String::from_utf8(v.to_vec())?
        } else {return Err(DbError::NoData);};
        Ok(ItemData { type_name: name, data })
    }
    pub fn get_event(&self, key: EventId) -> Result<EventData, DbError> {
        let name = if let Some(v) = self.type_tree.get(key)? {
            String::from_utf8(v.to_vec())?
        } else {return Err(DbError::NoTypeName);};
        let data = if let Some(v) = self.events.get(key)? {
            String::from_utf8(v.to_vec())?
        } else {return Err(DbError::NoData);};
        Ok(EventData {
            type_name: name,
            data,
            date: key.date_from_key(),
        })
    }
    pub fn add_item(&self, item: &ItemData) -> Result<ItemId, DbError> {
        let uuid = Uuid::new_v4();
        self.type_tree.insert(uuid, item.type_name.as_str())?;
        self.db.insert(uuid, item.data.as_str())?;
        Ok(ItemId(uuid))
    }
    pub fn insert_item<T: Serialize>(&self, id: ItemId, item: T) -> anyhow::Result<()> {
        let ser = ron::to_string(&item)?;
        self.type_tree.insert(id, std::any::type_name::<T>())?;
        self.db.insert(id, ser.as_str())?;
        Ok(())
    }
    pub fn insert_item_data(&self, id: ItemId, item: &ItemData) -> Result<(), DbError> {
        self.type_tree.insert(id, item.type_name.as_str())?;
        self.db.insert(id, item.data.as_str())?;
        Ok(())
    }
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Database, DbError> {
        let db = sled::open(path)?;
        Ok(Database {
            type_registry: type_registry(),
            events: db.open_tree("events")?,
            type_tree: db.open_tree("types")?,
            db
        })
    }

    pub fn remove(&self, id: ItemId) -> Result<(), DbError> {
        self.db.remove(id)?;
        self.type_tree.remove(id)?;
        Ok(())
    }
}

fn type_registry() -> bevy_reflect::TypeRegistry {
    let mut type_reg = bevy_reflect::TypeRegistry::new();
    type_reg.register::<ItemId>();
    type_reg.register::<Vec<ItemId>>();
    worms::register_types(&mut type_reg);
    greenhouse::register_types(&mut type_reg);
    type_reg
}

#[cfg(test)]
mod test {
    use crate::{EventId, Database, plants::{Plant, PlantTypes}};
    use super::Date;
    fn test_db() -> Database {
        let db_options = sled::Config::new().temporary(true);
        let db = db_options.open().unwrap();
        Database {
            type_tree: db.open_tree("types").unwrap(),
            events: db.open_tree("events").unwrap(),
            db,
            type_registry: bevy_reflect::TypeRegistry::new(),
        }
    }

    fn test_date() -> Date {
        Date::new_ymd(2022, 10, 01)
    }
    #[test]
    fn date_key() {
        let date = Date::new_ymd(2030, 12, 11);
        let key = EventId::date_key(date);
        assert!(key.is_date_key());
        assert_eq!(date, key.date_from_key());
    }

    #[test]
    fn database_key_test() {
        let db = test_db();
        let date = Date::new_ymd(2022, 09, 30);
        let next = date.next();
        let key1 = db.get_next_key(date).unwrap();
        assert_eq!(key1, EventId::date_key(date).with_id(1));
        let key2 = db.get_next_key(date).unwrap();
        assert_eq!(key2, EventId::date_key(date).with_id(2));
        let next1 = db.get_next_key(next).unwrap();
        assert_eq!(next1, EventId::date_key(next).with_id(1));
        let next2 = db.get_next_key(next).unwrap();
        assert_eq!(next2, EventId::date_key(next).with_id(2));
    }

    #[test]
    fn database_typecheck() {
        use bevy_reflect::prelude::*;
        let mut db = test_db();
        db.type_registry.register::<Plant>();
        db.type_registry.register::<PlantTypes>();
        let date = test_date();
        let event0 = Plant::test(0);
        let event1 = Plant::test(1);
        let event0_key = db.add_event_obj(event0.as_reflect(), date).unwrap();
        let event1_key = db.add_event_obj(event1.as_reflect(), date).unwrap();
        assert!(event0_key == EventId::date_key(date).with_id(1));
        assert!(event1_key == EventId::date_key(date).with_id(2));
        let event0_ref = db.get_event_obj(event0_key).unwrap();
        let event1_ref = db.get_event_obj(event1_key).unwrap();
        let event0 = Plant::test(0);
        let event1 = Plant::test(1);
        assert!(event0_ref.is::<Plant>());
        assert!(event1_ref.is::<Plant>());
        assert_eq!(event0, event0_ref.take::<Plant>().unwrap());
        assert_eq!(event1, event1_ref.take::<Plant>().unwrap());
    }
}

use thiserror::*;
#[derive(Debug, Error)]
pub enum DbError {
    #[error("Type {0} is not registered")]
    TypeNotRegistered(String),
    #[error("Type {0} does not have ReflectSerialize")]
    NoReflectSerialize(&'static str),
    #[error("no type id found for event")]
    NoTypeName,
    #[error("Type {0} does not have ReflectSerialize")]
    NoReflectDeSerialize(&'static str),
    #[error("There is not event data for this key")]
    NoData,
    #[error("The type was wrong here is the reflect object")]
    TypeMissMatch(Box<dyn Reflect>),
    #[error("sled error")]
    SledError(#[from] sled::Error),
    #[error("From Utf8Error")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
    #[cfg(feature = "Rocket")]
    #[error("Rocket Error")]
    RocketError(#[from] rocket::Error),
    #[error("Io Error")]
    IoError(#[from] std::io::Error),
    #[error("ron spanned error")]
    RonSpannedError(#[from] ron::error::SpannedError)
}