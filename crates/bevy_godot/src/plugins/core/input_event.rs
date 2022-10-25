use crate::prelude::{
    godot_prelude::{InputEvent as GodotInputEvent, SubClass},
    *,
};

pub struct GodotInputEventPlugin;

impl Plugin for GodotInputEventPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            CoreStage::First,
            write_input_events
                .before(Events::<InputEvent>::update_system)
                .before(Events::<UnhandledInputEvent>::update_system),
        )
        .add_event::<InputEvent>()
        .add_event::<UnhandledInputEvent>();
    }
}

/// An input event from the `_input` callback
#[derive(Debug)]
pub struct InputEvent(Ref<GodotInputEvent>);

impl InputEvent {
    pub fn get<T: SubClass<GodotInputEvent>>(&self) -> TRef<T> {
        self.try_get().unwrap()
    }

    pub fn try_get<T: SubClass<GodotInputEvent>>(&self) -> Option<TRef<T>> {
        unsafe { self.0.assume_safe().cast() }
    }
}

/// An input event from the `_unhandled_input` callback
#[derive(Debug)]
pub struct UnhandledInputEvent(Ref<GodotInputEvent>);

fn write_input_events(
    events: NonSendMut<InputEventReader>,
    mut unhandled_evts: EventWriter<UnhandledInputEvent>,
    mut normal_evts: EventWriter<InputEvent>,
) {
    let (normal, unhandled) = events
        .0
        .try_iter()
        .partition::<Vec<_>, _>(|(ty, _)| *ty == InputEventType::Normal);

    normal_evts.send_batch(normal.into_iter().map(|(_, evt)| InputEvent(evt)));
    unhandled_evts.send_batch(
        unhandled
            .into_iter()
            .map(|(_, evt)| UnhandledInputEvent(evt)),
    );
}

impl UnhandledInputEvent {
    pub fn get<T: SubClass<GodotInputEvent>>(&self) -> TRef<T> {
        self.try_get().unwrap()
    }

    pub fn try_get<T: SubClass<GodotInputEvent>>(&self) -> Option<TRef<T>> {
        unsafe { self.0.assume_safe().cast() }
    }
}

#[doc(hidden)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum InputEventType {
    Normal,
    Unhandled,
}

#[doc(hidden)]
pub struct InputEventReader(pub std::sync::mpsc::Receiver<(InputEventType, Ref<GodotInputEvent>)>);
