use rui::*;

fn add_button(todos: impl Binding<Vec<String>>) -> impl View {
    state(
        || String::new(),
        move |name, _| {
            hstack((
                text_editor(name),
                button(text("Add Item"), move |cx| {
                    let name_str = cx[name].clone();
                    todos.with_mut(cx, |todos| todos.push(name_str));
                    // Gotta fix a bug in text_editor!
                    // cx[name] = String::new();
                }),
            ))
        },
    )
}

fn todo_list(todos: impl Binding<Vec<String>>) -> impl View {
    state(
        || (),
        move |_, cx| {
            let len = todos.with(cx, |todos| todos.len());
            let ids = (0usize..len).collect();

            list(ids, move |id| {
                let id = *id;
                state(
                    || (),
                    move |_, cx| todos.with(cx, |todos| todos[id].clone()),
                )
            })
        },
    )
}

fn main() {
    rui(state(
        || vec![],
        move |todos: State<Vec<String>>, _| vstack((add_button(todos), todo_list(todos))),
    ));
}
