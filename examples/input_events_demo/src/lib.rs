use bevy_godot::prelude::{
    godot_prelude::{InputEvent, InputEventMouseMotion},
    *,
};

fn init(_handle: &InitHandle) {}

fn build_app(app: &mut App) {
    app.add_system(print_mouse_motion);
}

bevy_godot_init!(init, build_app);

fn print_mouse_motion(mut input_events: EventReader<Ref<InputEvent>>) {
    for input_event in input_events.iter() {
        let input_event = unsafe { input_event.assume_safe().cast::<InputEventMouseMotion>() };

        match input_event {
            Some(evt) => println!("{:?}", evt.speed()),
            None => (),
        }
    }
}
