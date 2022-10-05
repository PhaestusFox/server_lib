use bevy_reflect::{prelude::*, FromReflect};
use serde::{Serialize,Deserialize};
use crate::*;

pub mod silkworms;

#[cfg(feature="yew")]
pub use self::yew_impl::{YewObj, ObjList};

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemData {
    pub type_name: String,
    pub data: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventData {
    pub type_name: String,
    pub data: String,
    pub date: Date,
}

#[cfg(feature = "rocket")]
mod rocket {
    use std::str::FromStr;

    pub use rocket::*;

    use crate::DbError;
    #[rocket::async_trait]
    impl<'r> rocket::data::FromData<'r> for super::ItemData {
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

    #[rocket::async_trait]
    impl<'r> rocket::data::FromData<'r> for super::EventData {
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

    impl<'a> rocket::request::FromParam<'a> for super::EventKey {
        type Error = <u64 as std::str::FromStr>::Err;

        fn from_param(param: &'a str) -> Result<Self, Self::Error> {
            super::EventKey::from_str(param)
        }
    }

    impl<'a> rocket::request::FromParam<'a> for super::ItemId {
        type Error = <Self as std::str::FromStr>::Err;
        
        fn from_param(param: &'a str) -> Result<Self, Self::Error> {
            Ok(crate::ItemId(uuid::Uuid::from_str(param)?))
        }
    }
}

#[derive(Debug, Reflect, Serialize, Deserialize, PartialEq)]
#[reflect(Serialize, Deserialize)]
pub struct Plant {
    plant_type: PlantTypes,
}

#[derive(Debug, Reflect, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[reflect_value(Serialize, Deserialize)]
pub enum PlantTypes {
    Lettuce,
    Tomato
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct EventKey([u8; 8]);
impl EventKey {
    pub fn date_key(date: Date) -> EventKey {
        let data: [u8; 8] = ((date.0 as u64) << 32).to_be_bytes();
        EventKey(data)
    }
    pub fn is_date_key(&self) -> bool {
        self.0[3] & 0x80 == self.0[3] && self.0[4..] == [0; 4]
    }
    pub fn date_from_key(&self) -> Date {
        let data: [u8;4] = [self.0[0], self.0[1], self.0[2], self.0[3] & 0x80];
        Date(u32::from_be_bytes(data))
    }
    pub fn set_val(&mut self, num: u64) {
        let num = num.to_be_bytes();
        self.0[4] = (self.0[4] & 0x80) | (num[4] & 0x7F);
        for i in 5..8 {
            self.0[i] = num[i];
        }
    }
    pub fn set_date(&mut self, date: Date) {
        let date = date.0.to_be_bytes();
        for i in 0..3 {
            self.0[i] = date[i];
        }
        self.0[3] = (date[3] & 0x80) | (self.0[3] & 0x7F);
    }
    pub fn with_id(mut self, id: u64) -> EventKey {
        self.set_val(id);
        self
    }
}

impl AsRef<[u8]> for EventKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl ToString for EventKey {
    fn to_string(&self) -> String {
        u64::from_be_bytes(self.0).to_string()
    }
}

impl std::str::FromStr for EventKey {
    type Err = <u64 as std::str::FromStr>::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(EventKey(u64::from_str(s)?.to_be_bytes()))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Reflect, FromReflect, Hash)]
#[reflect_value(Deserialize, Serialize)]
pub struct ItemId(pub(crate) Uuid);

impl ItemId {
    pub fn from_u128(id: u128) -> ItemId {
        ItemId(Uuid::from_u128(id))
    }
}

impl std::ops::Deref for ItemId {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for ItemId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ItemId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        Ok(ItemId(Uuid::deserialize(deserializer)?))
    }
}

impl ItemId {
    pub fn new() -> ItemId {
        ItemId(Uuid::new_v4())
    }
}

impl ToString for ItemId {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl std::str::FromStr for ItemId {
    type Err = <Uuid as std::str::FromStr>::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ItemId(Uuid::from_str(s)?))
    }
}

impl AsRef<Uuid> for ItemId {
    fn as_ref(&self) -> &Uuid {
        &self.0
    }
}

impl AsRef<[u8]> for ItemId {
    fn as_ref(&self) -> &[u8] {
        &self.0.as_ref()
    }
}

pub trait Item: Reflect + Extend {
    fn dependencies(&self) -> Option<Vec<ItemId>> {
        None
    }
    fn id(&self) -> ItemId;
}

impl PartialEq for dyn Item {
    fn eq(&self, other: &Self) -> bool {
        self.reflect_partial_eq(other.as_reflect()).unwrap()
    }
}
#[cfg(feature = "yew")]
pub trait Extend: Reflect {
    fn yew_obj(&self) -> Option<&dyn crate::events::yew_impl::YewObj> {
        None
    }
    fn yew_view(&self, ctx: &yew::Context<crate::ObjList>) -> yew::Html {
        if let Some(yew_obj) = self.yew_obj() {
            yew_obj.view(ctx)
        } else {
            yew::html!{
                <div class="error">
                {self.type_name()}{" does not impl YewObj"}
                </div>
            }
        }
    }
}
#[cfg(not(feature = "yew"))]
pub trait Extend { }

#[cfg(not(feature = "yew"))]
impl<T> Extend for T  { }

#[cfg(feature = "yew")]
pub mod yew_impl {
    use std::rc::Rc;

    use yew::*;
    use crate::*;

    impl<T: YewObj> super::Extend for T {
        fn yew_obj(&self) -> Option<&dyn crate::events::yew_impl::YewObj> {
            Some(self as &dyn YewObj)
        }
    }

    pub trait YewObj: super::Item {
        fn view(&self, ctx: &Context<ObjList>) -> Html;
    }

    impl PartialEq for dyn YewObj {
        fn eq(&self, other: &Self) -> bool {
            self.reflect_partial_eq(other.as_reflect()).unwrap()
        }
    }

    #[derive(Debug, Clone)]
    pub struct ObjList;
    pub struct LoadedItems3(std::sync::RwLock<std::collections::HashMap<ItemId, Box<dyn Item>>>);

    pub(crate) static LOADED_ITEMS: once_cell::sync::Lazy<LoadedItems3> = once_cell::sync::Lazy::new(||
        LoadedItems3(std::sync::RwLock::new(std::collections::HashMap::default())));

    impl LoadedItems3 {
        pub fn read() -> std::sync::RwLockReadGuard<'static, std::collections::HashMap<ItemId, Box<dyn Item>>> {
            LOADED_ITEMS.0.read().unwrap()
        }
        pub fn write() -> std::sync::RwLockWriteGuard<'static, std::collections::HashMap<ItemId, Box<dyn Item>>>{
            LOADED_ITEMS.0.write().unwrap()
        }
        pub fn load(id: Option<ItemId>, item: Box<dyn Item>) -> ItemId {
            let id = if let Some(id) = id {id} else {item.id()};
            LoadedItems3::write().insert(id, item);
            id
        }
    }

    impl Component for ObjList {
        type Message = ObjMsg;
        type Properties = Props;

        fn create(_ctx: &Context<Self>) -> Self {
            ObjList
        }

        fn view(&self, ctx: &Context<Self>) -> Html {
            html! {
                <div>
                {for ctx.props().display.iter().map(|item| LoadedItems3::read().get(item).map_or(html!{"loading..."}, |e| e.yew_view(ctx)))}
                <button onclick={ctx.link().callback(|_| ObjMsg::Test(0))}>{"add 0"}</button>
                <button onclick={ctx.link().callback(|_| ObjMsg::Test(1))}>{"add 1"}</button>
                <button onclick={ctx.link().callback(|_| ObjMsg::Test(2))}>{"add 2"}</button>
                </div>
            }
        }

        fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
            match msg {
                ObjMsg::Load(item) => {
                    LoadedItems3::load(None, item);
                    true
                },
                ObjMsg::Get(_) => {false},
                ObjMsg::Test(id) => {
                    let obj = match id {
                        0 => super::TestString {id: ItemId::from_u128(0), data: "Test 0".to_string()},
                        1 => super::TestString {id: ItemId::from_u128(1), data: "Loaded".to_string()},
                        2 => super::TestString {id: ItemId::from_u128(1), data: "Alt Loaded".to_string()},
                        3 => super::TestString {id: ItemId::from_u128(0), data: "Alt 0".to_string()},
                        4 => super::TestString {id: ItemId::from_u128(0), data: "Alt 1".to_string()},
                        _ => {return false;}
                    };
                    _ctx.link().send_message(ObjMsg::Load(Box::new(obj)));
                    false
                },
            }
        }
    }

    #[derive(Properties, PartialEq)]
    pub struct ViewProps{
        id: ItemId,
    }

    pub struct ItemView;
    pub enum ViewMsg {
        Apply(Box<dyn Reflect>),
    }

    impl Component for ItemView {
        type Message = ViewMsg;
        type Properties = ViewProps;
        fn create(_ctx: &Context<Self>) -> Self {
            Self
        }
        fn view(&self, ctx: &Context<Self>) -> Html {
            let id = ctx.props().id;
            if let Some(_) = LoadedItems3::read().get(&id) {
                html! {
                    <button onclick={ctx.link().callback(|_| {
                        let val = Box::new(
                            super::TestString {
                                id: ItemId::from_u128(0),
                                data: "Set 100%".to_string(),
                            }
                        );
                        ViewMsg::Apply(val)
                    })}>{"click to set 100%"}</button>
                }
            } else {
                html!{"loading..."}
            }
        }
        fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
            match msg {
                ViewMsg::Apply(data) => {
                    let mut lock = LoadedItems3::write();
                    if let Some(obj) = lock.get_mut(&ctx.props().id) {
                        obj.apply(data.as_reflect());
                        true
                    } else {
                        false
                    }
                }
            }
        }
    }


    pub enum ObjMsg {
        Load(Box<dyn Item>),
        Get(ItemId),
        Test(u8),
    }

    #[derive(Properties, PartialEq)]
    pub struct Props {
        pub display: Rc<Vec<ItemId>>
    }

    impl YewObj for super::TestString {
        fn view(&self, _ctx: &yew::Context<ObjList>) -> yew::Html {
            yew::html!{
                {&self.data}
            }
        }
    }
}

#[cfg(test)]
mod test {
    impl super::Plant {
        pub fn test(id: u8) -> super::Plant {
            match id {
                0 => super::Plant {
                    plant_type: super::PlantTypes::Lettuce,
                },
                _ => super::Plant {
                    plant_type: super::PlantTypes::Tomato,
                },
            }
            
        }
    }
    #[test]
    fn reflect() {
        use crate::*;
        use crate::events::*;
        use crate::events::silkworms::SilkWormEvents;
        let plant = Plant {
            plant_type: PlantTypes::Tomato,
        };
        let silkworm = SilkWormEvents::Hatched(Date::new_ymd(2022, 10, 01));
        let mut type_regisury = bevy_reflect::TypeRegistry::new();
        type_regisury.register::<Plant>();
        type_regisury.register::<PlantTypes>();
        type_regisury.register::<SilkWormEvents>();
        
        fn to_data(to_ser: &dyn Reflect, type_regisury: &bevy_reflect::TypeRegistry) -> (&'static str, String) {
            let info = type_regisury.get(to_ser.type_id()).unwrap();
            let ser = info.data::<ReflectSerialize>().unwrap();
            match ser.get_serializable(to_ser) {
                bevy_reflect::serde::Serializable::Owned(s) => (info.type_name(), ron::to_string(&s).unwrap()),
                bevy_reflect::serde::Serializable::Borrowed(s) => (info.type_name(), ron::to_string(s).unwrap()),
            }
        }
        
        fn from_data(from: &(&str, String), type_regisury: &bevy_reflect::TypeRegistry) -> Box<dyn Reflect> {
            let info = type_regisury.get_with_name(from.0).unwrap();
            let ser = info.data::<ReflectDeserialize>().unwrap();
            let mut de = ron::Deserializer::from_str(&from.1).unwrap();
            ser.deserialize(&mut de).unwrap()
        }
    let data0 = to_data(plant.as_reflect(), &type_regisury);
    let data1 = to_data(silkworm.as_reflect(), &type_regisury);
    let data = [data0, data1];
    println!("{:?}", data);
    let obj0 = from_data(&data[0], &type_regisury);
    let obj1 = from_data(&data[1], &type_regisury);
    assert!(obj0.is::<Plant>());
    assert!(obj1.is::<SilkWormEvents>());
}

}
#[derive(Debug, Reflect)]
pub struct TestString {
    id: ItemId,
    data: String,
}

impl Item for TestString {
    fn id(&self) -> ItemId {
        self.id
    }
}

