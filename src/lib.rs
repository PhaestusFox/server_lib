use bevy_reflect::prelude::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

pub use prelude::*;
mod prelude {
    pub use crate::date::Date;
    pub use crate::events::ItemData;
    pub use crate::events::EventData;
    pub use crate::events::EventKey;
    pub use crate::events::ItemId;
    pub use crate::events::Item;
    #[cfg(feature = "yew")]
    pub use crate::events::{YewObj, ObjList, yew_impl::ObjMsg, yew_impl::LoadedItems3};
    //#[cfg(feature = "yew")]
    //pub use crate::events:;
}
pub mod events;
pub mod date;
pub mod worms;

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
    fn get_next_key(&self, date: Date) -> Result<EventKey, DbError> {
        use bincode::Options;
        let key = EventKey::date_key(date);
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
    pub fn add_event(&self, event: &EventData) -> Result<EventKey, DbError> {
        let key = self.get_next_key(event.date)?;
        self.type_tree.insert(key, event.type_name.as_str())?;
        self.events.insert(key, event.data.as_str())?;
        Ok(key)
    }
    pub fn get_event_obj(&self, key: EventKey) -> anyhow::Result<Box<dyn Reflect>> {
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
    pub fn add_event_obj(&self, event: &dyn Reflect, date: Date) -> anyhow::Result<EventKey> {
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
    pub fn get<'de, T: Deserialize<'de> + Reflect>(&self, key: ItemId) -> anyhow::Result<T> {
        match self.get_obj(key)?.take() {
            Err(e) => Err(DbError::TypeMissMatch(e).into()),
            Ok(w) => Ok(w)
        }
    }
    pub fn get_item(&self, key: ItemId) -> Result<ItemData, DbError> {
        let name = if let Some(v) = self.type_tree.get(key)? {
            String::from_utf8(v.to_vec())?
        } else {return Err(DbError::NoTypeName);};
        let data = if let Some(v) = self.db.get(key)? {
            String::from_utf8(v.to_vec())?
        } else {return Err(DbError::NoData);};
        Ok(ItemData { type_name: name, data })
    }
    pub fn get_event(&self, key: EventKey) -> Result<EventData, DbError> {
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

    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Database, DbError> {
        let db = sled::open(path)?;
        Ok(Database {
            type_registry: type_registry(),
            events: db.open_tree("events")?,
            type_tree: db.open_tree("types")?,
            db
        })
    }
}

fn type_registry() -> bevy_reflect::TypeRegistry {
    let mut type_reg = bevy_reflect::TypeRegistry::new();
    type_reg.register::<events::Plant>();
    type_reg.register::<events::silkworms::SilkWormEvents>();
    type_reg
}

#[cfg(test)]
mod test {
    use crate::{EventKey, Database, events::{Plant, PlantTypes, silkworms::SilkWormEvents}};
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
        let key = EventKey::date_key(date);
        assert!(key.is_date_key());
        assert_eq!(date, key.date_from_key());
    }

    #[test]
    fn database_key_test() {
        let db = test_db();
        let date = Date::new_ymd(2022, 09, 30);
        let next = date.next();
        let key1 = db.get_next_key(date).unwrap();
        assert_eq!(key1, EventKey::date_key(date).with_id(1));
        let key2 = db.get_next_key(date).unwrap();
        assert_eq!(key2, EventKey::date_key(date).with_id(2));
        let next1 = db.get_next_key(next).unwrap();
        assert_eq!(next1, EventKey::date_key(next).with_id(1));
        let next2 = db.get_next_key(next).unwrap();
        assert_eq!(next2, EventKey::date_key(next).with_id(2));
    }

    #[test]
    fn database_typecheck() {
        use bevy_reflect::prelude::*;
        let mut db = test_db();
        db.type_registry.register::<Plant>();
        db.type_registry.register::<PlantTypes>();
        db.type_registry.register::<SilkWormEvents>();
        let date = test_date();
        let event0 = Plant::test(0);
        let event1 = Plant::test(1);
        let event2 = SilkWormEvents::Hatched(date);
        let event0_key = db.add_event_obj(event0.as_reflect(), date).unwrap();
        let event1_key = db.add_event_obj(event1.as_reflect(), date).unwrap();
        let event2_key = db.add_event_obj(event2.as_reflect(), date).unwrap();
        assert!(event0_key == EventKey::date_key(date).with_id(1));
        assert!(event1_key == EventKey::date_key(date).with_id(2));
        assert!(event2_key == EventKey::date_key(date).with_id(3));
        let event0_ref = match db.get_event_obj(event0_key) {
            Ok(v) => v,
            Err(e) => {println!("{:?}", e); panic!()},
        };
        let event1_ref = db.get_event_obj(event1_key).unwrap();
        let event2_ref = db.get_event_obj(event2_key).unwrap();
        let event0 = Plant::test(0);
        let event1 = Plant::test(1);
        let event2 = SilkWormEvents::Hatched(date);
        assert!(event0_ref.is::<Plant>());
        assert!(event1_ref.is::<Plant>());
        assert!(event2_ref.is::<SilkWormEvents>());
        assert_eq!(event0, event0_ref.take::<Plant>().unwrap());
        assert_eq!(event1, event1_ref.take::<Plant>().unwrap());
        assert_eq!(event2, event2_ref.take::<SilkWormEvents>().unwrap())
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