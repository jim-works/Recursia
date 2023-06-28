use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use big_brain::{prelude::*, scorers::FixedScorerBuilder};

use crate::{
    physics::PhysicsObjectBundle,
    ui::healthbar::{spawn_billboard_healthbar, HealthbarResources},
    world::LevelLoadState,
};

use super::{
    personality::components::*, CombatInfo, CombatantBundle, IdleAction, UninitializedActor, Idler,
};

#[derive(Resource)]
pub struct GlowjellyResources {
    pub anim: Handle<AnimationClip>,
    pub scene: Handle<Scene>,
    pub action_frame: f32,
}

#[derive(Component, Default)]
pub struct Glowjelly {
    scene: Option<Entity>,
    color: Color
}

#[derive(Component)]
pub struct GlowjellyScene;

pub struct SpawnGlowjellyEvent {
    pub location: Transform,
    pub color: Color,
}
#[derive(Component, Debug)]
pub struct FloatHeight {
    pub curr_height: f32,
    pub preferred_height: f32,
    pub seconds_elapsed: f32,
    pub floated: bool
}

impl FloatHeight {
    pub fn new(preferred_height: f32) -> Self {
        Self {
            curr_height: 0.0,
            preferred_height,
            seconds_elapsed: 0.0,
            floated: false,
        }
    }
}

#[derive(Clone, Component, Debug, ActionBuilder)]
pub struct FloatAction;

#[derive(Clone, Component, Debug, ScorerBuilder)]
pub struct FloatScorer;

pub struct GlowjellyPlugin;

impl Plugin for GlowjellyPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(load_resources)
            .add_system(trigger_spawning.in_schedule(OnEnter(LevelLoadState::Loaded)))
            .add_system(spawn_glowjelly)
            .add_system(eval_height)
            .add_system(setup_glowjelly)
            .add_system(float_scorer_system.in_set(BigBrainSet::Scorers))
            .add_system(float_action_system.in_set(BigBrainSet::Actions))
            .add_event::<SpawnGlowjellyEvent>();
    }
}

pub fn load_resources(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(GlowjellyResources {
        anim: assets.load("glowjelly/glowjelly.gltf#Animation0"),
        scene: assets.load("glowjelly/glowjelly.gltf#Scene0"),
        action_frame: 1.1,
    });
}

fn trigger_spawning(mut writer: EventWriter<SpawnGlowjellyEvent>) {
    for i in 0..5 {
        writer.send(SpawnGlowjellyEvent {
            location: Transform::from_xyz(i as f32 * 5.0, -45.0, 0.0),
            color: Color::rgb(i as f32, 1.0, 1.0),
        });
    }
}

pub fn spawn_glowjelly(
    mut commands: Commands,
    jelly_res: Res<GlowjellyResources>,
    mut spawn_requests: EventReader<SpawnGlowjellyEvent>,
    healthbar_resources: Res<HealthbarResources>,
    _children_query: Query<&Children>,
) {
    for spawn in spawn_requests.iter() {
        let id = commands
            .spawn((
                SceneBundle {
                    scene: jelly_res.scene.clone_weak(),
                    transform: spawn.location,
                    ..default()
                },
                Name::new("glowjelly"),
                CombatantBundle {
                    combat_info: CombatInfo {
                        knockback_multiplier: 10.0,
                        ..CombatInfo::new(10.0, 0.0)
                    },
                    ..default()
                },
                PhysicsObjectBundle {
                    rigidbody: RigidBody::Dynamic,
                    collider: Collider::cuboid(0.5, 0.5, 0.5),
                    ..default()
                },
                PersonalityBundle {
                    personality: PersonalityValues {
                        family: FacetValue::new(0.0, 1.0).unwrap(),
                        power: FacetValue::new(0.0, 1.0).unwrap(),
                        tradition: FacetValue::new(0.0, 1.0).unwrap(),
                        wealth: FacetValue::new(0.0, 1.0).unwrap(),
                        status: FacetValue::new(0.0, 1.0).unwrap(),
                        hedonism: FacetValue::new(0.0, 1.0).unwrap(),
                        excitement: FacetValue::new(0.0, 1.0).unwrap(),
                        pacifism: FacetValue::new(0.0, 1.0).unwrap(),
                    },
                    mental_attributes: MentalAttributes {
                        willpower: FacetValue::new(0.0, 1.0).unwrap(),
                        creativity: FacetValue::new(0.0, 1.0).unwrap(),
                        memory: FacetValue::new(0.0, 1.0).unwrap(),
                        patience: FacetValue::new(0.0, 1.0).unwrap(),
                        empathy: FacetValue::new(0.0, 1.0).unwrap(),
                        persistence: FacetValue::new(0.0, 1.0).unwrap(),
                        intelligence: FacetValue::new(0.0, 1.0).unwrap(),
                        social_awareness: FacetValue::new(0.0, 1.0).unwrap(),
                    },
                    physical_attributes: PhysicalAttributes {
                        strength: FacetValue::new(0.0, 1.0).unwrap(),
                        agility: FacetValue::new(0.0, 1.0).unwrap(),
                        disease_resistence: FacetValue::new(0.0, 1.0).unwrap(),
                        fortitude: FacetValue::new(0.0, 1.0).unwrap(),
                    },
                    tasks: TaskSet {
                        dream: None,
                        long_term: None,
                        short_term: None,
                    },
                },
                GravityScale(0.2),
                Glowjelly {
                    color: spawn.color,
                    ..default()
                },
                UninitializedActor,
                FloatHeight::new(20.0),
                Idler::default(),
                Thinker::build()
                    .label("glowjelly thinker")
                    .picker(FirstToScore::new(0.1))
                    .when(FloatScorer, FloatAction)
                    .otherwise(IdleAction {seconds: 1.0})
            ))
            .id();
        //add healthbar
        spawn_billboard_healthbar(
            &mut commands,
            &healthbar_resources,
            id,
            Vec3::new(0.0, 2.0, 0.0),
        );

    }
}

//TODO: extract into separate function. all scenes should have this setup
pub fn setup_glowjelly(
    mut commands: Commands,
    children_query: Query<&Children>,
    mut glowjelly_query: Query<(Entity, &mut Glowjelly), With<UninitializedActor>>,
    anim_query: Query<&AnimationPlayer>,
) {
    for (parent_id, mut glowjelly) in glowjelly_query.iter_mut() {
        //hierarchy is parent -> scene -> gltfnode (with animation player)
        //find first child with a child that has an animation player
        //not perfect but whatevs
        // info!("glowjelly id: {:?}", parent_id);
        if let Ok(children) = children_query.get(parent_id) {
            for child in children {
                let mut found = false;
                if let Ok(grandchildren) = children_query.get(*child) {
                    for candidate_anim_player in grandchildren {
                        if anim_query.contains(*candidate_anim_player) {
                            commands
                                .entity(*candidate_anim_player)
                                .insert(GlowjellyScene);
                            commands.entity(parent_id).remove::<UninitializedActor>()
                                .with_children(|cb| {
                                    cb.spawn(PointLightBundle {
                                        point_light: PointLight {
                                            color: glowjelly.color,
                                            intensity: 100.0,
                                            shadows_enabled: true,
                                            ..default()
                                        },
                                        ..default()
                                    });
                                });;
                            glowjelly.scene = Some(*candidate_anim_player);
                            
                            found = true;
                            break;
                        } else {
                            error!("glowjelly animation player not found");
                        }
                    }
                }
                if found {
                    break;
                }
            }
        }
    }
}

pub fn eval_height(
    collision: Res<RapierContext>,
    mut query: Query<(&mut FloatHeight, &GlobalTransform)>,
) {
    let groups = QueryFilter {
        groups: Some(CollisionGroups::new(
            Group::ALL,
            Group::from_bits_truncate(crate::physics::TERRAIN_GROUP),
        )),
        ..default()
    };
    for (mut height, tf) in query.iter_mut() {
        height.curr_height = if let Some((_, dist)) = collision.cast_ray(
            tf.translation(),
            Vec3::NEG_Y,
            height.preferred_height,
            true,
            groups,
        ) {
            dist
        } else {
            height.preferred_height
        };
    }
}
//TODO: extract and make generic so we can use it for other ai
pub fn float_action_system(
    time: Res<Time>,
    _jelly_anim: Res<GlowjellyResources>,
    mut info: Query<(&Glowjelly, &mut FloatHeight, &mut ExternalImpulse)>,
    mut query: Query<(&Actor, &mut ActionState), With<FloatAction>>,
    jelly_anim: Res<GlowjellyResources>,
    mut animation_player: Query<&mut AnimationPlayer>,
    animations: Res<Assets<AnimationClip>>,
) {
    for (Actor(actor), mut state) in query.iter_mut() {
        if let Ok((jelly, mut floater, mut impulse)) = info.get_mut(*actor) {
            match *state {
                ActionState::Requested => {
                    *state = ActionState::Executing;
                    floater.seconds_elapsed = 0.0;
                    floater.floated = false;
                    if let Some(scene) = jelly.scene {
                        if let Ok(mut anim_player) = animation_player.get_mut(scene) {
                            anim_player.start(jelly_anim.anim.clone_weak());
                        }
                    }
                }
                ActionState::Executing => {
                    floater.seconds_elapsed += time.delta_seconds();
                    if !floater.floated && floater.seconds_elapsed >= jelly_anim.action_frame {
                        impulse.impulse += Vec3::Y * 5.0;
                        floater.floated = true;
                    }
                    //if there's an animation, wait for it to finish
                    if let Some(anim) = animations.get(&jelly_anim.anim) {
                        if floater.seconds_elapsed >= anim.duration() {
                            *state = ActionState::Success;
                        }
                    } else {
                        *state = ActionState::Success;
                    }
                }
                ActionState::Cancelled => {
                    *state = ActionState::Failure;
                }
                _ => {}
            }
        }
    }
}

pub fn float_scorer_system(
    floats: Query<&FloatHeight>,
    mut query: Query<(&Actor, &mut Score), With<FloatScorer>>,
) {
    for (Actor(actor), mut score) in query.iter_mut() {
        if let Ok(float) = floats.get(*actor) {
            score.set((float.preferred_height - float.curr_height) / float.preferred_height);
        }
    }
}
