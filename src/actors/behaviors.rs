use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use big_brain::prelude::*;
use rand::thread_rng;

use crate::{util::{sample_sphere_surface, plugin::SmoothLookTo}, actors::personality::scoring};

use super::{personality::components::*, DefaultAnimation, setup_animation, setup_animation_with_speed};

pub struct BehaviorsPlugin;

impl Plugin for BehaviorsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(float_scorer_system.in_set(BigBrainSet::Scorers))
            .add_systems((float_wander_action_system, float_action_system).in_set(BigBrainSet::Actions))
            .add_system(eval_height)
        ;
    }
}

#[derive(Component, Debug)]
pub struct FloatWander {
    pub target_direction: Vec3,
    pub task: Task,
}

impl Default for FloatWander {
    fn default() -> Self {
        Self {
            target_direction: Default::default(),
            task: Task {
                category: TaskCategory::Exploring,
                risks: Default::default(),
                outcomes: Default::default(),
            },
        }
    }
}

#[derive(Clone, Component, Debug, ActionBuilder)]
pub struct FloatWanderAction {
    pub impulse: f32,
    pub turn_speed: f32,
    pub squish_factor: Vec3,
    pub anim_speed: f32,
}

pub fn float_wander_action_system(
    time: Res<Time>,
    mut info: Query<(
        Entity,
        Option<&mut DefaultAnimation>,
        &mut FloatWander,
        &mut ExternalImpulse,
    )>,
    mut query: Query<(&Actor, &mut ActionState, &FloatWanderAction)>,
    mut animation_player: Query<&mut AnimationPlayer>,
    mut commands: Commands,
) {
    for (Actor(actor), mut state, wander) in query.iter_mut() {
        if let Ok((entity, anim_opt, mut floater, mut impulse)) = info.get_mut(*actor) {
            match *state {
                ActionState::Requested => {
                    *state = ActionState::Executing;
                    floater.target_direction = sample_sphere_surface(&mut thread_rng())*wander.squish_factor * wander.impulse;
                    commands.entity(entity).insert(SmoothLookTo{
                        to: floater.target_direction,
                        up: Vec3::Y,
                        speed: wander.turn_speed,
                    });
                    
                    setup_animation_with_speed(anim_opt, &mut animation_player, wander.anim_speed);
                }
                ActionState::Executing | ActionState::Cancelled => {
                    match anim_opt {
                        Some(mut anim) => {
                            //time according to animation
                            anim.tick(time.delta_seconds());
                            if anim.just_acted() {
                                impulse.impulse += floater.target_direction;
                            }
                            if anim.finished() {
                                *state = ActionState::Success;
                            }
                        }
                        None => {
                            //no animation, so execute immediately
                            impulse.impulse += floater.target_direction;
                            *state = ActionState::Success;
                        }
                    }
                }
                _ => {}
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
            2.0*height.preferred_height,
            true,
            groups,
        ) {
            dist
        } else {
            2.0*height.preferred_height
        };
        //height.task.outcomes.status = (height.preferred_height-height.curr_height)/height.preferred_height;
        //height.task.risks.physical_danger = (height.curr_height/height.preferred_height).powi(2);
    }
}

#[derive(Component, Debug)]
pub struct FloatHeight {
    pub curr_height: f32,
    pub preferred_height: f32,
    pub task: Task
}

impl FloatHeight {
    pub fn new(preferred_height: f32) -> Self {
        Self {
            curr_height: 0.0,
            preferred_height,
            task: Task {
                category: TaskCategory::Socializing,
                risks: TaskRisks::default(),
                outcomes: TaskOutcomes::default()
            }
        }
    }
}

#[derive(Clone, Component, Debug, ActionBuilder)]
pub struct FloatAction {
    pub impulse: f32,
    pub turn_speed: f32,
}

#[derive(Clone, Component, Debug, ScorerBuilder)]
pub struct FloatScorer;

pub fn float_action_system(
    time: Res<Time>,
    mut info: Query<(
        Entity,
        Option<&mut DefaultAnimation>,
        &mut ExternalImpulse
    ), With<FloatHeight>>,
    mut query: Query<(&Actor, &mut ActionState, &FloatAction)>,
    mut animation_player: Query<&mut AnimationPlayer>,
    mut commands: Commands
) {
    for (Actor(actor), mut state, float) in query.iter_mut() {
        if let Ok((entity, anim_opt, mut impulse)) = info.get_mut(*actor) {
            match *state {
                ActionState::Requested => {
                    *state = ActionState::Executing;
                    setup_animation(anim_opt, &mut animation_player);
                    commands.entity(entity).insert(SmoothLookTo{
                        to: Vec3::Y,
                        up: Vec3::X,
                        speed: float.turn_speed,
                    });
                }
                ActionState::Executing => {
                    match anim_opt {
                        Some(mut anim) => {
                            anim.tick(time.delta_seconds());
                            //time according to animation
                            if anim.just_acted()
                            {
                                impulse.impulse += Vec3::Y * float.impulse;
                            }
                            if anim.finished() {
                                *state = ActionState::Success;
                            }
                        }
                        None => {
                            //no animation, so execute immediately
                            impulse.impulse += Vec3::Y * float.impulse;
                            *state = ActionState::Success;
                        }
                    }
                },
                ActionState::Cancelled => {
                    *state = ActionState::Failure
                }
                _ => {}
            }
        }
    }
}

pub fn float_scorer_system(
    floats: Query<(&FloatHeight, &PersonalityValues, &MentalAttributes, &PhysicalAttributes, &TaskSet)>,
    mut query: Query<(&Actor, &mut Score), With<FloatScorer>>,
) {
    for (Actor(actor), mut score) in query.iter_mut() {
        if let Ok((float, values, mental, physical, tasks)) = floats.get(*actor) {
            score.set_unchecked(scoring::score_task(&mut float.task.clone(), physical, mental, values, tasks).0.overall());
        }
    }
}
