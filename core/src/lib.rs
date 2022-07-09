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
            app: App,
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
                self.app.update();
            }
        }

        fn __godot_init(init: InitHandle) {
            init.add_class::<Autoload>();
            init.add_class::<SceneTreeWatcher>();
            $init(&init);
        }

        fn __app(autoload: &mut Autoload, base: &Node) {
            let mut app = App::new();
            let (sender, reciever) = std::sync::mpsc::channel();
            app.add_plugin(bevy_godot::prelude::GodotPlugin)
                .insert_non_send_resource(reciever);
            $app(&mut app);

            autoload.app = app;

            let scene_tree_watcher = SceneTreeWatcher::new_instance();
            scene_tree_watcher
                .map_mut(|script, base| script.notification_channel = Some(sender))
                .unwrap();

            base.add_child(scene_tree_watcher.into_base().into_shared(), true);
        }

        #[derive(NativeClass, Default)]
        #[inherit(Node)]
        struct SceneTreeWatcher {
            notification_channel: Option<std::sync::mpsc::Sender<()>>,
        }

        #[methods]
        impl SceneTreeWatcher {
            fn new(_base: &Node) -> Self {
                Self::default()
            }

            #[export]
            fn _ready(&self, base: TRef<Node>) {
                unsafe {
                    base.get_tree()
                        .unwrap()
                        .assume_safe()
                        .connect(
                            "tree_changed",
                            base,
                            "scene_tree_modified",
                            VariantArray::default(),
                            0,
                        )
                        .unwrap()
                };
            }

            #[export]
            fn scene_tree_modified(&self, _base: TRef<Node>) {
                if let Some(channel) = self.notification_channel.clone() {
                    channel.send(()).unwrap()
                };
            }
        }

        godot_init!(__godot_init);
    };
}
