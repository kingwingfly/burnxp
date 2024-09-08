#[derive(Default, PartialEq, Clone, Copy)]
pub(crate) enum CurrentScreen {
    #[default]
    Main,
    Finished,
    Exiting,
}
