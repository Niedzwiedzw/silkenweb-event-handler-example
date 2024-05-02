use event_handler::EventHandler;
use futures_signals::signal::{Mutable, SignalExt};
use silkenweb::{dom::Dom, elements::html::*, prelude::*, value::Sig};

pub mod event_handler {
    use std::{cell::RefCell, rc::Rc};

    impl<F: FnMut(T) + 'static, T: 'static> From<F> for EventHandler<T> {
        fn from(callback: F) -> Self {
            EventHandler::new(callback)
        }
    }

    pub struct EventHandler<T> {
        callback: Rc<RefCell<dyn FnMut(T)>>,
    }

    impl<T> Clone for EventHandler<T> {
        fn clone(&self) -> Self {
            Self {
                callback: self.callback.clone(),
            }
        }
    }

    impl<T> PartialEq for EventHandler<T> {
        fn eq(&self, other: &Self) -> bool {
            Rc::ptr_eq(&self.callback, &other.callback)
        }
    }

    impl<T> EventHandler<T>
    where
        T: 'static,
    {
        pub fn new<F>(callback: F) -> Self
        where
            F: FnMut(T) + 'static,
        {
            Self {
                callback: Rc::new(RefCell::new(callback)),
            }
        }

        pub fn call(&self, value: T) {
            self.callback
                .try_borrow_mut()
                .map(|mut callback| callback(value))
                .expect("event handlers to be always callable");
        }

        pub fn filter_map<U, F>(self, map: F) -> EventHandler<U>
        where
            U: 'static,
            F: (Fn(U) -> Option<T>) + 'static,
        {
            (move |v| {
                if let Some(v) = map(v) {
                    self.call(v)
                }
            })
            .into()
        }

        pub fn map_some<U, F>(self, map: F) -> EventHandler<Option<U>>
        where
            U: 'static,
            F: (Fn(U) -> T) + 'static,
        {
            (move |v: Option<U>| {
                if let Some(v) = v {
                    self.call(map(v))
                }
            })
            .into()
        }

        pub fn map<U, F>(self, mut map: F) -> EventHandler<U>
        where
            U: 'static,
            F: FnMut(U) -> T + 'static,
        {
            (move |value| self.call(map(value))).into()
        }
    }
}

enum CounterEvent {
    Increase,
    Decrease,
}

fn counter<D: Dom>(
    handler: EventHandler<CounterEvent>,
    value: impl Signal<Item = i32> + 'static,
) -> Div<D> {
    div()
        .child(
            div()
                .class("value")
                .text(Sig(value.map(move |v| v.to_string()))),
        )
        .child(button().text("increase").on_click({
            let handler = handler.clone();
            move |_, _| handler.call(CounterEvent::Increase)
        }))
        .child(
            button()
                .text("decrease")
                .on_click(move |_, _| handler.call(CounterEvent::Decrease)),
        )
}

enum AppEvent {
    Counter(CounterEvent),
}

fn main() {
    let count = Mutable::new(0);
    let handler = {
        let count = count.clone();
        EventHandler::new(move |event: AppEvent| {
            match event {
                AppEvent::Counter(counter) => match counter {
                    CounterEvent::Increase => count.replace_with(|previous| *previous + 1),
                    CounterEvent::Decrease => count.replace_with(|previous| *previous - 1),
                },
            };
        })
    };

    let app = div().child(counter(handler.map(AppEvent::Counter), count.signal()));

    mount("app", app);
}
