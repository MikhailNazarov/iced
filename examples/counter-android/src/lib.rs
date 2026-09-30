use iced::widget::{button, column, text, text_input};
use iced::{Center, Element, Padding, Task};

const VALUE_KEY: &str = "counter-value";

#[derive(Default)]
struct Counter {
    value: i64,
    draft: String,
    clipboard: Option<String>,
    insets: iced::window::Insets,
}

#[derive(Debug, Clone)]
enum Message {
    Increment,
    Decrement,
    Read(Option<String>),
    InsetsChanged(iced::window::Insets),
    Draft(String),
    Submit,
}

impl Counter {
    fn boot() -> Self {
        // The state storage is only available on Android
        #[cfg(target_os = "android")]
        let value = iced::platform::load_persisted_state(VALUE_KEY)
            .or_else(|| iced::platform::load_instance_state(VALUE_KEY))
            .and_then(|bytes| <[u8; 8]>::try_from(bytes.as_slice()).ok())
            .map(i64::from_le_bytes)
            .unwrap_or_default();

        #[cfg(not(target_os = "android"))]
        let value = 0;

        Self {
            value,
            ..Self::default()
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Increment => {
                self.value += 1;

                self.persist();

                iced::clipboard::write(format!("The counter is {}", self.value)).discard()
            }
            Message::Decrement => {
                self.value -= 1;

                self.persist();

                iced::clipboard::read_text()
                    .map(|result| Message::Read(result.ok().map(|text| (*text).clone())))
            }
            Message::Read(text) => {
                self.clipboard = text;

                Task::none()
            }
            Message::InsetsChanged(insets) => {
                self.insets = insets;

                Task::none()
            }
            Message::Draft(draft) => {
                self.draft = draft;

                Task::none()
            }
            Message::Submit => {
                self.value = self.draft.parse().unwrap_or(self.value);
                self.draft.clear();

                self.persist();

                Task::none()
            }
        }
    }

    fn persist(&mut self) {
        // The state storage is only available on Android
        #[cfg(target_os = "android")]
        {
            let bytes = self.value.to_le_bytes().to_vec();

            iced::platform::save_persisted_state(VALUE_KEY, &bytes);
            iced::platform::save_instance_state(VALUE_KEY.to_owned(), bytes);
        }

        #[cfg(not(target_os = "android"))]
        let _ = (VALUE_KEY, self.value);
    }

    fn view(&self) -> Element<'_, Message> {
        column![
            button("Increment (writes to the clipboard)")
                .on_press(Message::Increment),
            text(self.value).size(50),
            button("Decrement (reads the clipboard)")
                .on_press(Message::Decrement),
            text(format!(
                "clipboard: {}",
                self.clipboard.as_deref().unwrap_or("press Decrement...")
            ))
            .size(16),
            text_input("Type a number and press enter...", &self.draft)
                .on_input(Message::Draft)
                .on_submit(Message::Submit)
                .width(300)
                .padding(8)
        ]
        .padding(Padding {
            top: 20.0 + self.insets.top,
            left: 20.0 + self.insets.left,
            right: 20.0 + self.insets.right,
            bottom: 20.0 + self.insets.bottom,
        })
        .align_x(Center)
        .into()
    }
}

pub fn run() -> iced::Result {
    iced::application(Counter::boot, Counter::update, Counter::view)
        .title("Iced Counter")
        .subscription(|_| {
            iced::window::insets_events().map(|(_id, insets)| Message::InsetsChanged(insets))
        })
        .run()
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(android_app: iced::platform::AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );

    iced::platform::set_android_app(android_app);

    let _ = run();
}
