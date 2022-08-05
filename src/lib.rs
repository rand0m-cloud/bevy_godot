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
            init.add_class::<signal_watcher::GodotSignalWatcher>();
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

            {
                let (sender, reciever) = std::sync::mpsc::channel();
                let signal_watcher = signal_watcher::GodotSignalWatcher::new_instance();
                signal_watcher
                    .map_mut(|script, base| script.notification_channel = Some(sender))
                    .unwrap();
                signal_watcher.base().set_name("GodotSignalWatcher");

                base.add_child(signal_watcher.into_base().into_shared(), true);

                app.insert_non_send_resource(bevy_godot::prelude::GodotSignalReader(reciever));
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

        mod signal_watcher {
            use bevy_godot::prelude::{godot_prelude::Variant, *, bevy_prelude::trace};

            #[derive(NativeClass, Default)]
            #[inherit(Node)]
            pub struct GodotSignalWatcher {
                pub notification_channel:
                    Option<std::sync::mpsc::Sender<bevy_godot::prelude::GodotSignal>>,
            }

            #[methods]
            impl GodotSignalWatcher {
                fn new(_base: &Node) -> Self {
                    Self::default()
                }

                #[allow(clippy::too_many_arguments)]
                #[export]
                fn event(
                    &self,
                    base: TRef<Node>,
                    #[opt]
                    arg_1: Option<Variant>,
                    #[opt]
                    arg_2: Option<Variant>,
                    #[opt]
                    arg_3: Option<Variant>,
                    #[opt]
                    arg_4: Option<Variant>,
                    #[opt]
                    arg_5: Option<Variant>,
                    #[opt]
                    arg_6: Option<Variant>,
                    #[opt]
                    arg_7: Option<Variant>,
                    #[opt]
                    arg_8: Option<Variant>,
                    #[opt]
                    arg_9: Option<Variant>,
                ) {
                    let args = vec![
                        arg_1, arg_2, arg_3, arg_4, arg_5, arg_6, arg_7, arg_8, arg_9,
                    ].into_iter().flatten().collect::<Vec<_>>();

                    let signal_args = args
                        .iter()
                        .take_while(|arg| **arg != Variant::new(base))
                        .cloned()
                        .collect::<Vec<_>>();
                    let origin = args[signal_args.len() + 1].clone();
                    let signal_name = args[signal_args.len() + 2].clone();

                    let signal_event = bevy_godot::prelude::GodotSignal::new(
                        signal_name.try_to::<String>().unwrap(),
                        unsafe { origin.try_to_object::<Object>().unwrap().assume_safe() },
                        signal_args,
                    );

                    trace!(target: "godot_signal", signal = ?signal_event);

                    self.notification_channel
                        .as_ref()
                        .unwrap()
                        .send(signal_event)
                        .unwrap();
                }
            }
        }

        godot_init!(__godot_init);
    };
}
