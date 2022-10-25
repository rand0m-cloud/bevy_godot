use crate::prelude::{
    godot_prelude::{FromVariant, ToVariant},
    *,
};

pub struct GodotCollisionsPlugin;

impl Plugin for GodotCollisionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::PreUpdate, update_godot_collisions)
            .add_system_to_stage(
                CoreStage::First,
                write_godot_collision_events.before(Events::<CollisionEvent>::update_system),
            )
            .add_event::<CollisionEvent>();
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Component, Default)]
pub struct Collisions {
    colliding_entities: Vec<Entity>,
    recent_collisions: Vec<Entity>,
}

impl Collisions {
    pub fn colliding(&self) -> &[Entity] {
        &self.colliding_entities
    }

    pub fn recent_collisions(&self) -> &[Entity] {
        &self.recent_collisions
    }
}

#[doc(hidden)]
pub struct CollisionEventReader(pub std::sync::mpsc::Receiver<CollisionEvent>);

#[doc(hidden)]
#[derive(ToVariant, FromVariant, Debug)]
pub enum CollisionEventType {
    Started,
    Ended,
}

#[derive(Debug)]
pub struct CollisionEvent {
    pub event_type: CollisionEventType,
    pub origin: i64,
    pub target: i64,
}

fn write_godot_collision_events(
    events: NonSendMut<CollisionEventReader>,
    mut event_writer: EventWriter<CollisionEvent>,
) {
    event_writer.send_batch(events.0.try_iter());
}

fn update_godot_collisions(
    mut events: EventReader<CollisionEvent>,
    mut entities: Query<(&ErasedGodotRef, &mut Collisions)>,
    all_entities: Query<(Entity, &ErasedGodotRef)>,
) {
    for (_, mut collisions) in entities.iter_mut() {
        collisions.recent_collisions = vec![];
    }

    for event in events.iter() {
        trace!(target: "godot_collisions_update", event = ?event);

        let target = all_entities.iter().find_map(|(ent, reference)| {
            if reference.instance_id() == event.target {
                Some(ent)
            } else {
                None
            }
        });
        let collisions = entities.iter_mut().find_map(|(reference, collisions)| {
            if reference.instance_id() == event.origin {
                Some(collisions)
            } else {
                None
            }
        });

        let (target, mut collisions) = match (target, collisions) {
            (Some(target), Some(collisions)) => (target, collisions),
            _ => return,
        };

        match event.event_type {
            CollisionEventType::Started => {
                collisions.colliding_entities.push(target);
                collisions.recent_collisions.push(target);
            }
            CollisionEventType::Ended => collisions.colliding_entities.retain(|x| *x != target),
        };
    }
}
