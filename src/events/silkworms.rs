use crate::Date;
use bevy_reflect::prelude::*;
use serde::{Serialize, Deserialize};
#[derive(Debug, PartialEq, Reflect, Serialize, Deserialize, Clone)]
#[reflect(Serialize, Deserialize)]
pub enum SilkWormEvents {
    Hatched(Date),
}

#[cfg(feature = "yew")]
pub mod yew {

    use yew::*;
    use crate::events::Item;

    use super::SilkWormEvents;
    pub struct SilkWormEvent;
    #[derive(Debug, Properties, PartialEq)]
    pub struct SilkWormProp{
        event: SilkWormEvents,
    }
    impl Component for SilkWormEvent {
        type Message = ();
        type Properties = SilkWormProp;

        fn create(_ctx: &Context<Self>) -> Self {
            SilkWormEvent
        }

        fn view(&self, ctx: &Context<Self>) -> Html {
            match ctx.props().event {
                SilkWormEvents::Hatched(date) => html! {
                    <div>
                        <p>{"SilkWorms: Hatched on "}{date}</p>
                    </div>
                },
            }
        }
    }

    pub trait Event {
        fn view(&self) -> Html;
        fn type_id(&self) -> std::any::TypeId;
    }

    impl PartialEq for dyn Event {
        fn eq(&self, other: &Self) -> bool {
            self.type_id() == other.type_id()
        }
    }

    #[derive(Properties)]
    pub struct EventProps {
        pub event: std::rc::Rc<dyn Event>
    }

    impl PartialEq for EventProps {
        fn eq(&self, other: &Self) -> bool {
            self.event.type_id() == other.event.type_id()
        }
    }

    use yew::function_component;
    #[function_component(EventView)]
    pub fn event_view(props: &EventProps) -> Html {
        props.event.view()
    }

    impl crate::YewObj for SilkWormEvents {
        fn view(&self, _ctx: &Context<crate::ObjList>) -> Html {
            match self {
                SilkWormEvents::Hatched(date) => html! {
                    <p>{"Silkworms Hatched "}{date}</p>
                }
            }
        }
    }

    impl Item for SilkWormEvents {
        fn id(&self) -> crate::ItemId {
            crate::ItemId(uuid::Uuid::nil())
        }
    }
}