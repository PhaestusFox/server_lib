use bevy_reflect::{prelude::*, FromReflect};
use serde::{Serialize,Deserialize};
use crate::*;

#[cfg(feature="yew")]
pub use self::yew_impl::{YewObj, ObjList, ObjView};

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

    impl<'a> rocket::request::FromParam<'a> for super::EventId {
        type Error = <u64 as std::str::FromStr>::Err;

        fn from_param(param: &'a str) -> Result<Self, Self::Error> {
            super::EventId::from_str(param)
        }
    }

    impl<'a> rocket::request::FromParam<'a> for super::ItemId {
        type Error = <Self as std::str::FromStr>::Err;
        
        fn from_param(param: &'a str) -> Result<Self, Self::Error> {
            Ok(crate::ItemId(uuid::Uuid::from_str(param)?))
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct EventId([u8; 8]);
impl EventId {
    pub fn date_key(date: Date) -> EventId {
        let data: [u8; 8] = ((date.0 as u64) << 32).to_be_bytes();
        EventId(data)
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
    pub fn with_id(mut self, id: u64) -> EventId {
        self.set_val(id);
        self
    }
}

impl AsRef<[u8]> for EventId {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl ToString for EventId {
    fn to_string(&self) -> String {
        u64::from_be_bytes(self.0).to_string()
    }
}

impl std::str::FromStr for EventId {
    type Err = <u64 as std::str::FromStr>::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(EventId(u64::from_str(s)?.to_be_bytes()))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Reflect, FromReflect, Hash)]
#[reflect_value(Deserialize, Serialize)]
pub struct ItemId(pub(crate) Uuid);

impl ItemId {
    pub fn from_u128(id: u128) -> ItemId {
        ItemId(Uuid::from_u128(id))
    }
    pub fn nil() -> ItemId {
        ItemId(Uuid::nil())
    }
    pub fn new() -> ItemId {
        ItemId(Uuid::new_v4())
    }
    pub fn is_nil(&self) -> bool {
        self.0.is_nil()
    }
}

impl Default for ItemId {
    fn default() -> Self {
        ItemId(Uuid::nil())
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
    fn id(&self) -> ItemId {
        ItemId::nil()
    }
    fn set_id(&mut self, id: ItemId);
}

#[bevy_reflect::reflect_trait]
pub trait ToItem {
    fn as_item(&self) -> &dyn Item;
    fn to_item(self: Box<Self>) -> Box<dyn Item>;
}

impl<T: Item> ToItem for T {
    fn as_item(&self) -> &dyn Item {
        self as &dyn Item
    }
    fn to_item(self:Box<Self>) -> Box<dyn Item> {
        self as Box<dyn Item>
    }
}

impl PartialEq for dyn Item {
    fn eq(&self, other: &Self) -> bool {
        self.reflect_partial_eq(other.as_reflect()).unwrap()
    }
}
#[cfg(feature = "yew")]
pub trait Extend: Reflect {
    fn yew_obj(&self) -> Option<&dyn crate::items::yew_impl::YewObj> {
        None
    }
    fn yew_simple(&self, ctx: &yew::Context<crate::ObjView>) -> yew::Html {
        if let Some(yew_obj) = self.yew_obj() {
            if let Some(view) = yew_obj.simple(ctx) {view} else {
                yew::html! {
                    <div class="error">
                    {self.type_name()}{" does not impl YewObj::simple"}
                    </div>
                }
            }
        } else {
            yew::html!{
                <div class="error">
                {self.type_name()}{" does not impl YewObj"}
                </div>
            }
        }
    }

    fn yew_edit(&self, ctx: &yew::Context<crate::ObjView>) -> yew::Html {
        if let Some(yew_obj) = self.yew_obj() {
            yew_obj.edit(ctx)
        } else {
            yew::html!{
                <div class="error">
                {self.type_name()}{" does not impl YewObj"}
                </div>
            }
        }
    }

    fn yew_view(&self, ctx: &yew::Context<crate::ObjView>) -> yew::Html {
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

    impl<T> super::Extend for T where T: YewObj {
        fn yew_obj(&self) -> Option<&dyn crate::items::yew_impl::YewObj> {
            Some(self as &dyn YewObj)
        }
    }

    pub trait YewObj: Reflect {
        /// the default view for this item
        fn view(&self, ctx: &Context<ObjView>) -> Html;
        /// a simple view of the item for things like lists or thumbs
        /// none if no simple specified
        #[allow(unused_variables)]
        fn simple(&self, ctx: &Context<ObjView>) -> Option<Html> {
            None
        }
        /// a edit view of the object such as making enums <EnumSelect>
        /// defaults to view
        fn edit(&self, ctx: &Context<ObjView>) -> Html {
            self.view(ctx)
        }
    }

    impl PartialEq for dyn YewObj {
        fn eq(&self, other: &Self) -> bool {
            self.reflect_partial_eq(other.as_reflect()).unwrap()
        }
    }

    #[derive(Debug, Clone)]
    pub struct ObjList{
        cb_id: Option<usize>,
    }

    impl Component for ObjList {
        type Message = ObjMsg;
        type Properties = Props;

        fn create(ctx: &Context<Self>) -> Self {
            web_sys::console::log_1(&"Create List".into());
            if let Some((cb,_)) = ctx.link().context::<crate::components::CallbackReg>(Callback::noop()) {
                let id = cb.reg_cb(ctx.link().callback(|_| ObjMsg::CallBack));
                web_sys::console::log_1(&"Found CBR".into());
                ObjList {
                    cb_id: Some(id),
                }
            } else {
                web_sys::console::log_1(&"No CBR".into());
                ObjList {
                    cb_id: None,
                }
            }
        }

        fn view(&self, ctx: &Context<Self>) -> Html {
            html! {
                <div>
                {for ctx.props().display.iter().map(|item| html!{<ObjView id={item.clone()} edit={false}/>})}
                <button onclick={ctx.link().callback(|_| ObjMsg::Test(0))}>{"add 0"}</button>
                <button onclick={ctx.link().callback(|_| ObjMsg::Test(1))}>{"add 1"}</button>
                <button onclick={ctx.link().callback(|_| ObjMsg::Test(2))}>{"add 2"}</button>
                </div>
            }
        }

        fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
            match msg {
                ObjMsg::Load(item) => {
                    let (cbr, _) = ctx.link().context::<crate::components::CallbackReg>(Callback::noop()).unwrap();
                    cbr.load(item);
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
                    ctx.link().send_message(ObjMsg::Load(Box::new(obj)));
                    false
                },
                ObjMsg::Apply(_) => {todo!()},
                ObjMsg::SetEdit(_) => {todo!()},
                ObjMsg::CallBack => {true}
            }
        }
        fn destroy(&mut self, ctx: &Context<Self>) {
            if let Some(id) = self.cb_id {
                if let Some((cb,_)) = ctx.link().context::<crate::components::CallbackReg>(Callback::noop()) {
                    cb.un_reg_cb(id)
                }
            }
        }
    }

    #[derive(Properties, PartialEq)]
    pub struct ViewProps{
        pub id: ItemId,
        //#[properties_or(false)]
        pub edit: bool,
    }

    pub struct ObjView {
        cb_id: Option<usize>
    }

    impl Component for ObjView {
        type Message = ObjMsg;
        type Properties = ViewProps;
        fn create(ctx: &Context<Self>) -> Self {
            if let Some((cb,_)) = ctx.link().context::<crate::components::CallbackReg>(Callback::noop()) {
                let id = cb.reg_cb(ctx.link().callback(|_| ObjMsg::CallBack));
                ObjView {
                    cb_id: Some(id),
                }
            } else {
                ObjView {
                    cb_id: None,
                }
            }
        }
        fn view(&self, ctx: &Context<Self>) -> Html {
            let (cbr, _) = ctx.link().context::<crate::components::CallbackReg>(Callback::noop()).unwrap();
            let id = ctx.props().id;
            let objs = cbr.read_items();
            if let Some(data) = objs.get(&id) {
                if ctx.props().edit {
                    data.yew_edit(ctx)
                } else {
                    data.yew_view(ctx)
                }
            } else {
                ctx.link().send_message(ObjMsg::Get(id));
                html!{"loading..."}
            }
        }
        fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
            match msg {
                ObjMsg::Apply(data) => {
                    let (cbr, _) = ctx.link().context::<crate::components::CallbackReg>(Callback::noop()).unwrap();
                    cbr.apply(data.as_ref())
                },
                ObjMsg::Get(id) => {
                    web_sys::console::log_1(&"ObjView Get".into());
                    let cb = ctx.link().callback(|msg| msg);
                    let cbr = if let Some((reg, _)) = ctx.link().context::<crate::components::CallbackReg>(Callback::noop()) {reg} else {
                        web_sys::console::error_1(&"filed to get callbackreg from context".into());
                        return false;
                    };
                    wasm_bindgen_futures::spawn_local(async move {
                        let res = gloo_net::http::Request::get(&format!("{}/db_item/{}", CONFIG.server_id, id.0))
                        .send().await
                        .unwrap();
                        if res.status() == 200 {
                            if let Ok(list) = ron::from_str::<ItemData>(&res.text().await.unwrap()) {
                                if let Some(reg) = cbr.type_reg().get_with_name(&list.type_name) {
                                    let deser = if let Some(ser) = reg.data::<ReflectDeserialize>() {
                                        ser
                                    } else {
                                        web_sys::console::error_1(&format!("{} does not reflect Deserialize", reg.type_name()).into());
                                        return;
                                    };
                                    let mut de = ron::Deserializer::from_str(&list.data).unwrap();
                                    match deser.deserialize(&mut de) {
                                        Ok(val) => {
                                            let mut item = match cbr.as_item(val) {
                                                Ok(v) => v,
                                                Err(_) => return,
                                            };
                                            item.set_id(id);
                                            cb.emit(ObjMsg::Load(item))},
                                        Err(e) => {
                                            web_sys::console::error_1(&e.to_string().into());
                                            return;
                                        }
                                    }
                                } else {
                                    web_sys::console::error_1(&format!("{} is not registured with reflect", list.type_name).into());
                                    return;
                                };
                            } else {
                                web_sys::console::error_1(&"ron failed to make item".into());
                            }
                        } else {
                            web_sys::console::error_1(&format!("server responed with: {} status code", res.status()).into())
                        }
                    });
                    false
                },
                ObjMsg::Load(item) => {
                    let (cbr, _) = ctx.link().context::<crate::components::CallbackReg>(Callback::noop()).unwrap();
                    cbr.load(item);
                    false
                },
                ObjMsg::Test(id) => {
                    let obj = match id {
                        0 => super::TestString {id: ItemId::from_u128(0), data: "Test 0".to_string()},
                        1 => super::TestString {id: ItemId::from_u128(1), data: "Loaded".to_string()},
                        2 => super::TestString {id: ItemId::from_u128(1), data: "Alt Loaded".to_string()},
                        3 => super::TestString {id: ItemId::from_u128(0), data: "Alt 0".to_string()},
                        4 => super::TestString {id: ItemId::from_u128(0), data: "Alt 1".to_string()},
                        _ => {return false;}
                    };
                    ctx.link().send_message(ObjMsg::Load(Box::new(obj)));
                    false
                },
                ObjMsg::SetEdit(to) => {
                    let (cbr, _) = ctx.link().context::<crate::components::CallbackReg>(Callback::noop()).unwrap();
                    if let Err(e) = cbr.set(ctx.props().id, "edit", to) {
                        web_sys::console::error_1(&e.to_string().into());
                        false
                    } else {
                        true
                    }
                },
                ObjMsg::CallBack => {true}
            }
        }
        fn destroy(&mut self, ctx: &Context<Self>) {
            if let Some(id) = self.cb_id {
                if let Some((cb,_)) = ctx.link().context::<crate::components::CallbackReg>(Callback::noop()) {
                    cb.un_reg_cb(id)
                }
            }
        }
    }

    pub enum ObjMsg {
        Load(Box<dyn Item>),
        Get(ItemId),
        Test(u8),
        Apply(Box<dyn Item>),
        SetEdit(bool),
        CallBack,
    }

    #[derive(Properties, PartialEq)]
    pub struct Props {
        pub display: Rc<Vec<ItemId>>
    }

    impl YewObj for super::TestString {
        fn view(&self, _ctx: &yew::Context<ObjView>) -> yew::Html {
            yew::html!{
                {&self.data}
            }
        }
    }

    impl Item for bevy_reflect::DynamicStruct {
        fn id(&self) -> ItemId {
            if let Some(v) = self.get_field("id") {
                *v
            } else {
                ItemId(Uuid::nil())
            }
        }
        fn set_id(&mut self, id: ItemId) {
            if let Some(f) = self.get_field_mut::<ItemId>("id") {
                *f = id;
            } else {
                self.insert("id", id);
            }
        }
    }

    impl YewObj for bevy_reflect::DynamicStruct {
        fn view(&self, _ctx: &Context<ObjView>) -> Html {
            html! {
                {format!("{:?}", self)}
            }
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn reflect() {
        use crate::*;
        use crate::plants::*;
        let plant = Plant::test(1);
        let mut type_regisury = bevy_reflect::TypeRegistry::new();
        type_regisury.register::<Plant>();
        type_regisury.register::<PlantTypes>();
        
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
    let data = [data0];
    let obj0 = from_data(&data[0], &type_regisury);
    assert!(obj0.is::<Plant>());
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
    fn set_id(&mut self, id: ItemId) {
        self.id = id;
    }
}
