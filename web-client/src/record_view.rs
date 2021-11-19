use sauron::Node;
use crate::api::Record;
use crate::Msg;
use sauron::node;
use sauron::html::text;
use crate::field_view::field_view;
use crate::field_view::color_label;

pub(crate) fn record_view(record: Record) -> Node<Msg> {

    node! [
        <div class="record" id={ format!("{}", record.id) }>
            <pre>
{ text("{\n") }
{ text(format!("   id: \"{}\",\n", record.id)) }
{ text("   column: ") } { color_label(&record.column) }
        {
            for idx in 0..record.fields.len() {
                field_view(record.fields[idx].clone(), &record.id, idx)
            }
        }

{ text("\n") } { text("}") }
            </pre>


        </div>

    ]
}