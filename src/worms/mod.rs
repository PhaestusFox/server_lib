use std::sync::Arc;

use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Reflect, Clone, Copy)]
#[reflect_value(Deserialize, Serialize)]
enum Gender {
    Male,
    SuspectedMale,
    Female,
    SuspectedFemale,
    UnSexed,
}

impl std::fmt::Display for Gender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Deserialize, Serialize, Reflect, Clone, Copy)]
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

#[derive(Debug, Deserialize, Serialize, Reflect, Clone, Copy)]
#[reflect_value(Deserialize, Serialize)]
enum Stage {
    Hatched,
    Isolated,
    Pupate,
    Emerge,
    Dead,
}

impl std::fmt::Display for Stage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Deserialize, Serialize, Reflect, Clone)]
#[reflect(Deserialize, Serialize)]
struct Individual {
    id: ItemId,
    #[serde(skip)]
    edit: bool,
    worm_type: WormType,
    stage: Stage,
    origin: ItemId,
    gender: Gender,
    location: ItemId,
}

#[derive(Debug, Deserialize, Serialize, Reflect, Clone)]
#[reflect(Deserialize, Serialize)]
struct Brood {
    id: ItemId,
    parents: Vec<ItemId>,
    diet: ItemId,
}

impl Item for Brood {
    fn id(&self) -> ItemId {
        self.id
    }
}

impl Item for Individual {
    fn id(&self) -> ItemId {
        self.id
    }
    fn dependencies(&self) -> Option<Vec<ItemId>> {
        Some(vec![self.origin, self.location])
    }
}

#[cfg(feature = "yew")]
pub mod yew {
    use std::rc::Rc;
    use yew::*;

    use crate::events::Item;
    use crate::events::YewObj;
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
        fn view(&self, _: &Context<crate::events::yew_impl::ObjList>) -> Html {
            html!{
                <p>{"hi i am "}{&self.name}</p>
            }
        }
    }

    impl Item for Test {
        fn id(&self) -> ItemId {
            todo!()
        }
    }

    #[derive(Reflect)]
    pub struct OtherTest(pub u8);

    impl Item for OtherTest {
        fn id(&self) -> ItemId {
            todo!()
        }
    }

    impl YewObj for OtherTest {
        fn view(&self, ctx: &Context<crate::ObjList>) -> Html {
            html! {
                if let Some(obj) = LoadedItems3::read().get(&ItemId::from_u128(0)) {
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
        fn id(&self) -> ItemId {
        todo!()
    }
    }

    impl YewObj for LoadTest {
        fn view(&self, ctx: &Context<crate::ObjList>) -> Html {
            html! {
                if let Some(data) = LoadedItems3::read().get(&ItemId::from_u128(1)) {
                    if let Some(obj) = data.yew_obj() {
                        {obj.view(ctx)}<br/>
                    } else {
                        {"cant view obj"}
                    }
                } else {
                    {"loading..."}<br/>
                }
            }
        }
    }

    impl YewObj for super::Individual {
        fn view(&self, ctx: &Context<crate::events::ObjList>) -> Html {
            /*
                id: ItemId,
                worm_type: WormType,
                stage: Stage,
                orgin: ItemId,
                gender: Gender,
                location: ItemId,
            */
            let objs = LoadedItems3::read();
            html!{
                <div>
                <h1>{self.worm_type}</h1><br/>
                <strong class="stage">{"stage: "}{self.stage}</strong><br/>
                <strong class="origin">{"from: "} if let Some(origin) = objs.get(&self.origin) {
                    {origin.yew_view(ctx)}
                } else {
                    {{ctx.link().send_message(ObjMsg::Get(self.origin)); "Origin not loaded"}}
                }</strong><br/>
                <strong class="gender">{"Gender: "}{self.gender}</strong><br/>
                <strong class="location">{"at: "} if let Some(location) = objs.get(&self.location) {
                    {location.yew_view(ctx)}
                } else {
                    {{ctx.link().send_message(ObjMsg::Get(self.location)); "Location not loaded"}}
                }</strong>
                </div>
            }
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
        fn view(&self, ctx: &Context<ObjList>) -> Html {
            let objs = LoadedItems3::read();
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
pub fn load_test_worm() -> Vec<ItemId> {
    use crate::*;
    let individual0 = LoadedItems3::load(None, Box::new(Individual {
        id: ItemId::from_u128(10),
        edit: true,
        worm_type: WormType::KingWorm,
        gender: Gender::Male,
        stage: Stage::Hatched,
        origin: ItemId::from_u128(11),
        location: ItemId::from_u128(12)
    }));
    let individual1 = LoadedItems3::load(None, Box::new(Individual {
        id: ItemId::from_u128(14),
        edit: false,
        worm_type: WormType::KingWorm,
        gender: Gender::Female,
        stage: Stage::Isolated,
        origin: ItemId::from_u128(11),
        location: ItemId::from_u128(15)
    }));
    LoadedItems3::load(None, Box::new(Brood {
        id: ItemId::from_u128(11),
        parents: vec![ItemId::from_u128(1), ItemId::from_u128(2)],
        diet: ItemId::from_u128(13),
    }));
    vec![individual0, individual1]
}