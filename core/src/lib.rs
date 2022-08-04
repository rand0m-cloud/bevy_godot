#![allow(clippy::type_complexity)]

use bevy::app::*;
pub mod plugins;
pub mod prelude;

pub struct GodotPlugin;

impl Plugin for GodotPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(plugins::DefaultGodotPlugin);
    }
}

#[macro_export]
macro_rules! bevy_godot_init {
    ( $init: ident, $app: ident ) => {
        #[derive(NativeClass, Default)]
        #[inherit(Node)]
        struct Autoload {
            app: Option<App>,
        }

        #[methods]
        impl Autoload {
            fn new(_base: &Node) -> Self {
                Self::default()
            }

            #[export]
            fn _ready(&mut self, base: &Node) {
                __app(self, base)
            }

            #[export]
            fn _process(&mut self, _base: TRef<Node>, _delta: f32) {
                use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};

                if let Some(app) = self.app.as_mut() {
                    if let Err(e) = catch_unwind(AssertUnwindSafe(|| app.update())) {
                        self.app = None;
                        resume_unwind(e);
                    }
                }
            }
        }

        fn __godot_init(init: InitHandle) {
            init.add_class::<Autoload>();
            init.add_class::<SceneTreeWatcher>();
            init.add_class::<CollisionWatcher>();
            $init(&init);
        }

        fn __app(autoload: &mut Autoload, base: &Node) {
            let mut app = App::new();
            app.add_plugin(bevy_godot::prelude::GodotPlugin);

            $app(&mut app);

            {
                let (sender, reciever) = std::sync::mpsc::channel();
                let scene_tree_watcher = SceneTreeWatcher::new_instance();
                scene_tree_watcher
                    .map_mut(|script, base| script.notification_channel = Some(sender))
                    .unwrap();
                scene_tree_watcher.base().set_name("SceneTreeWatcher");

                base.add_child(scene_tree_watcher.into_base().into_shared(), true);

                app.insert_non_send_resource(bevy_godot::prelude::SceneTreeEventReader(reciever));
            }

            {
                let (sender, reciever) = std::sync::mpsc::channel();
                let collision_watcher = CollisionWatcher::new_instance();
                collision_watcher
                    .map_mut(|script, base| script.notification_channel = Some(sender))
                    .unwrap();
                collision_watcher.base().set_name("CollisionWatcher");

                base.add_child(collision_watcher.into_base().into_shared(), true);

                app.insert_non_send_resource(bevy_godot::prelude::CollisionEventReader(reciever));
            }

            autoload.app = Some(app);
        }

        #[derive(NativeClass, Default)]
        #[inherit(Node)]
        struct SceneTreeWatcher {
            notification_channel:
                Option<std::sync::mpsc::Sender<bevy_godot::prelude::SceneTreeEvent>>,
        }

        #[methods]
        impl SceneTreeWatcher {
            fn new(_base: &Node) -> Self {
                Self::default()
            }

            #[export]
            fn scene_tree_event(
                &self,
                _base: TRef<Node>,
                node: Ref<Node>,
                event_type: bevy_godot::prelude::SceneTreeEventType,
            ) {
                self.notification_channel
                    .as_ref()
                    .unwrap()
                    .send(bevy_godot::prelude::SceneTreeEvent {
                        node: unsafe {
                            ErasedGodotRef::from_instance_id(node.assume_safe().get_instance_id())
                        },
                        event_type,
                    })
                    .unwrap();
            }
        }

        #[derive(NativeClass, Default)]
        #[inherit(Node)]
        struct CollisionWatcher {
            notification_channel:
                Option<std::sync::mpsc::Sender<bevy_godot::prelude::CollisionEvent>>,
        }

        #[methods]
        impl CollisionWatcher {
            fn new(_base: &Node) -> Self {
                Self::default()
            }

            #[export]
            fn collision_event(
                &self,
                _base: TRef<Node>,
                target: Ref<Node>,
                origin: Ref<Node>,
                event_type: bevy_godot::prelude::CollisionEventType,
            ) {
                let (origin, target) = unsafe { (origin.assume_safe(), target.assume_safe()) };
                self.notification_channel
                    .as_ref()
                    .unwrap()
                    .send(bevy_godot::prelude::CollisionEvent {
                        event_type,
                        origin: origin.get_instance_id(),
                        target: target.get_instance_id(),
                    })
                    .unwrap();
            }
        }

        godot_init!(__godot_init);
    };
}
