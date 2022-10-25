use bevy_godot::prelude::{godot_prelude::InputEventMouseMotion, *};

fn init(_handle: &InitHandle) {}

fn build_app(app: &mut App) {
    app.add_system(print_mouse_motion);
}

bevy_godot_init!(init, build_app);

fn print_mouse_motion(mut input_events: EventReader<UnhandledInputEvent>) {
    for input_event in input_events.iter() {
        if let Some(evt) = input_event.try_get::<InputEventMouseMotion>() {
            println!("{:?}", evt.speed());
        }
    }
}
