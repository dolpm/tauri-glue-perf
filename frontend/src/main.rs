use std::sync::atomic::{AtomicUsize, Ordering};

use crate::web_sys::MouseEvent;
use leptos::*;
use rand::prelude::*;
use tauri_glue::*;

static ADJECTIVES: &[&str] = &[
    "pretty",
    "large",
    "big",
    "small",
    "tall",
    "short",
    "long",
    "handsome",
    "plain",
    "quaint",
    "clean",
    "elegant",
    "easy",
    "angry",
    "crazy",
    "helpful",
    "mushy",
    "odd",
    "unsightly",
    "adorable",
    "important",
    "inexpensive",
    "cheap",
    "expensive",
    "fancy",
];

static COLOURS: &[&str] = &[
    "red", "yellow", "blue", "green", "pink", "brown", "purple", "brown", "white", "black",
    "orange",
];

static NOUNS: &[&str] = &[
    "table", "chair", "house", "bbq", "desk", "car", "pony", "cookie", "sandwich", "burger",
    "pizza", "mouse", "keyboard",
];

#[derive(Copy, Debug, Clone, PartialEq, Eq, Hash)]
struct RowData {
    id: usize,
    label: (ReadSignal<String>, WriteSignal<String>),
}

static ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

#[tauri_glue::bind_command(name = create_row)]
pub async fn create_row(iid: usize, rid: usize, label: String) -> Result<(), ()>;

async fn build_data(cx: Scope, count: usize) -> Vec<RowData> {
    let mut thread_rng = thread_rng();

    let mut data = Vec::new();
    data.reserve_exact(count);

    for i in 0..count {
        let adjective = ADJECTIVES.choose(&mut thread_rng).unwrap();
        let colour = COLOURS.choose(&mut thread_rng).unwrap();
        let noun = NOUNS.choose(&mut thread_rng).unwrap();
        let capacity = adjective.len() + colour.len() + noun.len() + 2;
        let mut label = String::with_capacity(capacity);
        label.push_str(adjective);
        label.push(' ');
        label.push_str(colour);
        label.push(' ');
        label.push_str(noun);

        let r_data = RowData {
            id: ID_COUNTER.load(Ordering::Relaxed),
            label: create_signal(cx, label),
        };

        create_row(i, r_data.id, r_data.label.0())
            .await
            .expect("oops");

        data.push(r_data);

        ID_COUNTER.store(ID_COUNTER.load(Ordering::Relaxed) + 1, Ordering::Relaxed);
    }

    data
}

#[component]
fn Button(cx: Scope, id: String, text: String) -> Element {
    view! {
        cx,
        <div class="col-sm-6 smallpad">
            <button id=id class="btn btn-primary btn-block" type="button">{text}</button>
        </div>
    }
}

#[component]
fn App(cx: Scope) -> Element {
    let (data, set_data) = create_signal(cx, Vec::<RowData>::new());
    let (selected, set_selected) = create_signal(cx, None::<usize>);
    let (op_time_avg, set_op_time_avg) = create_signal(cx, 0.0);

    let remove = move |id| {
        set_data.update(move |data| data.retain(|row| row.id != id));
    };

    let run = move |_| {
        spawn_local(async move {
            let start = js_sys::Date::now();
            set_data(build_data(cx, 1000).await);
            set_op_time_avg(js_sys::Date::now() - start);
        });
        set_selected(None);
    };

    let run_lots = move |_| {
        spawn_local(async move {
            let start = js_sys::Date::now();
            set_data(build_data(cx, 10000).await);
            set_op_time_avg((js_sys::Date::now() - start) / 10.0);
        });
        set_selected(None);
    };

    let add = move |_| {
        spawn_local(async move {
            let start = js_sys::Date::now();
            let mut rows = build_data(cx, 1000).await;
            set_data.update(move |data| {
                data.append(&mut rows);
            });
            set_op_time_avg(js_sys::Date::now() - start);
        });
    };

    let update = move |_| {
        data.with(|data| {
            for row in data.iter().step_by(10) {
                row.label.1.update(|n| n.push_str(" !!!"));
            }
        });
    };

    let clear = move |_| {
        set_data(Vec::new());
        set_selected(None);
        set_op_time_avg(0.0);
    };

    let swap_rows = move |_| {
        set_data.update(|data| {
            if data.len() > 998 {
                data.swap(1, 998);
            }
        });
    };

    let is_selected = create_selector(cx, selected);

    view! {
        cx,
        <div class="container">
            <div class="jumbotron"><div class="row">
            <div class="col-md-6"><h1>"Leptos"</h1><h2>"Avg. Operation Time: "{op_time_avg}"µs"</h2></div>
            <div class="col-md-6"><div class="row">
                <Button id="run".to_string() text="Create 1,000 rows".to_string() on:click=run />
                <Button id="runlots".to_string() text="Create 10,000 rows".to_string() on:click=run_lots />
                <Button id="add".to_string() text="Append 1,000 rows".to_string() on:click=add />
                <Button id="update".to_string() text="Update every 10th row".to_string() on:click=update />
                <Button id="clear".to_string() text="Clear".to_string() on:click=clear />
                <Button id="swaprows".to_string() text="Swap Rows".to_string() on:click=swap_rows />
            </div></div>
            </div></div>
            <table class="table table-hover table-striped test-data">
                <tbody>
                    <For each=data key=|row| row.id>{{
                        let is_selected = is_selected.clone();
                        move |cx, row: &RowData| {
                            let row_id = row.id;
                            let (label, _) = row.label;
                            let is_selected = is_selected.clone();
                            view! {
                                cx,
                                <tr class:danger={move || is_selected(Some(row_id))}>
                                    <td class="col-md-1">{row_id.to_string()}</td>
                                    <td class="col-md-4"><a on:click=move |_| set_selected(Some(row_id))>{move || label.get()}</a></td>
                                    <td class="col-md-1"><a on:click=move |_| remove(row_id)><span class="glyphicon glyphicon-remove" aria-hidden="true"></span></a></td>
                                    <td class="col-md-6"/>
                                </tr>
                            }
                        }
                    }}</For>
                </tbody>
            </table>
            <span class="preloadicon glyphicon glyphicon-remove" aria-hidden="true" />
        </div>
    }
}

pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|cx| {
        view! { cx,
            <App />
        }
    })
}
