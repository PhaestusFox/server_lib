use super::*;
use std::collections::HashSet;
use web_sys::HtmlInputElement;
use ::yew::*;
use std::str::FromStr;

impl YewObj for Crate {
    fn view(&self, _ctx: &::yew::Context<ObjView>) -> Html {
        let id = self.id;
        html! {
            <div class="crate" id={id.to_string()}>
                <div class="crop tooltip"><strong>{self.crop}</strong>
                <span class="tooltiptext">{self.id.to_string()}</span>
                </div>
                <strong class="size">{self.size}</strong>
                <strong class="grade">{self.grade}</strong>
            </div>
        }
    }
    fn edit(&self, ctx: &::yew::Context<ObjView>) -> Html {
        let id = ctx.props().id;
        html! {
            <div id={id.to_string()}>
                {"Type: "     } <crate::components::EnumSelect<super::Crop> target={id} field="crop"/><br/>
                {"Size: "     } <crate::components::EnumSelect<super::CrateSize> target={id} field="size"/><br/>
                {"Grade: "    } <crate::components::EnumSelect<super::Grade> target={id} field="grade"/><br/>
            </div>
        }
    }
}
use crate::components::CallbackReg;
pub struct GreenHouse {
    data: CallbackReg,
    edit: Option<ItemId>,
    date: Date,
    date_node: NodeRef,
    items: HashSet<ItemId>,
    _sse: gloo_net::eventsource::futures::EventSource,
}
impl Component for GreenHouse {
    type Message = GreenHouseMsg;
    type Properties = ();
    fn create(ctx: &Context<Self>) -> Self {
        use chrono::Datelike;
        let today = chrono::Utc::now().date();
        let now_date =  Date::new_ymd(today.year() as i16, today.month() as u8, today.day() as u8);
        let id = date_to_id(now_date);
        let cb = ctx.link().callback(|item| GreenHouseMsg::LoadList(item));
        wasm_bindgen_futures::spawn_local(async move {
            let res = gloo_net::http::Request::get(&format!("{}/db_item/{}", CONFIG.server_id, id.0))
            .send().await
            .unwrap();
            if res.status() == 200 {
                if let Ok(list) = ron::from_str::<ItemData>(&res.text().await.unwrap()) {
                    if list.type_name == "alloc::vec::Vec<server_lib::items::ItemId>" {
                        if let Ok(list) = ron::from_str(&list.data) {
                            cb.emit(list)
                        } else {
                            web_sys::console::error_1(&"failed ron for item data".into());
                        }
                    } else {    
                        web_sys::console::error_2(&"Wrong type from server: ".into(), &list.type_name.into());
                    }
                } else {
                    web_sys::console::error_1(&"failed ron to make item".into());
                }
            }
        });

        let mut sse = gloo_net::eventsource::futures::EventSource::new(&format!("greenhouse_events"/* , CONFIG.server_id*/)).unwrap();
        let sub = sse.subscribe("add").unwrap();
        let sub2 = sse.subscribe("remove").unwrap();
        let sub3 = sse.subscribe("update").unwrap();
        let cb = ctx.link().callback(|msg| msg);
        wasm_bindgen_futures::spawn_local( async move {
            use futures::{stream, StreamExt};
            let mut all_streams = stream::select_all([sub, sub2, sub3]);
                while let Some(Ok((event_type, msg))) = all_streams.next().await {
                    let data = msg.data().as_string().unwrap();
                    let mut segs = data.split("\n");
                    let id = if let Some(val) = segs.next() {
                        match ItemId::from_str(val) {
                            Ok(val) => val,
                            Err(e) => {web_sys::console::error_1(&e.to_string().into()); continue;}
                        }
                    } else {
                        web_sys::console::error_1(&"no first seg in event".into());
                        continue;
                    };
                    let date = if let Some(val) = segs.next() {
                        match Date::from_str(val) {
                            Ok(val) => Some(val),
                            Err(e) => {web_sys::console::error_1(&e.to_string().into()); continue;}
                        }
                    } else {
                        None
                    };
                    cb.emit(GreenHouseMsg::ServerEvent(match event_type.as_str() {
                        "add" => ServerSideEvent::AddedItem(id, if let Some(date) = date {date} else {web_sys::console::error_1(&"no date givin in date".into()); continue;}),
                        "remove" => ServerSideEvent::RemovedItem(id, if let Some(date) = date {date} else {web_sys::console::error_1(&"no date givin in date".into()); continue;}),
                        "update" => ServerSideEvent::UpadteItem(id),
                        _ => unreachable!()
                    }));
                    web_sys::console::log_1(&format!("1. {}: {:?}", event_type, msg).into())
                }
                web_sys::console::log_1(&"EventSource Closed".into());
            });

        GreenHouse {
            data: CallbackReg::new(),
            edit: None,
            date: now_date,
            items: Default::default(),
            date_node: NodeRef::default(),
            _sse: sse,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let date_node = self.date_node.clone();
        html! {
            <ContextProvider<CallbackReg> context={self.data.clone()}>
                <h1>{"GreenHouse"}</h1>
                <h4>{"Date open is "}<input type="date" ref={self.date_node.clone()}onchange={ctx.link().callback(move |_| {
                    let date = date_node.cast::<HtmlInputElement>().expect("date to be input");
                    GreenHouseMsg::SetDate(Date::from_str(&date.value()).expect("Date to be html date"))
                })}/></h4>
                //<ObjList display={Rc::new(self.items.iter().cloned().collect())}/>
                {for self.items.iter().map(|item| {let i2 = item.clone(); html!{<div><ObjView id={item.clone()} edit={false}/>
                <button onclick={ctx.link().callback(move |_| GreenHouseMsg::EditCrate(i2))}>{"edit"}</button></div>}})}
                if let Some(id) = self.edit {
                    <div class="edit">
                        <ObjView id={id} edit={true}/>
                        <button onclick={ctx.link().callback(|_| GreenHouseMsg::SaveCrate)}>{"Save"}</button>
                    </div>
                }
                <button onclick={ctx.link().callback(|_| GreenHouseMsg::NewCrate)}>{"New Crate"}</button>
                <button onclick={|_| {
                    wasm_bindgen_futures::spawn_local( async {
                        let _ = gloo_net::http::Request::put(&format!("{}/greenhouse_events/add/{}/{}", CONFIG.server_id, CONFIG.greenhouse_namespace, Date::new_ymd(2022, 10, 11).to_web_string())).send().await;
                    });
                }}>{"add"}</button>
                <button onclick={|_| {
                    wasm_bindgen_futures::spawn_local( async {
                        let _ = gloo_net::http::Request::put(&format!("{}/greenhouse_events/remove/{}/{}", CONFIG.server_id, CONFIG.greenhouse_namespace, Date::new_ymd(2022, 10, 11).to_web_string())).send().await;
                    });
                }}>{"remove"}</button>
            </ContextProvider<CallbackReg>>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        use GreenHouseMsg::*;
        match msg {
            Get(id) => {
                web_sys::console::log_1(&format!("getting {}", id.to_string()).into());
                let cb = ctx.link().callback(|msg| msg);
                let cbr = if let Some((reg, _)) = ctx.link().context::<CallbackReg>(Callback::noop()) {reg} else {
                    web_sys::console::error_1(&"filed to get callbackreg from context".into());
                    return false;
                };
                wasm_bindgen_futures::spawn_local(async move {
                    let res = gloo_net::http::Request::get(&format!("{}/db_item/{}", CONFIG.server_id, id.0))
                    .send().await
                    .unwrap();
                    web_sys::console::log_1(&format!("got {}", id.to_string()).into());
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
                                        cb.emit(GreenHouseMsg::LoadItem(item))},
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
            LoadList(list) => {
                web_sys::console::log_2(&"Loading list: ".into(), &format!("{:?}", list).into());
                self.items.clear();
                for id in list {self.items.insert(id);} true},
            LoadItem(item) => {
                web_sys::console::log_1(&format!("loading {}", item.id().to_string()).into());
                self.data.load(item);
                false
            },
            SetDate(new_date) => {self.date = new_date;
                let id = date_to_id(new_date);
                let cb = ctx.link().callback(|item| GreenHouseMsg::LoadList(item));
                wasm_bindgen_futures::spawn_local(async move {
                    let res = gloo_net::http::Request::get(&format!("{}/db_item/{}", CONFIG.server_id, id.0))
                    .send().await
                    .unwrap();
                    if res.status() == 200 {
                        if let Ok(list) = ron::from_str::<ItemData>(&res.text().await.unwrap()) {
                            if list.type_name == "alloc::vec::Vec<server_lib::items::ItemId>" {
                                if let Ok(list) = ron::from_str(&list.data) {
                                    cb.emit(list)
                                } else {
                                    web_sys::console::error_1(&"failed ron for item data".into());
                                }
                            } else {    
                                web_sys::console::error_2(&"Wrong type from server: ".into(), &list.type_name.into());
                            }
                        } else {
                            web_sys::console::error_1(&"failed ron to make item".into());
                        }
                    } else {
                        cb.emit(Vec::new());
                    }
                });
                false},
            NewCrate => {let id = self.data.load(Box::new(Crate::default())); self.edit = Some(id); true},
            SaveCrate => {
                let id = self.edit.expect("Can only save when have edit");
                self.items.insert(id);
                let item = self.data.get_itemdata(id).expect("type to be registered");
                self.edit = None;
                let event = ServerSideEvent::AddedItem(id, self.date);
                wasm_bindgen_futures::spawn_local(async move {
                    if let Ok(body) = ron::to_string(&item) {
                        if let Err(e) = gloo_net::http::Request::put(&format!("{}/db_item/{}", CONFIG.server_id, id.to_string())).body(body).send().await {
                            web_sys::console::error_1(&e.to_string().into());
                        };
                        if let Ok(body) = ron::to_string(&event) {
                            if let Err(e) = gloo_net::http::Request::put(&format!("{}/greenhouse_event", CONFIG.server_id)).body(body).send().await {
                                web_sys::console::error_1(&e.to_string().into());
                            };
                        }
                    }
                });
                true
            },
            EditCrate(id) => {
                if self.items.remove(&id) {
                    self.edit = Some(id);
                    true
                } else {
                    web_sys::console::error_3(&"Tried to edit a crate that is not loaded for this day".into(),
                    &"This probabley means something did not redraw/update".into(),
                    &"otherwise This is a bug you should NewCrate msg".into());
                    false
                }
            }
            ServerEvent(e) => {
                match e {
                    ServerSideEvent::AddedItem(id, date) => {
                        if date == self.date {
                            if self.items.insert(id) {
                                ctx.link().send_message(GreenHouseMsg::Get(id));
                            }
                            true
                        } else {
                            false
                        }
                    },
                    ServerSideEvent::RemovedItem(id, date) => {
                        if date == self.date {
                            self.items.remove(&id)
                        } else {
                            false
                        }
                    },
                    ServerSideEvent::UpadteItem(id) => {
                        ctx.link().send_message(GreenHouseMsg::Get(id));
                        false
                    }
                }
            }
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let date_val = self.date_node.cast::<HtmlInputElement>().expect("date to be input");
            let val = self.date.to_string().replace("/","-");
            date_val.set_value(&val);
            web_sys::console::log_1(&val.into());
        }
    }
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
}