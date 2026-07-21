use leptos::prelude::*;
use std::str::FromStr;

fn main() {
    leptos::mount::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let (input, set_input) = signal(String::new());

    let result = move || {
        let value = input.get();
        if value.is_empty() {
            return None;
        }
        Some(packageurl::PackageUrl::from_str(&value))
    };

    view! {
        <div class="container">
            <header>
                <img src="logo.png" alt="purl logo" class="logo" />
                <h1>"PackageURL Validator"</h1>
                <p class="subtitle">
                    "Paste a "
                    <a href="https://github.com/package-url/purl-spec" target="_blank">"Package URL"</a>
                    " to validate and inspect its components."
                </p>
            </header>

            <input
                type="text"
                placeholder="pkg:type/namespace/name@version?key=value#subpath"
                class="purl-input"
                prop:value=move || input.get()
                on:input=move |ev| {
                    set_input.set(event_target_value(&ev));
                }
            />

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
