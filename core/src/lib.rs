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
        #[derive(NativeClass)]
        #[inherit(Node)]
        struct Autoload {
            app: Option<App>,
        }

        #[methods]
        impl Autoload {
            fn new(_base: &Node) -> Self {
                Self { app: None }
            }

            #[export]
            fn _ready(&mut self, _base: TRef<Node>) {
                self.app = Some(__app());
            }

            #[export]
            fn _process(&mut self, _base: TRef<Node>, _delta: f32) {
                self.app.as_mut().unwrap().update();
            }
        }

        fn __godot_init(init: InitHandle) {
            init.add_class::<Autoload>();
            $init(&init);
        }

        fn __app() -> App {
            let mut app = App::new();

            app.add_plugin(bevy_godot::prelude::GodotPlugin);
            $app(&mut app);

            app
        }

        godot_init!(__godot_init);
    };
}
