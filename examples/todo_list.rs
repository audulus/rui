use rui::*;

fn add_button(todos: impl Binding<Vec<String>>) -> impl View {
    state(String::new, move |name, _| {
        hstack((
            text_editor(name),
            button(text("Add Item"), move |cx| {
                let name_str = cx[name].clone();
                todos.with_mut(cx, |todos| todos.push(name_str));
                // Gotta fix a bug in text_editor!
                // cx[name] = String::new();
            }),
        ))
    })
}

fn todo_list(todos: impl Binding<Vec<String>>) -> impl View {
    get_cx(move |cx| {
        let len = todos.with(cx, |todos| todos.len());
        let ids = (0usize..len).collect();

        list(ids, move |id| {
            let id = *id;
            get_cx(move |cx| todos.with(cx, |todos| todos[id].clone()))
        })
    })
}

fn main() {
    rui(state(std::vec::Vec::new, move |todos, _| {
        vstack((add_button(todos), todo_list(todos)))
    }));
}
