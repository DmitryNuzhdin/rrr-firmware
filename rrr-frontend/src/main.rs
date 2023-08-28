use gloo::console::log;
use yew::prelude::*;
use yew_hooks::prelude::*;
use reqwasm::http::Request;
use wasm_bindgen_futures::spawn_local;
use serde::{Serialize, Deserialize};
use serde::de::DeserializeOwned;
use material_yew::MatButton;

use gloo::timers::callback::{Timeout};

#[derive(Properties, PartialEq)]
struct RestButtonProps {
    text: String,
    command: Command,
}

#[function_component]
fn RestButton(props: &RestButtonProps) -> Html {
    let text = props.text.clone();
    let commmand: Command = props.command.clone();

    let onclick: Callback<MouseEvent, ()> = Callback::from(move |_| {
        let s = "http://rrr.local/command".to_owned();
        let s = s.clone();
        let commmand = commmand.clone();
        spawn_local(async move {
            Request::post(&s)
                .body(serde_json::to_string(&commmand).unwrap())
                .send()
                .await
                .unwrap();
        });
        ()
    });

    html! {<span {onclick}><MatButton label={text}/></span>}
}

#[function_component]
fn App() -> Html {
    html! {
        <div>
            <div>
                <RestButton text="BLUE" command={Command::SetLedColor {r: 0, g: 0, b: 20}}/>
                <RestButton text="RED" command={Command::SetLedColor {r: 20, g: 0, b: 0}}/>
                <RestButton text="GREEN" command={Command::SetLedColor {r: 0, g: 20, b: 0}}/>
            </div>
            <div>
                <StateComponent/>
            </div>
        </div>
    }
}


#[function_component]
fn StateComponent() -> Html {
    let state = use_state_eq(|| State::default());
    let updateRequired = use_state_eq(|| true);

    async fn fetch_state() -> Result<State, Error> {
        fetch::<State>("http://rrr.local/state".to_string()).await
    }

    async fn fetch<T>(url: String) -> Result<T, Error>
        where
            T: DeserializeOwned,
    {
        let response = Request::get(&url).send().await;
        if let Ok(data) = response {
            (data.json::<T>().await).map_or(Err(Error::DeserializeError), |repo| Ok(repo))
        } else {
            Err(Error::RequestError)
        }
    }

    let u3 = updateRequired.clone();

    let state2 = state.clone();

    let async_request: UseAsyncHandle<State, Error> = use_async(async move {
        let ans = fetch_state().await;
        let ans2 = ans.clone();
        if ans.is_ok() {state2.set(ans.unwrap())};
        Timeout::new(1000, move || {
            log!("request");
            u3.set(true);
        }).forget();
        ans2
    });

    let u2 = updateRequired.clone();
    if *u2 {
        async_request.run();
        u2.set(false);
    }

    html! {
        <div>
            <div>{format!("Battery charge: {:.2}%", state.battery.soc )}</div>
            <div>{format!("Battery voltage: {:.2}V", state.battery.voltage)}</div>
            <div>{format!("Battery charge rate: {:.2}%/hr", state.battery.charge_rate)}</div>
            <div>{format!("Pyro 1 test voltage: {:.2}V", state.pyro.channel1.test_voltage)}</div>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}


#[derive(Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct State {
    pub battery: BatteryState,
    pub pyro: PyroState,
}

#[derive(Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct BatteryState{
    pub soc: f32,
    pub voltage: f32,
    pub charge_rate: f32,
}

#[derive(PartialEq, Serialize, Deserialize)]
pub struct WifiConnectionConfiguration{
    pub connection_type: WifiConnectionType,
    pub ssid: String,
    pub password: String,
}

#[derive(PartialEq, Serialize, Deserialize)]
pub enum WifiConnectionType{
    ConnectToExternal,
    StartAccessPoint
}

#[derive(Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PyroChannelState{
    pub fire: bool,
    pub test_voltage: f32,
}

#[derive(Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PyroState{
    pub channel1: PyroChannelState,
    pub channel2: PyroChannelState
}


#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum Command{
    Reset,
    SetWifi{ssid: String, password: String},
    SetLedColor{r: u8, g: u8, b: u8},
}

#[derive(Clone, Debug, PartialEq)]
enum Error {
    RequestError,
    DeserializeError,
}