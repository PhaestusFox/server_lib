use crate::*;
use enum_utils::FromStr;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoStaticStr};

pub fn register_types(reg: &mut bevy_reflect::TypeRegistry) {
    reg.register::<Gender>();
    reg.register::<WormType>();
    reg.register::<Stage>();
}

#[derive(Debug, Deserialize, Serialize, Reflect, Clone, Copy, FromStr, EnumIter, IntoStaticStr)]
#[reflect_value(Deserialize, Serialize)]
pub enum Gender {
    Unknown,
    Male,
    SuspectedMale,
    Female,
    SuspectedFemale,
}

impl std::fmt::Display for Gender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Deserialize, Serialize, Reflect, Clone, Copy, FromStr, EnumIter, IntoStaticStr)]
#[reflect_value(Deserialize, Serialize)]
enum WormType {
    KingWorm,
    SilkWorm,
}

impl std::fmt::Display for WormType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Deserialize, Serialize, Reflect, Clone)]
#[reflect(Deserialize, Serialize)]
enum Event {
    ChangedStage(ItemId, Stage, Date),
    AssinedGender(ItemId, Gender, Date),
    Count(ItemId, usize, Date),
    Weight(ItemId, f32, Date),
    Clean(ItemId, Date),
    Added(ItemId, ItemId),
}

#[derive(Debug, Deserialize, Serialize, Reflect, Clone, Copy, FromStr, EnumIter, IntoStaticStr)]
#[reflect_value(Deserialize, Serialize)]
enum Stage {
    Egg,
    Larvae,
    Isolated,
    Pupa,
    Adult,
    Dead,
}

impl std::fmt::Display for Stage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Deserialize, Serialize, Reflect, Clone)]
#[reflect(Deserialize, Serialize, Default)]
struct Worm {
    #[serde(skip)]
    id: ItemId,
    worm_type: WormType,
    stage: Stage,
    origin: ItemId,
    gender: Gender,
    location: ItemId,
}

impl Default for Worm {
    fn default() -> Self {
        Worm {
            id: ItemId::default(),
            worm_type: WormType::SilkWorm,
            stage: Stage::Egg,
            origin: ItemId::nil(),
            gender: Gender::Unknown,
            location: ItemId::nil()
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Reflect, Clone)]
#[reflect(Deserialize, Serialize)]
struct Brood {
    #[serde(skip)]
    id: ItemId,
    parents: Vec<ItemId>,
    diet: ItemId,
}

impl Item for Brood {
    fn id(&self) -> ItemId {
        self.id
    }
    fn set_id(&mut self, id: ItemId) {
        self.id = id;
    }
}

impl Item for Worm {
    fn id(&self) -> ItemId {
        self.id
    }
    fn dependencies(&self) -> Option<Vec<ItemId>> {
        Some(vec![self.origin, self.location])
    }
    fn set_id(&mut self, id: ItemId) {
        self.id = id;
    }
}

#[cfg(feature = "yew")]
pub mod yew {
    use std::rc::Rc;
    use yew::*;

    use crate::items::Item;
    use crate::items::YewObj;
    use crate::*;

    #[derive(Properties)]
    struct VProps {
        obj: Rc<dyn YewObj>,
    }

    impl PartialEq for VProps {
        fn eq(&self, other: &Self) -> bool {
            self.obj.reflect_partial_eq(other.obj.as_reflect()).unwrap()
        }
    }

    #[derive(Debug, Reflect)]
    pub struct Test {
        pub name: String,
    }

    impl YewObj for Test {
        fn view(&self, _: &Context<crate::items::yew_impl::ObjView>) -> Html {
            html!{
                <p>{"hi i am "}{&self.name}</p>
            }
        }
    }

    impl Item for Test {
        fn set_id(&mut self, _id: ItemId) {
        }
    }

    #[derive(Reflect)]
    pub struct OtherTest(pub u8);

    impl Item for OtherTest {
        fn set_id(&mut self, _id: ItemId) {
            
        }
    }

    impl YewObj for OtherTest {
        fn view(&self, ctx: &Context<crate::ObjView>) -> Html {
            let (cbr, _) = ctx.link().context::<crate::components::CallbackReg>(Callback::noop()).unwrap();
            let objs = cbr.read_items();
            html! {
                if let Some(obj) = objs.get(&ItemId::from_u128(0)) {
                    <div>
                    {obj.yew_obj().unwrap().view(ctx)}<br/>
                    <button onclick={
                        let r = self.0;
                        ctx.link().callback(move |_| ObjMsg::Test(r))}>{"click to load alt"}</button>
                    </div>
                } else {
                    <div>{"not loaded"}<br/>
                    <button onclick={ctx.link().callback(|_| ObjMsg::Test(0))}>{"click to load"}</button>
                    </div>
                }
            }
        }
    }

    #[derive(Reflect)]
    pub struct LoadTest;
    impl Item for LoadTest {
        fn set_id(&mut self, _id: ItemId) {
            
        }
    }

    impl YewObj for LoadTest {
        fn view(&self, ctx: &Context<crate::ObjView>) -> Html {
            let (cbr, _) = ctx.link().context::<crate::components::CallbackReg>(Callback::noop()).unwrap();
            let objs = cbr.read_items();
            html! {
                <>
                if let Some(data) = objs.get(&ItemId::from_u128(1)) {
                    if let Some(obj) = data.yew_obj() {
                        {obj.view(ctx)}<br/>
                    } else {
                        {"cant view obj"}
                    }
                } else {
                    {"loading..."}<br/>
                }
                </>
            }
        }
    }

    impl YewObj for super::Worm {
        fn view(&self, ctx: &Context<crate::ObjView>) -> Html {
            let (cbr, _) = ctx.link().context::<crate::components::CallbackReg>(Callback::noop()).unwrap();
            let objs = cbr.read_items();
            let id = self.id();
            html!{
                <div id={id.to_string()}>
                <div class="tooltip"><strong>{self.worm_type}</strong>
                <span class="tooltiptext">{self.id.to_string()}</span>
                </div><br/>
                <strong class="stage">{"stage: "}{self.stage}</strong><br/>
                <strong class="gender">{"Gender: "}{self.gender}</strong><br/>
                <strong class="origin">{"from: "} if let Some(origin) = objs.get(&self.origin) {
                    {origin.yew_view(ctx)}
                } else {
                    {{ctx.link().send_message(ObjMsg::Get(self.origin)); "Origin not loaded"}}
                }</strong><br/>
                <strong class="location">{"at: "} if let Some(location) = objs.get(&self.location) {
                    {location.yew_view(ctx)}
                } else {
                    {{ctx.link().send_message(ObjMsg::Get(self.location)); "Location not loaded"}}
                }</strong><br/>
                </div>
            }
        }
        fn edit(&self, ctx: &Context<ObjView>) -> Html {
            let id = ctx.props().id;
            html! {
                <div id={id.to_string()}>
                    {"Type: "    } <crate::components::EnumSelect<super::WormType> target={id} field="worm_type"/><br/>
                    {"Gender: "  } <crate::components::EnumSelect<super::Gender> target={id} field="gender"/><br/>
                    {"Stage: "   } <crate::components::EnumSelect<super::Stage> target={id} field="stage"/><br/>
                    {"Origin: "  } <crate::components::ItemIdInput target={id} field="origin"/><br/>
                    {"Location: "} <crate::components::ItemIdInput target={id} field="location"/><br/>
                </div>
            }
        }
        fn simple(&self, ctx: &Context<ObjView>) -> Option<Html> {
            let id = ctx.props().id;
            Some(html!{
                <div id={id.to_string()}>
                    <h5 class="tooltip">{self.worm_type}</h5>
                    <span class="tooltiptext">{self.id.to_string()}</span><br/>
                    <h6>{self.gender}</h6><br/>
                    <h6>{self.stage}</h6><br/>
                </div>
            })
        }
    }

    impl YewObj for super::Brood {
        /*
        struct Brood {
            id: ItemId,
            parents: Vec<ItemId>,
            diet: ItemId,
        }
        */
        fn view(&self, ctx: &Context<ObjView>) -> Html {
            let (cbr, _) = ctx.link().context::<crate::components::CallbackReg>(Callback::noop()).unwrap();
            let objs = cbr.read_items();
            html! {
                <div class="brood">
                <strong>{"brood: "}{self.id.to_string()}</strong><br/>
                <strong>{"from: "}{self.parents.len()}</strong><br/>
                if let Some(diet) = objs.get(&self.diet) {
                    {diet.yew_view(ctx)}
                } else {
                    {{ctx.link().send_message(ObjMsg::Get(self.diet)); "diet loading"}}
                }
                </div>
            }
        }
    }
}

#[cfg(feature = "yew")]
pub fn load_test_worm(cbr: &crate::components::CallbackReg) -> Vec<ItemId> {
    use crate::*;
    let individual0 = cbr.load(Box::new(Worm {
        id: ItemId::from_u128(10),
        worm_type: WormType::KingWorm,
        gender: Gender::SuspectedMale,
        stage: Stage::Larvae,
        origin: ItemId::from_u128(11),
        location: ItemId::from_u128(12)
    }));
    let individual1 = cbr.load(Box::new(Worm {
        id: ItemId::from_u128(14),
        worm_type: WormType::SilkWorm,
        gender: Gender::Female,
        stage: Stage::Isolated,
        origin: ItemId::from_u128(11),
        location: ItemId::from_u128(15)
    }));
    cbr.load(Box::new(Brood {
        id: ItemId::from_u128(11),
        parents: vec![ItemId::from_u128(1), ItemId::from_u128(2)],
        diet: ItemId::from_u128(13),
    }));
    vec![individual0, individual1]
}