use bevy_ecs::{
    schedule::{ExecutorKind, InternedScheduleLabel, Schedule, ScheduleLabel},
    system::{Local, Resource},
    world::{Mut, World},
};

use crate::application::Application;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Main;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PreStartup;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Startup;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PostStartup;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Start;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PreUpdate;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Update;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PostUpdate;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Last;

#[derive(Resource, Debug)]
pub struct MainScheduleOrder {
    pub labels: Vec<InternedScheduleLabel>,
    pub startup_labels: Vec<InternedScheduleLabel>,
}

impl Default for MainScheduleOrder {
    fn default() -> Self {
        Self {
            labels: vec![
                Start.intern(),
                PreUpdate.intern(),
                Update.intern(),
                PostUpdate.intern(),
                Last.intern(),
            ],
            startup_labels: vec![PreStartup.intern(), Startup.intern(), PostStartup.intern()],
        }
    }
}

impl MainScheduleOrder {
    pub fn insert_after(&mut self, after: impl ScheduleLabel, schedule: impl ScheduleLabel) {
        let index = self
            .labels
            .iter()
            .position(|current| (**current).eq(&after))
            .unwrap_or_else(|| panic!("Expected {after:?} to exist"));
        self.labels.insert(index + 1, schedule.intern());
    }

    pub fn insert_startup_after(
        &mut self,
        after: impl ScheduleLabel,
        schedule: impl ScheduleLabel,
    ) {
        let index = self
            .startup_labels
            .iter()
            .position(|current| (**current).eq(&after))
            .unwrap_or_else(|| panic!("Expected {after:?} to exist"));
        self.startup_labels.insert(index + 1, schedule.intern());
    }
}

impl Main {
    pub fn build(&self, app: &mut Application) {
        let mut main_schedule = Schedule::new(Main);
        main_schedule.set_executor_kind(ExecutorKind::SingleThreaded);

        app.add_schedule(main_schedule)
            .init_resource::<MainScheduleOrder>()
            .add_systems(Main, Main::run_main);
    }

    pub fn run_main(world: &mut World, mut run_at_least_once: Local<bool>) {
        if !*run_at_least_once {
            world.resource_scope(|world, order: Mut<MainScheduleOrder>| {
                for &label in &order.startup_labels {
                    let _ = world.try_run_schedule(label);
                }
            });
            *run_at_least_once = true;
        }

        world.resource_scope(|world, order: Mut<MainScheduleOrder>| {
            for &label in &order.labels {
                let _ = world.try_run_schedule(label);
            }
        });
    }
}
