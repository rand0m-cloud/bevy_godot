use bevy_godot::prelude::{
    godot_prelude::{Label, ProgressBar},
    *,
};

fn init(_handle: &InitHandle) {}

fn build_app(app: &mut App) {
    app.add_startup_system(setup_ui)
        .add_system(update_ui.as_visual_system());
}

bevy_godot_init!(init, build_app);

#[derive(NodeTreeView, Component)]
pub struct Ui {
    #[node("Ui/ProgressBar")]
    progress_bar: ErasedGodotRef,

    #[node("Ui/Label")]
    label: ErasedGodotRef,

    #[node("MissingNode")]
    _missing_node: Option<ErasedGodotRef>,
}

fn setup_ui(mut commands: Commands, mut entities: Query<(&Name, &mut ErasedGodotRef)>) {
    let mut ui_canvas = entities
        .iter_mut()
        .find_entity_by_name("UiCanvasLayer")
        .unwrap();
    let ui = Ui::from_node(&ui_canvas.get::<Node>());

    commands.spawn().insert(ui);
}

fn update_ui(mut ui: Query<&mut Ui>, mut time: SystemDelta) {
    let delta = time.delta_seconds_f64();
    let mut ui = ui.single_mut();

    let progress_bar = ui.progress_bar.get::<ProgressBar>();
    let value = progress_bar.value() + delta;
    progress_bar.set_value(value);

    let label = ui.label.get::<Label>();
    label.set_text(format!("{:0.3}", delta));
}
