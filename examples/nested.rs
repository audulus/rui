use rui::*;

fn my_rectangle() -> impl View {
    rectangle()
        .corner_radius(5.0)
        .color(AZURE_HIGHLIGHT)
        .padding(Auto)
}

fn main() {
    rui(hstack! {
        my_rectangle();
        vstack! {
            my_rectangle();
            hstack! {
                my_rectangle();
                vstack! {
                    my_rectangle();
                    hstack! {
                        my_rectangle();
                        vstack! {
                            my_rectangle();
                            my_rectangle()
                        }
                    }
                }
            }
        }
    });
}
