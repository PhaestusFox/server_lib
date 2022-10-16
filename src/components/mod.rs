use yew::*;
use crate::*;
use strum::IntoEnumIterator;
use std::{str::FromStr, collections::HashMap, sync::RwLock, rc::Rc};
use web_sys::HtmlInputElement;
pub(crate) struct EnumSelect<T> {
    marler: std::marker::PhantomData<T>,
    node: NodeRef,
}

#[derive(Debug, Properties, PartialEq)]
pub(crate) struct ItemCompProps {
    pub target: ItemId,
    pub field: &'static str,
}

impl<T: Reflect + Clone + FromStr + IntoEnumIterator + std::fmt::Display + Into<&'static str>> Component for EnumSelect<T> {
    type Message = ();
    type Properties = ItemCompProps;
    fn create(_ctx: &Context<Self>) -> Self {
        EnumSelect { marler: Default::default(), node: NodeRef::default() }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let target = props.target;
        let field = props.field;
        let (cbr, _) = ctx.link().context::<crate::components::CallbackReg>(Callback::noop()).unwrap();
        let current = cbr.get::<T>(target, field).unwrap_or(T::iter().next().expect("Enum Needs atleast one Vairent"));
        let current: &'static str = current.into();
        web_sys::console::log_1(&current.into());
        let node = self.node.clone();
        html! {
            <select ref={self.node.clone()} onchange={move |_| {
                let val = node.cast::<web_sys::HtmlTextAreaElement>().unwrap().value();
                match T::from_str(&val) {
                    Ok(v) => if let Err(e) = cbr.set::<T>(target, field, v) {web_sys::console::error_1(&e.to_string().into())},
                    Err(_) => {web_sys::console::error_1(&format!{"Failed to parse '{}' to {}", val, std::any::type_name::<T>()}.into());}
                };
            }}>
                {for T::iter().map(|v| {
                    let v: &'static str = v.into();
                    html!{<option selected={current == v}>{v}</option>}
                })}
            </select>
        }
    }
}

pub struct Radio {
    node: NodeRef,
}

#[derive(Debug, Properties, PartialEq)]
pub struct RadioProps {
    pub state: bool,
    pub cb: Callback<ObjMsg>
}

impl Component for Radio {
    type Message = ();
    type Properties = RadioProps;
    fn create(_ctx: &Context<Self>) -> Self {
        Self{node: NodeRef::default()}
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let cb = ctx.props().cb.clone();
        let toggel = !ctx.props().state;
        html!{
            <input ref={self.node.clone()} type="radio" onclick={move |_|
                cb.emit(ObjMsg::SetEdit(toggel))
            } checked={ctx.props().state}/>
        }
    }
}

pub(crate) struct ItemIdInput {
    node: NodeRef,
}

impl Component for ItemIdInput {
    type Message = ();
    type Properties = ItemCompProps;
    fn create(_ctx: &Context<Self>) -> Self {
        ItemIdInput { node: NodeRef::default() }
    }
    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <input class="itemid" ref={self.node.clone()} onchange={_ctx.link().callback(|_| ())}/>
        }
    }
    fn update(&mut self, ctx: &Context<Self>, _: Self::Message) -> bool {
        let msg = self.node.cast::<HtmlInputElement>().expect("ItemIdInput Component is Input node").value();
        let uuid = match Uuid::from_str(&msg) {
            Ok(uuid) => uuid,
            Err(e) => {web_sys::console::error_1(&e.to_string().into()); return false;}
        };
        let props = ctx.props();
        let (cbr, _) = ctx.link().context::<crate::components::CallbackReg>(Callback::noop()).unwrap();
        if let Err(e) = cbr.set(props.target, props.field, ItemId(uuid)) {web_sys::console::error_1(&e.to_string().into()); return false;};
        true
    }
    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let node = self.node.cast::<HtmlInputElement>().expect("ItemIdInput Component is Input node");
            let (cbr, _) = ctx.link().context::<crate::components::CallbackReg>(Callback::noop()).unwrap();
            if let Some(val) = cbr.get::<ItemId>(ctx.props().target, ctx.props().field) {
                node.set_value(&val.to_string())
            } else {
                node.set_value("00000000-0000-0000-0000-000000000000")
            }
        }
    }
}

#[derive(Clone)]
pub struct CallbackReg{
    type_reg: Rc<bevy_reflect::TypeRegistry>,
    loaded_items: Rc<RwLock<HashMap<ItemId, Box<dyn Item>>>>,
    reg: Rc<RwLock<HashMap<usize, Callback<()>>>>,
    next: Rc<RwLock<usize>>,
}

impl PartialEq for CallbackReg {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl CallbackReg {
    pub fn new() -> CallbackReg {
        CallbackReg{
            type_reg: Rc::new(crate::type_registry()),
            loaded_items: Default::default(),
            reg: Default::default(),
            next: Default::default(),
        }
    }

    pub fn get_itemdata(&self, id: ItemId) -> Option<ItemData> {
        let loaded = self.loaded_items.read().unwrap();
        let item = loaded.get(&id)?;
        let reg = if let Some(reg) = self.type_reg.get(item.type_id()) {reg} else {
            web_sys::console::warn_1(&format!("{}: type not registred", item.type_name()).into());
            return None;
        };
        let traitid = reg.data::<ReflectSerialize>()?;
        let data = match traitid.get_serializable(item.as_reflect()) {
            bevy_reflect::serde::Serializable::Owned(v) => ron::to_string(&v).ok()?,
            bevy_reflect::serde::Serializable::Borrowed(v) => ron::to_string(v).ok()?,
        };
        Some(ItemData { type_name: reg.type_name().to_string(), data })
    }

    pub fn as_item(&self, item: Box<dyn Reflect>) -> Result<Box<dyn Item>, Box<dyn Reflect>> {
        let reg = if let Some(v) = self.type_reg.get(item.type_id()) {v} else {web_sys::console::warn_1(&format!("type {} is not in registry", item.type_name()).into()); return Err(item);};
        let reg = if let Some(v) = reg.data::<crate::items::ReflectToItem>() {v} else {web_sys::console::warn_1(&format!("type {} does no registor to_item", item.type_name()).into()); return Err(item);};
        let item = reg.get_boxed(item)?;
        Ok(item.to_item())
    }

    pub fn type_reg(&self) -> Rc<bevy_reflect::TypeRegistry> {
        self.type_reg.clone()
    }

    pub fn reg_cb(&self, cb: Callback<()>) -> usize {
        let mut g = self.reg.write().unwrap();
        let mut next = self.next.write().unwrap();
        let id = *next;
        *next += 1;
        g.insert(id, cb);
        id
    }

    pub fn un_reg_cb(&self, id: usize) {
        let mut g = self.reg.write().unwrap();
        g.remove(&id);
    }

    pub fn read_items(&self) -> std::sync::RwLockReadGuard<std::collections::HashMap<ItemId, Box<dyn Item>>> {
        match self.loaded_items.read() {
            Ok(v) => v,
            Err(e) => {panic!("{}",e)}
        }
    }

    fn write_items(&self) -> std::sync::RwLockWriteGuard<std::collections::HashMap<ItemId, Box<dyn Item>>>{
        self.loaded_items.write().unwrap()
    }
    pub fn load(&self, mut item: Box<dyn Item>) -> ItemId {
        let id = if item.id().is_nil() {
            let id = ItemId::new();
            item.set_id(id);
            id
        } else {
            item.id()
        };
        self.write_items().insert(id, item);
        self.emit();
        id
    }
    pub fn get<T: Reflect + Clone>(&self, item: ItemId, field: &'static str) -> Option<T> {
        use bevy_reflect::ReflectRef;
        match self.read_items().get(&item)?.reflect_ref() {
            ReflectRef::Struct(s) => {s.get_field::<T>(field).cloned()},
            _ => unimplemented!(),
        }
    }
    pub fn set<T: Reflect>(&self, item: ItemId, field: &'static str, new_val: T) -> anyhow::Result<()> {
        use bevy_reflect::ReflectMut;
        match self.write_items().get_mut(&item).ok_or(anyhow::anyhow!("failed to find item {:?}", item))?.reflect_mut() {
            ReflectMut::Struct(s) => {if let Some(val) = s.get_field_mut::<T>(field) {
                *val = new_val;
            }},
            _ => unimplemented!()
        }
        self.emit();
        Ok(())
    }

    pub fn apply(&self, item: &dyn Item) -> bool {
        if let Some(obj) = self.write_items().get_mut(&item.id()) {
            obj.apply(item.as_reflect());
            if let (bevy_reflect::ReflectRef::Struct(s), bevy_reflect::ReflectRef::Struct(d)) = (obj.reflect_ref(), item.reflect_ref()) {
                web_sys::console::log_1(&format!("{:?}:{:?}", s.get_field::<crate::worms::Gender>("gender"), d.field_len()).into());
            }
            self.emit();
            true
        } else {
            false
        }
    }

    fn emit(&self) {
        for cb in self.reg.read().unwrap().values() {
            cb.emit(());
        }
    }
}