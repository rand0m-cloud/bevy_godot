use crate::prelude::{
    bevy_prelude::{CoreStage, EventWriter, NonSendMut},
    godot_prelude::{Variant, VariantArray},
    *,
};
use bevy::ecs::event::Events;

pub struct GodotSignalsPlugin;

impl Plugin for GodotSignalsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            CoreStage::First,
            write_godot_signal_events.before(Events::<GodotSignal>::update_system),
        )
        .add_event::<GodotSignal>();
    }
}

#[derive(Debug)]
pub struct GodotSignal {
    name: String,
    origin: ErasedGodotRef,
    args: Vec<Variant>,
}

impl GodotSignal {
    #[doc(hidden)]
    pub fn new(name: impl ToString, origin: TRef<Object>, args: Vec<Variant>) -> Self {
        Self {
            name: name.to_string(),
            origin: unsafe { ErasedGodotRef::from_instance_id(origin.get_instance_id()) },
            args,
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn origin(&self) -> ErasedGodotRef {
        self.origin.clone()
    }

    pub fn args(&self) -> &[Variant] {
        &self.args
    }
}

#[doc(hidden)]
pub struct GodotSignalReader(pub std::sync::mpsc::Receiver<GodotSignal>);

fn write_godot_signal_events(
    events: NonSendMut<GodotSignalReader>,
    mut event_writer: EventWriter<GodotSignal>,
) {
    event_writer.send_batch(events.0.try_iter());
}

pub fn connect_godot_signal(
    node: &mut ErasedGodotRef,
    signal_name: &str,
    scene_tree: &mut SceneTreeRef,
) {
    let node = node.get::<Object>();
    let scene_root = unsafe { scene_tree.get().root().unwrap().assume_safe() };

    let signal_watcher = scene_root
        .get_node("/root/Autoload/GodotSignalWatcher")
        .unwrap();

    node.connect(
        signal_name,
        signal_watcher,
        "event",
        VariantArray::from_iter(
            [
                Variant::new(signal_watcher),
                Variant::new(node),
                Variant::new(signal_name),
            ]
            .into_iter(),
        )
        .into_shared(),
        0,
    )
    .unwrap();
}
