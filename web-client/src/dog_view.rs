use crate::{Msg};
use sauron::Node;
use sauron::node;
use sauron::html::text;
use sauron::html::view_if;
use crate::dog_model::{DogModel, Gender};

pub(crate) fn simplified_dog_view(dog: &DogModel) -> Node<Msg> {
    let gender_emoji = if let Gender::Male = dog.gender {"♂"} else {"♀"};
    node![
        <div class="dog-simplified">
            <img class="mini-avatar" src={&dog.avatar_src}/>
            <span class="gender"> { text(gender_emoji) } </span>
            <span class="dog_name"> { text(&dog.name) } </span>
        </div>
    ]
}

pub(crate) fn dog_view(dog: DogModel) -> Node<Msg> {
    let id = dog.id.clone();
    let id2 = dog.id.clone();
    let gender = if let Gender::Male = dog.gender {"Male"} else {"Female"};
    let gender_emoji = if let Gender::Male = dog.gender {"♂"} else {"♀"};
    node![
        <div class={format!("dog {}", gender)}>
            <img src={&dog.avatar_src}/>
            <input type="text" name="dog_name" title={&dog.name}
                value={&dog.name}
                on_input=move |e| { Msg::DogRenamed {id: id.clone(), new_name: e.value.clone() } }
            />
            <span class="gender">
                { text(gender_emoji) }
            </span>
            <span class="trophys">
                { for _ in 0..dog.count_of_trophys { text("🏆") } }
            </span>
            <span class="country">
                <img src={&dog.country_src}/>
            </span>
            {
                view_if(dog.papa_id.is_some() || dog.mama_id.is_some(), node! {
                    <div class="hierarchy">
                        <button on_click=move |_| { Msg::ToggleParents {id: id2.clone() } }> {text("Toggle parents ")}</button>
                        {view_if(dog.shown_parents, node! { <div>
                            { for parent in &dog.loaded_parents { simplified_dog_view(parent) } }
                        </div> })}
                    </div>
                })

            }
        </div>
    ]
}