use rui::*;

fn main() {
    rui(state(Counters::default, |counters, cx| {
        vstack((
            list(cx[counters].ids(), move |&i| {
                with_cx(move |cx| {
                    let count = bind(counters, CounterLens(i));
                    hstack((
                        format!("{}", count.get(cx)).padding(Auto),
                        button("increment", move |cx| {
                            *count.get_mut(cx) += 1;
                        })
                        .padding(Auto),
                        button("decrement", move |cx| {
                            *count.get_mut(cx) -= 1;
                        })
                        .padding(Auto),
                        button("remove", move |cx| cx[counters].remove_counter(i)).padding(Auto),
                    ))
                })
            }),
            format!("total: {}", cx[counters].sum_counters()).padding(Auto),
            button("add counter", move |cx| cx[counters].add_counter()).padding(Auto),
        ))
    }));
}

#[derive(Default, Debug)]
struct Counters {
    counters: Vec<Option<i32>>,
}

impl Counters {
    fn ids(&self) -> Vec<usize> {
        let mut ids = vec![];
        for i in 0..self.counters.len() {
            if let Some(_) = self.counters[i] {
                ids.push(i);
            }
        }
        ids
    }
    fn add_counter(&mut self) {
        self.counters.push(Some(0));
    }
    fn remove_counter(&mut self, id: usize) {
        self.counters[id] = None;
    }
    fn sum_counters(&self) -> i32 {
        let mut sum = 0;
        for c in &self.counters {
            if let Some(x) = c {
                sum += x
            }
        }
        sum
    }
}

#[derive(Copy, Clone, Debug)]
struct CounterLens(usize);

impl Lens<Counters, i32> for CounterLens {
    fn focus<'a>(&self, data: &'a Counters) -> &'a i32 {
        data.counters[self.0].as_ref().unwrap()
    }

    fn focus_mut<'a>(&self, data: &'a mut Counters) -> &'a mut i32 {
        data.counters[self.0].as_mut().unwrap()
    }
}
