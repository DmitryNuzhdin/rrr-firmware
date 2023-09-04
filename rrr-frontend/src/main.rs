use rrr_api::*;

use gloo::console::log;
use yew::prelude::*;
use yew_hooks::prelude::*;
use reqwasm::http::Request;
use wasm_bindgen_futures::spawn_local;
use serde::{Serialize, Deserialize};
use serde::de::DeserializeOwned;
use material_yew::*;

use gloo::timers::callback::{Timeout};
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;

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

static command_uri: &str = "http://rrr.local/command";

fn send_command(command: Command) {
    spawn_local(async move {
        Request::post(command_uri)
            .body(serde_json::to_string(&command).unwrap())
            .send()
            .await
            .unwrap();
    });
}

#[function_component]
fn WifiSettings() -> Html {
    let ssid = use_state(||String::new());
    let password = use_state(||String::new());

    let ssid1 = ssid.clone();
    let password1 = password.clone();
    let onclick = move |_| {
        let cmd = Command::SetWifi {ssid: (*ssid1).clone(), password: (*password1).clone()};
        send_command(cmd);
    };

    // let oninput = |s: String| {log!(s)};

    html! { <div>
                <MatTextField label="ssid" value={(*ssid).clone()} oninput={move |s:String| {ssid.set(s)}}/>
                <MatTextField label="password" value={(*password).clone()} oninput={move |s:String| {password.set(s)}}/>
                <span {onclick}><MatButton label="Set wifi"/></span>
        </div>
    }
}

#[function_component]
fn App() -> Html {
    html! {
        <div>
            <StateComponent/>
            <RestButton text="BLUE" command={Command::SetLedColor {r: 0, g: 0, b: 20}}/>
            <RestButton text="RED" command={Command::SetLedColor {r: 20, g: 0, b: 0}}/>
            <RestButton text="GREEN" command={Command::SetLedColor {r: 0, g: 20, b: 0}}/>
            <WifiSettings/>
        </div>
    }
}


#[function_component]
fn StateComponent() -> Html {
    let state = use_state_eq(|| State::default());
    let update_required = use_state_eq(|| true);

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

    let u3 = update_required.clone();

    let state2 = state.clone();

    let async_request: UseAsyncHandle<State, Error> = use_async(async move {
        let ans = fetch_state().await;
        let ans2 = ans.clone();
        if ans.is_ok() { state2.set(ans.unwrap()) };
        Timeout::new(1000, move || {
            log!("request");
            u3.set(true);
        }).forget();
        ans2
    });

    let u2 = update_required.clone();
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

#[derive(Clone, Debug, PartialEq)]
enum Error {
    RequestError,
    DeserializeError,
}