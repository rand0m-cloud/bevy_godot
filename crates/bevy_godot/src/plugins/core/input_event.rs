use crate::prelude::{godot_prelude::InputEvent, *};

pub struct GodotInputEventPlugin;

impl Plugin for GodotInputEventPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            CoreStage::First,
            write_godot_input_events.before(Events::<CollisionEvent>::update_system),
        )
        .add_event::<Ref<InputEvent>>();
    }
}

#[doc(hidden)]
// Can't use ErasedGodotRef code here in place of Ref<InputEvent> because InputEvent isn't manually-managed,
// required by ErasedGodotRef.get().
pub struct InputEventReader(pub std::sync::mpsc::Receiver<Ref<InputEvent>>);

fn write_godot_input_events(
    events: NonSendMut<InputEventReader>,
    mut event_writer: EventWriter<Ref<InputEvent>>,
) {
    event_writer.send_batch(events.0.try_iter());
}
