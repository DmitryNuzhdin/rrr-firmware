use yew::prelude::*;
use reqwasm::http::Request;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen::JsValue;

#[function_component]
fn App() -> Html {
    let onclick = Callback::from(move |_| {
        spawn_local(async move {
            Request::get("http://rrr.local/blue")
                .send()
                .await
                .unwrap();
        });

        web_sys::console::log_1(&JsValue::from("Blue clicked"));
    });

    html! {
        <div>
            <button {onclick}>{"Blue"}</button>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}