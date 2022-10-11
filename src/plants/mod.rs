use enum_utils::FromStr;
use serde::{Serialize, Deserialize};
use strum::{IntoStaticStr, EnumIter};
use bevy_reflect::prelude::*;
use crate::*;
use derive_more::Display;

#[derive(Debug, FromStr, IntoStaticStr, EnumIter, Serialize, Deserialize, Clone, Reflect, Display, Copy, PartialEq)]
#[reflect_value(Serialize, Deserialize)]
pub enum PlantTypes {
    Sunflower,
    Silverbeet,
    Spinach,
    Sage,
    Lettuce,
    Basil,
    LemonBalm,
    Thyme,
    Snapdragon,
    Daisy,
    Nesturtium,
}

#[derive(Debug, FromStr, IntoStaticStr, EnumIter, Serialize, Deserialize, Clone, Reflect, Display, Copy, PartialEq)]
#[reflect_value(Serialize, Deserialize)]
pub enum Stage {
    Planted,
    Sprout,
    Flowering,
    Frugting,
    Dead,
}

#[derive(Debug, Deserialize, Serialize, Reflect, Clone)]
#[reflect(Deserialize, Serialize)]
enum Event {
    ChangedStage(ItemId, Stage, Date),
}
#[derive(Debug, Serialize, Deserialize, Reflect, PartialEq)]
#[reflect(Serialize, Deserialize, PartialEq)]
pub struct Plant {
    id: ItemId,
    plant_type: PlantTypes,
    stage: Stage,
    location: ItemId,
}

impl Item for Plant {
    fn id(&self) -> ItemId {
        self.id
    }
    fn dependencies(&self) -> Option<Vec<ItemId>> {
        Some(vec![self.location])
    }
    fn set_id(&mut self, id: ItemId) {
        self.id = id;
    }
}

#[reflect_trait]
trait Location {
    fn location_of(&self, find: ItemId) -> Option<String>;
}

#[derive(Debug, Serialize, Deserialize, Reflect, PartialEq)]
#[reflect(Serialize, Deserialize, PartialEq, Location)]
pub struct SeedTray {
    #[serde(skip)]
    id: ItemId,
    name: String,
    slots: Vec<ItemId>,
    width: usize,
    hight: usize,
}

impl Location for SeedTray {
    fn location_of(&self,find:ItemId) -> Option<String> {
        for y in 0..self.hight {
            for x in 0..self.width {
                if self.slots[y*self.width + x] == find {
                    return Some(format!("Cell {}:{} of {}", x, y, self.name));
                }
            }
        }
        None
    }
}

impl SeedTray {
    pub fn new<const X:usize, const Y:usize>(name: impl ToString) -> SeedTray {
        SeedTray {
            id: ItemId(Uuid::nil()),
            name: name.to_string(),
            slots: Vec::with_capacity(X*Y),
            width: X,
            hight: Y,
        }
    }
}

#[cfg(feature = "yew")]
mod yew {
    use crate::*;
    use yew::*;
    impl YewObj for super::Plant {
        fn view(&self, ctx: &yew::Context<ObjView>) -> yew::Html {
            let (cbr, _) = ctx.link().context::<crate::components::CallbackReg>(Callback::noop()).unwrap();
            let objs = cbr.read_items();
            let id = self.id;
            html! {
                <div id={id.to_string()}>
                <div class="tooltip"><strong>{self.plant_type}</strong>
                <span class="tooltiptext">{self.id.to_string()}</span>
                </div><br/>
                <strong>{"Stage:"}{self.stage}</strong>
                if let Some(v) = objs.get(&self.location) {
                    {v.yew_view(ctx)}
                } else {
                    {{ctx.link().send_message(ObjMsg::Get(self.location)); "Location not loaded"}}
                }
                </div>
            }
        }
    }
}

#[cfg(test)]
mod test {
    impl crate::plants::Plant {
        pub fn test(id: u8) -> Self {
            use crate::plants::*;
            match id {
                0 => Plant {
                    id: ItemId::from_u128(1000),
                    stage: Stage::Planted,
                    location: ItemId::from_u128(1001),
                    plant_type: PlantTypes::Lettuce,
                },
                _ => Plant {
                    id: ItemId::from_u128(1010),
                    stage: Stage::Planted,
                    location: ItemId::from_u128(1011),
                    plant_type: PlantTypes::Lettuce,
                },
            }
            
        }
    }
}