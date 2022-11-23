use std::{collections::HashMap, borrow::Cow};
use bevy_reflect::{Reflect, TypeRegistry, ReflectSerialize, ReflectDeserialize, GetTypeRegistration, TypeRegistration};
use ::serde::{Serialize, Deserialize};
use strum::{IntoStaticStr, EnumIter, IntoEnumIterator};
use crate::ItemId;
use enum_utils::FromStr;

mod serde;

#[cfg(feature = "yew")]
use yew::*;

type FiledType = Cow<'static, str>;
// type FiledType = &'static str;
#[derive(Default)]
pub struct Fields(Vec<Box<dyn CropData>>);
impl Fields {
    #[inline(always)]
    fn contains_key(&self, key: &FiledType) -> bool {
        for i in self.0.iter() {
            if &i.fild_type() == key {
                return true;
            }
        }
        false
    }
    fn push<T: CropData>(&mut self, item: T) {
        self.0.push(Box::new(item));
    }
    fn set(&mut self, item: Box<dyn CropData>) {
        for val in self.0.iter_mut() {
            if val.fild_type() == item.fild_type() {
                *val = item;
                return;
            }
        }
        self.0.push(item);
    }
    fn iter(&self) -> impl Iterator<Item = &Box<dyn CropData>> {
        self.0.iter()
    }
    fn into_iter(self) -> impl Iterator<Item = Box<dyn CropData>> {
        self.0.into_iter()
    }
    fn get(&self, key: &FiledType) -> Option<&Box<dyn CropData>> {
        for val in self.0.iter() {
            if val.fild_type() == *key {
                return Some(val);
            } 
        }
        None
    }
    fn clear(&mut self) {
        self.0.clear()
    }
}

#[derive(Reflect, Default)]
pub struct Crate {
    pub id: ItemId,
    pub crop: Crop,
    #[reflect(ignore)]
    pub fields: Fields,
}

impl Crop {
    fn default_filds(&self) -> Fields {
        let mut map = Fields::default();
        match self {
            Crop::None => {},
            Crop::Tomato => {
                map.push(TomatoType::Normal);
                map.push(Size::Small);
                map.push(Grade::First);
                map.push(Packing::Truss);
            }
            Crop::Parsley => {
                map.push(ParsleyType::Curly);
            }
        }
        map
    }
}

#[derive(Debug, Default, IntoStaticStr, Reflect, Clone, FromStr, PartialEq, Eq, EnumIter, Copy, ::serde::Serialize, ::serde::Deserialize)]
pub enum Crop {
    #[default]
    None,
    Tomato,
    Parsley,
}

#[bevy_reflect::reflect_trait]
pub trait CropData: Reflect {
    #[cfg(feature = "yew")]
    fn view(&self) -> yew::Html;
    #[cfg(feature = "yew")]
    fn edit(&self, cb: yew::Callback<CrateMsg>, node: &yew::NodeRef) -> yew::Html;
    fn fild_type(&self) -> FiledType;
    fn get_type_registration(&self) -> TypeRegistration;
}

#[cfg(feature = "yew")]
pub struct CrateView {
    nodes: HashMap<FiledType, yew::NodeRef>,    
}
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "yew", derive(yew::Properties))]
pub struct ViewProps {
    pub id: ItemId,
    pub edit: bool,
}

pub enum CrateMsg {
    SetCrop(Crop),
    SetField(Box<dyn CropData>),
}

#[cfg(feature = "yew")]
impl yew::Component for CrateView {
    type Message = CrateMsg;
    type Properties = ViewProps;
    fn create(ctx: &yew::Context<Self>) -> Self {
        let (cbr, _) = ctx.link().context::<crate::components::CallbackReg>(Callback::noop()).unwrap();
        let crop = cbr.get::<Crop>(ctx.props().id, "crop").expect("valid id");
        let mut nodes = HashMap::default();
        nodes.insert("Crop".into(), NodeRef::default());
        for f in crop.default_filds().into_iter() {
            let v = f.fild_type();
            web_sys::console::log_1(&v.to_string().into());
            nodes.insert(v, NodeRef::default());
        }
        CrateView {
            nodes
        }
    }
    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        let (cbr, _) = ctx.link().context::<crate::components::CallbackReg>(Callback::noop()).unwrap();
        let mut items = cbr.write_items();
        let id = ctx.props().id;
        let cb = ctx.link().callback(|msg| msg);
        let Some(gh_crate) = items.get_mut(&id) else {return html! {
            <p>{format!("item {:?} not loaded", id)}</p>
        };};
        let Some(gh_crate) = gh_crate.as_reflect_mut().downcast_mut::<Crate>() else {
            return html! {<p>{format!("item {:?} is not a crate", id)}</p>}
        };
        html! {
            <>
            <h1>{"THIS IS THE TEST CRATE"}</h1>
            if ctx.props().edit {
                <>
                {gh_crate.crop.edit(ctx.link().callback(|msg| msg), self.nodes.get(&Cow::Borrowed("Crop")).unwrap())}
                {for gh_crate.crop.default_filds().into_iter().map(|v| {
                    let f = v.fild_type();
                    if gh_crate.fields.contains_key(&f) {
                        gh_crate.fields.get(&f).unwrap().edit(cb.clone(), self.nodes.get(&f).unwrap())
                    } else {
                        let res = v.edit(cb.clone(), self.nodes.get(&f).unwrap());
                        gh_crate.fields.set(v);
                        res
                    }
                })}
                </>
            } else {
                <>{gh_crate.crop.view()}
                {for gh_crate.fields.iter().map(|d| d.view())}</>
            }
            </>
        }
    }
    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        let (cbr, _) = ctx.link().context::<crate::components::CallbackReg>(Callback::noop()).unwrap();
        let crop = cbr.get::<Crop>(ctx.props().id, "crop").expect("valid id");
        for f in crop.default_filds().into_iter() {
            let f = f.fild_type();
            if !self.nodes.contains_key(&f) {
                self.nodes.insert(f, NodeRef::default());
            }
        }
        true
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let (cbr, _) = ctx.link().context::<crate::components::CallbackReg>(Callback::noop()).unwrap();
        let mut items = cbr.write_items();
        match msg {
            CrateMsg::SetCrop(crop) => {
                let Some(item) = items.get_mut(&ctx.props().id) else {
                    web_sys::console::error_1(&format!("Item {:?} not Loaded", ctx.props().id).into());
                    return false;
                };
                let Some(gh_crate) = item.as_reflect_mut().downcast_mut::<Crate>() else {
                    web_sys::console::error_1(&format!("Item {:?} is not Crate", ctx.props().id).into());
                    return false;
                };
                gh_crate.crop = crop;
                gh_crate.fields.clear();
                true
            }
            CrateMsg::SetField(val) => {
                let Some(item) = items.get_mut(&ctx.props().id) else {
                    web_sys::console::error_1(&format!("Item {:?} not Loaded", ctx.props().id).into());
                    return false;
                };
                let Some(gh_crate) = item.as_reflect_mut().downcast_mut::<Crate>() else {
                    web_sys::console::error_1(&format!("Item {:?} is not Crate", ctx.props().id).into());
                    return false;
                };
                gh_crate.fields.set(val);
                true
            }
        }
    }
}

// struct CropDataComponent;
// struct CropDataPros {
//     id: ItemId,
//     edit: bool,
// }
// #[cfg(feature = "yew")]
// impl yew::Component for CropDataComponent {
    
// }

#[derive(Debug, IntoStaticStr, EnumIter, FromStr, Reflect, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[reflect(CropData, Serialize, Deserialize)]
enum Size {
    Small,
    Normal,
    Large,
}

#[derive(Debug, IntoStaticStr, EnumIter, FromStr, Reflect, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[reflect(CropData, Serialize, Deserialize)]
enum Packing {
    SmallPunnet,
    LargePunnet,
    Truss,
}

impl CropData for Size {
    #[cfg(feature = "yew")]
    fn view(&self) -> yew::Html {
        html! {
            <h2>{format!("SIZE = {:?}", self)}</h2>
        }
    }
    #[cfg(feature = "yew")]
    fn edit(&self, cb: yew::Callback<CrateMsg>, node: &yew::NodeRef) -> yew::Html {
        let node = node.clone();
        html! {
            <div class="size">
            <h2>{"select SIZE"}</h2>
            <select ref={node.clone()} onchange={move |_| {
                let val = node.cast::<web_sys::HtmlTextAreaElement>().unwrap().value();
                match val.parse::<Size>() {
                    Ok(v) => cb.emit(CrateMsg::SetField(Box::new(v))),
                    Err(_) => {web_sys::console::error_1(&format!{"Failed to parse '{}' to Size", val}.into());}
                };
            }}>
            {for Size::iter().map(|v| {
                let val: &'static str = v.into();
                html!{<option selected={*self == v}>{val}</option>}
            })}
            </select>
            </div>
        }
    }
    fn fild_type(&self) -> FiledType {
        "Size".into()
    }
    fn get_type_registration(&self) -> TypeRegistration {
        <Self as bevy_reflect::GetTypeRegistration>::get_type_registration()
    }
}

impl CropData for Crop{
    #[cfg(feature = "yew")]
    fn view(&self) -> yew::Html {
        html! {
            <h2>{format!("Crop = {:?}", self)}</h2>
        }
    }
    #[cfg(feature = "yew")]
    fn edit(&self, cb: yew::Callback<CrateMsg>, node: &yew::NodeRef) -> yew::Html {
        let node = node.clone();
        html! {
            <div class="crop">
            <h2>{"select Crop"}</h2>
            <select ref={node.clone()} onchange={move |_| {
                let val = node.cast::<web_sys::HtmlTextAreaElement>().unwrap().value();
                match val.parse::<Crop>() {
                    Ok(v) => cb.emit(CrateMsg::SetCrop(v)),
                    Err(_) => {web_sys::console::error_1(&format!{"Failed to parse '{}' to Size", val}.into());}
                };
            }}>
            {for Crop::iter().map(|v| {
                let val: &'static str = v.into();
                html!{<option selected={*self == v}>{val}</option>}
            })}
            </select>
            </div>
        }
    }
    fn fild_type(&self) -> FiledType {
        "Crop".into()
    }
    fn get_type_registration(&self) -> TypeRegistration {
        <Self as bevy_reflect::GetTypeRegistration>::get_type_registration()
    }
}

impl crate::Item for Crate {
    fn id(&self) -> ItemId {
        self.id
    }
    fn set_id(&mut self, id: ItemId) {
        self.id = id;
    }
}

#[cfg(feature = "yew")]
impl crate::YewObj for Crate {
    fn view(&self, ctx: &Context<crate::ObjView>) -> Html {
        todo!()
    }

    fn view_no_context(&self) -> Html {
        todo!()
    }
}

#[derive(Debug, Reflect, PartialEq, Eq, IntoStaticStr, EnumIter, Clone, Copy, FromStr, Serialize, Deserialize)]
#[reflect(CropData, Serialize, Deserialize)]
pub enum Grade {
    First,
    Second,
    Third,
}

impl CropData for Grade {
    #[cfg(feature = "yew")]
    fn view(&self) -> yew::Html {
        html! {
            <h2>{format!("Grade = {:?}", self)}</h2>
        }
    }
    #[cfg(feature = "yew")]
    fn edit(&self, cb: yew::Callback<CrateMsg>, node: &yew::NodeRef) -> yew::Html {
        let fild = self.fild_type();
        let node = node.clone();
        html! {
            <div class={fild.to_string()}>
            <h2>{format!("Select {}", fild)}</h2>
            <select ref={node.clone()} onchange={move |_| {
                let val = node.cast::<web_sys::HtmlTextAreaElement>().unwrap().value();
                match val.parse::<Self>() {
                    Ok(v) => {
                        cb.emit(CrateMsg::SetField(Box::new(v)))
                    },
                    Err(_) => {web_sys::console::error_1(&format!{"Failed to parse '{}' to {}", val, fild}.into());}
                };
            }}>
            {for Self::iter().map(|v| {
                let val: &'static str = v.into();
                html!{<option selected={*self == v}>{val}</option>}
            })}
            </select>
            </div>
        }
    }
    fn fild_type(&self) -> FiledType {
        "Grade".into()
    }
    fn get_type_registration(&self) -> TypeRegistration {
        <Self as bevy_reflect::GetTypeRegistration>::get_type_registration()
    }
}

#[cfg(feature = "yew")]
fn gen_edit<T>(item: &T, cb: yew::Callback<CrateMsg>, node: yew::NodeRef) -> yew::Html
where T: CropData + std::str::FromStr + IntoEnumIterator + Into<&'static str> + PartialEq + Copy {
    let fild = item.fild_type();
    html! {
        <div class={fild.to_string()}>
        <h2>{format!("Select {}", fild)}</h2>
        <select ref={node.clone()} onchange={move |_| {
            let val = node.cast::<web_sys::HtmlTextAreaElement>().unwrap().value();
            match val.parse::<T>() {
                Ok(v) => {
                    cb.emit(CrateMsg::SetField(Box::new(v)))
                },
                Err(_) => {web_sys::console::error_1(&format!{"Failed to parse '{}' to {}", val, fild}.into());}
            };
        }}>
        {for T::iter().map(|v| {
            let val: &'static str = v.into();
            html!{<option selected={*item == v}>{val}</option>}
        })}
        </select>
        </div>
    }
}

impl CropData for Packing {
    fn view(&self) -> yew::Html {
        html! {
            <h2>{format!("Grade = {:?}", self)}</h2>
        }
    }
    fn edit(&self, cb: yew::Callback<CrateMsg>, node: &yew::NodeRef) -> yew::Html {
        gen_edit(self, cb, node.clone())
    }
    fn fild_type(&self) -> FiledType {
        "Packing".into()
    }
    fn get_type_registration(&self) -> TypeRegistration {
        <Self as bevy_reflect::GetTypeRegistration>::get_type_registration()
    }
}

#[derive(Debug, IntoStaticStr, EnumIter, FromStr, Reflect, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[reflect(CropData, Serialize, Deserialize)]
enum ParsleyType {
    Curly,
    Flat
}

impl CropData for ParsleyType {
    fn view(&self) -> yew::Html {
        html! {
            <h2>{format!("Type = {:?}", self)}</h2>
        }
    }
    fn edit(&self, cb: yew::Callback<CrateMsg>, node: &yew::NodeRef) -> yew::Html {
        gen_edit(self, cb, node.clone())
    }
    fn fild_type(&self) -> FiledType {
        "Type".into()
    }
    fn get_type_registration(&self) -> TypeRegistration {
        <Self as bevy_reflect::GetTypeRegistration>::get_type_registration()
    }
}

#[derive(Debug, IntoStaticStr, EnumIter, FromStr, Reflect, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[reflect(CropData, Serialize, Deserialize)]
enum TomatoType {
    Normal,
    Cherry
}

impl CropData for TomatoType {
    fn view(&self) -> yew::Html {
        html! {
            <h2>{format!("Type = {:?}", self)}</h2>
        }
    }
    fn edit(&self, cb: yew::Callback<CrateMsg>, node: &yew::NodeRef) -> yew::Html {
        gen_edit(self, cb, node.clone())
    }
    fn fild_type(&self) -> FiledType {
        "Type".into()
    }
    fn get_type_registration(&self) -> TypeRegistration {
        <Self as bevy_reflect::GetTypeRegistration>::get_type_registration()
    }
}

#[test]
fn serde_test() {
    use ::serde::de::DeserializeSeed;
    let mut type_reg = bevy_reflect::TypeRegistry::default();
    register_types(&mut type_reg);
    let gh_crate = Crate {
        id: ItemId::from_u128(777),
        crop: Crop::Tomato,
        fields: Crop::Tomato.default_filds()
    };
    let ser = ron::to_string(&serde::CrateSerializer::new(&gh_crate, &type_reg)).unwrap();
    println!("ser = {}", ser);
    let deserializer = serde::CrateDeserializer {
        type_registry: &type_reg
    };
    let mut ron_de = ron::Deserializer::from_str(&ser).unwrap();
    let de = deserializer.deserialize(&mut ron_de).unwrap();
    println!("de = {:?}", de.crop);
    for f in de.fields.iter() {
        println!("{}", f.fild_type())
    }
}

fn register_types(type_reg: &mut TypeRegistry) {
    type_reg.register::<Crate>();
    type_reg.register::<Crop>();
    type_reg.register::<TomatoType>();
    type_reg.register::<Packing>();
    type_reg.register::<Size>();
    type_reg.register::<ParsleyType>();
    type_reg.register::<Grade>();
}