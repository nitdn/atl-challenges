use graphing::ui_state::Inputs;
fn main() -> iced::Result {
    iced::application("Inputs", Inputs::update, Inputs::view)
        .theme(|_| iced::Theme::TokyoNight)
        .run_with(|| (Inputs::default(), iced::Task::none()))
}
