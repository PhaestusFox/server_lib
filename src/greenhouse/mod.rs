use std::fmt::Debug;
use std::fmt::Display;

use crate::*;
use crate::items::ReflectToItem;
use enum_utils::FromStr;
use strum::{EnumIter, IntoStaticStr};
use bevy_reflect::Reflect;

mod gh_crate;

pub(crate) fn register_types(type_reg: &mut bevy_reflect::TypeRegistry) {
    type_reg.register::<Crop>();
    type_reg.register::<Crate>();
    type_reg.register::<CrateSize>();
    type_reg.register::<Grade>();
}


#[derive(Debug, Deserialize, Serialize, Reflect, Clone, Copy, FromStr, EnumIter, IntoStaticStr, Default, PartialEq, Eq, Hash)]
#[reflect_value(Deserialize, Serialize)]
pub enum Crop {
    #[default]
    None,
    CherryTomato,
    TrussTomato,
    ParsleyCurly,
    ParsleyFlat,
}

impl Display for Crop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Display for CrateSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Deserialize, Serialize, Reflect, Clone, Copy, FromStr, EnumIter, IntoStaticStr, Default, PartialEq, Eq, Hash)]
#[reflect_value(Deserialize, Serialize)]
pub enum CrateSize {
    #[default]
    None,
    Small,
    Normal,
    Large,
}

impl Display for Grade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Deserialize, Serialize, Reflect, Clone, Copy, FromStr, EnumIter, IntoStaticStr, Default, PartialEq, Eq, Hash)]
#[reflect_value(Deserialize, Serialize)]
pub enum Grade {
    #[default]
    None,
    First,
    Second,
    Third,
}

#[derive(Debug, Deserialize, Serialize, Reflect, Default, Clone, Copy, Hash, PartialEq, Eq)]
#[reflect(Deserialize, Serialize, Default, ToItem)]
pub struct Crate {
    #[serde(skip)]
    id: ItemId,
    crop: Crop,
    size: CrateSize,
    grade: Grade,
}

impl Crate {
    fn match_crate(&self, patern: &Crate) -> bool {
        if patern.crop != self.crop && patern.crop != Crop::None {
            return false;
        }
        if patern.size != self.size && patern.size != CrateSize::None {
            return false;
        }
        if patern.grade != self.grade && patern.grade != Grade::None {
            return false;
        }
        true
    }
}

impl Item for Crate {
    fn id(&self) -> ItemId {
        self.id
    }
    fn set_id(&mut self, id: ItemId) {
        self.id = id;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerSideEvent {
    AddedItem(ItemId, Date),
    RemovedItem(ItemId, Date),
    UpadteItem(ItemId),
}

#[cfg(feature="rocket")]
mod rocket {
    use rocket::*;
    use crate::*;
    use super::ServerSideEvent;
#[rocket::async_trait]
impl<'r> rocket::data::FromData<'r> for ServerSideEvent {
    type Error = DbError;

    async fn from_data(_: &'r Request<'_>, data: Data<'r>) -> data::Outcome<'r, Self> {
        use rocket::data::ToByteUnit;
        let datastream = data.open(2.megabytes());
        let data_string = match datastream.into_string().await {
            Ok(str) => str,
            Err(e) => return data::Outcome::Failure((rocket::http::Status::InternalServerError, e.into())),
        };
        let note = match ron::from_str(&data_string) {
            Ok(note) => note,
            Err(e) => return data::Outcome::Failure((rocket::http::Status::InternalServerError, e.into())),
        };
        data::Outcome::Success(note)
    }
}
}
#[cfg(feature = "yew")]
pub use self::yew::GreenHouse;
#[cfg(feature = "yew")]
mod yew;

pub fn date_to_id(date: Date) -> ItemId {
    let bytes = date.0.to_be_bytes();
    ItemId(Uuid::new_v3(CONFIG.greenhouse_namespace, &bytes))
}

pub enum GreenHouseMsg {
    Get(ItemId),
    LoadList(Vec<ItemId>),
    LoadItem(Box<dyn Item>),
    SetDate(Date),
    ServerEvent(ServerSideEvent),
    NewCrate,
    EditCrate(ItemId),
    SaveCrate,
    DeleteCrate,
    AddToSum(Crate),
    RemoveFromSum(Crate),
    SetPlot(String),
}