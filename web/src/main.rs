use leptos::prelude::*;
use std::str::FromStr;

fn main() {
    leptos::mount::mount_to_body(App);
}

fn initial_purl() -> String {
    web_sys::window()
        .and_then(|w| w.location().search().ok())
        .and_then(|s| web_sys::UrlSearchParams::new_with_str(&s).ok())
        .and_then(|p| p.get("purl"))
        .unwrap_or_default()
}

fn build_share_url(purl: &str) -> Option<String> {
    let window = web_sys::window()?;
    let location = window.location();
    let origin = location.origin().ok()?;
    let pathname = location.pathname().ok()?;
    let encoded = js_sys::encode_uri_component(purl);
    Some(format!("{origin}{pathname}?purl={encoded}"))
}

#[component]
fn App() -> impl IntoView {
    let (input, set_input) = signal(initial_purl());
    let (copied, set_copied) = signal(false);

    let result = move || {
        let value = input.get();
        if value.is_empty() {
            return None;
        }
        Some(packageurl::PackageUrl::from_str(&value))
    };

    Effect::new(move |_| {
        let purl = input.get();
        if let Some(window) = web_sys::window() {
            let pathname = window.location().pathname().unwrap_or_default();
            let url = if purl.is_empty() {
                pathname
            } else {
                let encoded = js_sys::encode_uri_component(&purl);
                format!("{pathname}?purl={encoded}")
            };
            let _ = window.history().and_then(|h| {
                h.replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&url))
            });
        }
    });

    let copy_link = move |_| {
        let purl = input.get();
        if purl.is_empty() {
            return;
        }
        if let Some(url) = build_share_url(&purl) {
            let _ = web_sys::window().map(|w| w.navigator().clipboard().write_text(&url));
            set_copied.set(true);
            set_timeout(
                move || set_copied.set(false),
                std::time::Duration::from_secs(2),
            );
        }
    };

    view! {
        <div class="container">
            <header>
                <h1><span class="badge">"pkg:"</span>" PackageURL Validator"</h1>
                <p class="subtitle">
                    "Paste a "
                    <a href="https://github.com/package-url/purl-spec" target="_blank">"Package URL"</a>
                    " to validate and inspect its components."
                </p>
            </header>

            <div class="input-row">
                <input
                    type="text"
                    placeholder="pkg:type/namespace/name@version?key=value#subpath"
                    class="purl-input"
                    prop:value=move || input.get()
                    on:input=move |ev| {
                        set_input.set(event_target_value(&ev));
                    }
                />
                <button
                    class="share-btn"
                    class:copied=move || copied.get()
                    title="Copy shareable link"
                    on:click=copy_link
                    disabled=move || input.get().is_empty()
                >
                    {move || if copied.get() { "Copied!" } else { "Share" }}
                </button>
            </div>

            <div class="result">
                {move || match result() {
                    None => view! { <p class="hint">"Enter a purl above to validate it."</p> }.into_any(),
                    Some(Ok(purl)) => view! { <ParsedFields purl=purl /> }.into_any(),
                    Some(Err(err)) => view! {
                        <div class="error">
                            <span class="error-label">"Error: "</span>
                            {err.to_string()}
                        </div>
                    }.into_any(),
                }}
            </div>

            <footer>
                <a href="https://github.com/scm-rs/packageurl.rs" target="_blank">"github.com/scm-rs/packageurl.rs"</a>
            </footer>
        </div>
    }
}

#[component]
fn ParsedFields(purl: packageurl::PackageUrl<'static>) -> impl IntoView {
    let canonical = purl.to_string();
    let ty = purl.ty().to_string();
    let namespace = purl.namespace().map(|s| s.to_string());
    let name = purl.name().to_string();
    let version = purl.version().map(|s| s.to_string());
    let subpath = purl.subpath().map(|s| s.to_string());
    let qualifiers: Vec<String> = purl
        .qualifiers()
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect();

    let namespace_view = match namespace {
        Some(ns) => view! { <code>{ns}</code> }.into_any(),
        None => view! { <span class="none">"(none)"</span> }.into_any(),
    };
    let version_view = match version {
        Some(v) => view! { <code>{v}</code> }.into_any(),
        None => view! { <span class="none">"(none)"</span> }.into_any(),
    };
    let subpath_view = match subpath {
        Some(sp) => view! { <code>{sp}</code> }.into_any(),
        None => view! { <span class="none">"(none)"</span> }.into_any(),
    };
    let qualifiers_view = if qualifiers.is_empty() {
        view! { <span class="none">"(none)"</span> }.into_any()
    } else {
        view! {
            <ul class="qualifiers-list">
                {qualifiers.into_iter().map(|q| view! {
                    <li><code>{q}</code></li>
                }).collect::<Vec<_>>()}
            </ul>
        }
        .into_any()
    };

    view! {
        <div class="success">
            <div class="canonical">
                <span class="label">"Canonical"</span>
                <code>{canonical}</code>
            </div>
            <table class="fields">
                <tbody>
                    <tr>
                        <th>"Type"</th>
                        <td><code>{ty}</code></td>
                    </tr>
                    <tr>
                        <th>"Namespace"</th>
                        <td>{namespace_view}</td>
                    </tr>
                    <tr>
                        <th>"Name"</th>
                        <td><code>{name}</code></td>
                    </tr>
                    <tr>
                        <th>"Version"</th>
                        <td>{version_view}</td>
                    </tr>
                    <tr>
                        <th>"Qualifiers"</th>
                        <td>{qualifiers_view}</td>
                    </tr>
                    <tr>
                        <th>"Subpath"</th>
                        <td>{subpath_view}</td>
                    </tr>
                </tbody>
            </table>
        </div>
    }
}
