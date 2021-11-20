use sauron::Node;
use sauron::node;
use sauron::html::text;
use crate::api::RecordField;
use crate::Msg;
use serde_json::json;
use sauron::wasm_bindgen::JsCast;
use gloo_timers::callback::Timeout;

pub(crate) fn field_view(field: RecordField, rec_id: &str, field_idx: usize) -> Node<Msg> {
    let ftype = field.ftype.unwrap_or("none".to_string());
    node! [
        <span>
{  text(",\n   ") } { text(format!("field_{}: ", field_idx)) }

        {
            match &field.reference {
                Some(ref_id) => reference_view(ref_id, rec_id, field_idx),
                None => match ftype.as_str() {
                        "image" => image_view(field.value, rec_id, field_idx),
                        "color" => color_view(field.value, rec_id, field_idx),
                        "string" => string_view(field.value, rec_id, field_idx),
                        "int" => int_view(field.value, rec_id, field_idx),
                        "boolean" => boolean_view(field.value, rec_id, field_idx),
                        "pie" => pie_view(field.value, rec_id, field_idx),
                        "float" => float_view(field.value, rec_id, field_idx),
                        _ => unknown_view(field.value, rec_id, field_idx)
                    }
            }
        }

        </span>
    ]
}

fn color_view(value: serde_json::Value, rec_id: &str, field_idx: usize) -> Node<Msg> {
    let rec_id = rec_id.to_string();
    let value = value.as_str().unwrap().to_string();
    node! [ <span>
        <input
            type = { "color" }
            value = { value }
            on_change=move |e| { Msg::PatchRequested { id: rec_id.clone(), fi: field_idx, new_value: json!{ e.value } } }
        />
    </span> ]
}

fn event2ctx(e: &sauron::web_sys::MouseEvent) -> sauron::web_sys::CanvasRenderingContext2d {
    let canvas = e.target().unwrap().dyn_into::<sauron::web_sys::HtmlCanvasElement>().unwrap();
    canvas.get_context("2d").expect("no 2d ctx").expect("no 2d ctx 2").dyn_into::<sauron::web_sys::CanvasRenderingContext2d>().expect("no 2d ctx 3")
}

fn image_view(value: serde_json::Value, rec_id: &str, field_idx: usize) -> Node<Msg> {
    let url = value["data_url"].as_str().unwrap().to_string();
    let width = value["width"].as_u64().unwrap();
    let height = value["height"].as_u64().unwrap();
    let len = rec_id.len();
    let rec_id = rec_id.to_string();

    let img = sauron::html::img(
        vec![
            sauron::html::attributes::src(url),
            sauron::html::attributes::style("display", "none"),
            sauron::events::on("load", move |e| {
                let img = e.as_web().unwrap().target().unwrap().dyn_into::<sauron::web_sys::HtmlImageElement>().unwrap();
                let canvas = img.next_sibling().unwrap().dyn_into::<sauron::web_sys::HtmlCanvasElement>().unwrap();
                let ctx = canvas.get_context("2d").expect("no 2d ctx").expect("no 2d ctx 2").dyn_into::<sauron::web_sys::CanvasRenderingContext2d>().expect("no 2d ctx 3");
                Timeout::new(len as u32, move || {
                    ctx.draw_image_with_html_image_element(&img, 0.0, 0.0).unwrap();
                }).forget();
                Msg::Noop
            })
        ], vec![]
    );

    node! [ <span>
        { img }
        <canvas
            width={ width }
            height={ height }
            on_contextmenu = move |e| {
                e.prevent_default();
                Msg::Noop
            }
            on_mousemove = move |e| {
                let ctx = event2ctx(&e);
                if e.buttons() != 0 {
                    let x = e.offset_x();
                    let y = e.offset_y();
                    if e.buttons() == 1 {
                        ctx.set_line_width(2.0);
                        ctx.set_stroke_style(&sauron::prelude::JsValue::from_str("#000000"));
                    } else {
                        ctx.set_line_width(4.0);
                        ctx.set_stroke_style(&sauron::prelude::JsValue::from_str("#FFFFFF"));
                    }
                    ctx.line_to(x.into(), y.into());
                    ctx.stroke();
                    //ctx.stroke_rect(x.into(), y.into(), 1.0, 1.0);
                }
                Msg::Noop
            }
            on_mousedown = move |e| {
                let ctx = event2ctx(&e);
                let x = e.offset_x();
                let y = e.offset_y();
                ctx.begin_path();
                ctx.move_to(x.into(), y.into());
                e.prevent_default();
                Msg::Noop
            }
            on_mouseup = move |e| {
                e.prevent_default();
                let ctx = event2ctx(&e);
                let url = ctx.canvas().unwrap().to_data_url().unwrap();
                Msg::PatchRequested {id: rec_id.clone(), fi: field_idx, new_value: json!({ "width": width, "height": height, "data_url": url })}
            }
        >
        </canvas>
    </span> ]
}

fn int_view(value: serde_json::Value, _rec_id: &str, _field_idx: usize) -> Node<Msg> {
    node! [ <span>{text(value.to_string())}</span> ]
}

fn pie_view(value: serde_json::Value, _rec_id: &str, _field_idx: usize) -> Node<Msg> {

    node! [
        <div style={"display: inline-block"}>
            {
                for (key, value) in value.as_object().unwrap() {
                    node! [ <span> { color_label(key) } { text(":") } { percentage(value.as_f64().unwrap()) }  <br/></span> ]
                }
            }
        </div>
    ]
}

fn string_view(value: serde_json::Value, rec_id: &str, field_idx: usize) -> Node<Msg> {
    let rec_id = rec_id.to_string();
    let value = value.as_str().unwrap().to_string();
    node! [ <span>
        <input
            type = { "text" }
            value = { value }
            on_input=move |e| { Msg::PatchRequested { id: rec_id.clone(), fi: field_idx, new_value: json!{ e.value } } }
        />
    </span> ]
}

fn boolean_view(value: serde_json::Value, rec_id: &str, field_idx: usize) -> Node<Msg> {
    let rec_id = rec_id.to_string();
    let value = value.as_bool().unwrap();
    node! [ <span>
        <input
            type = { "checkbox" }
            checked = { value }
            on_checked=move |e| { Msg::PatchRequested { id: rec_id.clone(), fi: field_idx, new_value: json!{ e } } }
        />
    </span> ]
}

fn reference_view(ref_id: &str, _rec_id: &str, _field_idx: usize) -> Node<Msg> {
    node! [
        <span>
            <a href = { format!("#{}", ref_id) }>{text(ref_id)}</a>
        </span>
    ]
}

fn unknown_view(value: serde_json::Value, _rec_id: &str, _field_idx: usize) -> Node<Msg> {
    node! [ <span>{text(value.to_string())}</span> ]
}

pub(crate) fn color_label(color: &str) -> Node<Msg> {
    node![
        <span>
            { text(color) }
            <span style={format!("color: {};", color)}>{ text("◼") }</span>
        </span> ]
}

pub(crate) fn percentage(value: f64) -> Node<Msg> {
    node![
        <span title={format!("{}", value)}> { text(format!("{:.2} %", value*100.0)) } </span>
    ]
}

fn float_view(value: serde_json::Value, _rec_id: &str, _field_idx: usize) -> Node<Msg> {
    percentage(value.as_f64().unwrap())
}