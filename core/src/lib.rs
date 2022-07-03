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
        struct MyApp(App);
        unsafe impl Send for MyApp {}

        #[derive(NativeClass, Default)]
        #[inherit(Node)]
        struct Autoload {
            app: std::sync::Mutex<Option<MyApp>>,
            notification_channel: Option<std::sync::mpsc::Sender<()>>,
        }

        #[methods]
        impl Autoload {
            fn new(_base: &Node) -> Self {
                Self::default()
            }

            #[export]
            fn _ready(&mut self, base: TRef<Node>) {
                let (app, sender) = __app();
                *self.app.lock().unwrap() = Some(MyApp(app));
                self.notification_channel = Some(sender);

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
            fn _process(&self, _base: TRef<Node>, _delta: f32) {
                self.app.lock().unwrap().as_mut().unwrap().0.update();
            }

            #[export]
            fn scene_tree_modified(&self, _base: TRef<Node>) {
                if let Some(channel) = self.notification_channel.clone() {
                    channel.send(()).unwrap()
                };
            }
        }

        fn __godot_init(init: InitHandle) {
            init.add_class::<Autoload>();
            $init(&init);
        }

        fn __app() -> (App, std::sync::mpsc::Sender<()>) {
            let mut app = App::new();
            let (sender, reciever) = std::sync::mpsc::channel();
            app.add_plugin(bevy_godot::prelude::GodotPlugin)
                .insert_non_send_resource(reciever);
            $app(&mut app);

            (app, sender)
        }

        godot_init!(__godot_init);
    };
}
