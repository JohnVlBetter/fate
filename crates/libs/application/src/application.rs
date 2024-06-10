use std::collections::HashMap;

use bevy_ecs::{
    component::Component,
    event::{Event, Events},
    schedule::{
        InternedScheduleLabel, IntoSystemConfigs, IntoSystemSetConfigs, Schedule,
        ScheduleBuildSettings, ScheduleLabel, Schedules,
    },
    system::Resource,
    world::{FromWorld, World},
};
use std::fmt::Debug;

use crate::main_schedule::Start;

define_label!(AppLabel, APP_LABEL_INTERNER);

pub struct Application {
    pub world: World,
    pub runner: Box<dyn FnOnce(Application) + Send>,
    pub main_schedule_label: InternedScheduleLabel,
    sub_applications: HashMap<String, SubApplication>,
}

impl Application {
    pub fn new() -> Application {
        Application::default()
    }

    pub fn empty() -> Application {
        let mut world = World::new();
        world.init_resource::<Schedules>();
        Self {
            world,
            runner: Box::new(run_once),
            sub_applications: HashMap::default(),
            main_schedule_label: Main.intern(),
        }
    }

    pub fn update(&mut self) {
        self.world.run_schedule(self.main_schedule_label);
        for (_label, sub_app) in &mut self.sub_apps {
            sub_app.extract(&mut self.world);
            sub_app.run();
        }

        self.world.clear_trackers();
    }

    pub fn run(&mut self) {
        let mut app = std::mem::replace(self, Application::empty());
        let runner = std::mem::replace(&mut app.runner, Box::new(run_once));
        (runner)(app);
    }

    pub fn add_systems<M>(
        &mut self,
        schedule: impl ScheduleLabel,
        systems: impl IntoSystemConfigs<M>,
    ) -> &mut Self {
        let schedule = schedule.intern();
        let mut schedules = self.world.resource_mut::<Schedules>();

        if let Some(schedule) = schedules.get_mut(schedule) {
            schedule.add_systems(systems);
        } else {
            let mut new_schedule = Schedule::new(schedule);
            new_schedule.add_systems(systems);
            schedules.insert(new_schedule);
        }

        self
    }

    #[track_caller]
    pub fn configure_sets(
        &mut self,
        schedule: impl ScheduleLabel,
        sets: impl IntoSystemSetConfigs,
    ) -> &mut Self {
        let schedule = schedule.intern();
        let mut schedules = self.world.resource_mut::<Schedules>();
        if let Some(schedule) = schedules.get_mut(schedule) {
            schedule.configure_sets(sets);
        } else {
            let mut new_schedule = Schedule::new(schedule);
            new_schedule.configure_sets(sets);
            schedules.insert(new_schedule);
        }
        self
    }

    pub fn add_event<T>(&mut self) -> &mut Self
    where
        T: Event,
    {
        if !self.world.contains_resource::<Events<T>>() {
            self.init_resource::<Events<T>>().add_systems(
                Start,
                bevy_ecs::event::event_update_system::<T>
                    .run_if(bevy_ecs::event::event_update_condition::<T>),
            );
        }
        self
    }

    pub fn insert_resource<R: Resource>(&mut self, resource: R) -> &mut Self {
        self.world.insert_resource(resource);
        self
    }

    pub fn insert_non_send_resource<R: 'static>(&mut self, resource: R) -> &mut Self {
        self.world.insert_non_send_resource(resource);
        self
    }

    pub fn init_resource<R: Resource + FromWorld>(&mut self) -> &mut Self {
        self.world.init_resource::<R>();
        self
    }

    pub fn init_non_send_resource<R: 'static + FromWorld>(&mut self) -> &mut Self {
        self.world.init_non_send_resource::<R>();
        self
    }

    pub fn set_runner(&mut self, run_fn: impl FnOnce(Application) + 'static + Send) -> &mut Self {
        self.runner = Box::new(run_fn);
        self
    }

    pub fn sub_app_mut(&mut self, label: impl AppLabel) -> &mut Application {
        match self.get_sub_app_mut(label) {
            Ok(app) => app,
            Err(label) => panic!("Sub-App with label '{:?}' does not exist", label),
        }
    }

    pub fn get_sub_app_mut(
        &mut self,
        label: impl AppLabel,
    ) -> Result<&mut Application, impl AppLabel> {
        self.sub_apps
            .get_mut(&label.intern())
            .map(|sub_app| &mut sub_app.app)
            .ok_or(label)
    }

    pub fn sub_app(&self, label: impl AppLabel) -> &Application {
        match self.get_sub_app(label) {
            Ok(app) => app,
            Err(label) => panic!("Sub-App with label '{:?}' does not exist", label),
        }
    }

    pub fn insert_sub_app(&mut self, label: impl AppLabel, sub_app: SubApplication) {
        self.sub_apps.insert(label.intern(), sub_app);
    }

    pub fn remove_sub_app(&mut self, label: impl AppLabel) -> Option<SubApplication> {
        self.sub_apps.remove(&label.intern())
    }

    pub fn get_sub_app(&self, label: impl AppLabel) -> Result<&Application, impl AppLabel> {
        self.sub_apps
            .get(&label.intern())
            .map(|sub_app| &sub_app.app)
            .ok_or(label)
    }

    pub fn add_schedule(&mut self, schedule: Schedule) -> &mut Self {
        let mut schedules = self.world.resource_mut::<Schedules>();
        schedules.insert(schedule);

        self
    }

    pub fn init_schedule(&mut self, label: impl ScheduleLabel) -> &mut Self {
        let label = label.intern();
        let mut schedules = self.world.resource_mut::<Schedules>();
        if !schedules.contains(label) {
            schedules.insert(Schedule::new(label));
        }
        self
    }

    pub fn get_schedule(&self, label: impl ScheduleLabel) -> Option<&Schedule> {
        let schedules = self.world.get_resource::<Schedules>()?;
        schedules.get(label)
    }

    pub fn get_schedule_mut(&mut self, label: impl ScheduleLabel) -> Option<&mut Schedule> {
        let schedules = self.world.get_resource_mut::<Schedules>()?;
        schedules.into_inner().get_mut(label)
    }

    pub fn edit_schedule(
        &mut self,
        label: impl ScheduleLabel,
        f: impl FnOnce(&mut Schedule),
    ) -> &mut Self {
        let label = label.intern();
        let mut schedules = self.world.resource_mut::<Schedules>();

        if schedules.get(label).is_none() {
            schedules.insert(Schedule::new(label));
        }

        let schedule = schedules.get_mut(label).unwrap();
        f(schedule);

        self
    }

    pub fn configure_schedules(
        &mut self,
        schedule_build_settings: ScheduleBuildSettings,
    ) -> &mut Self {
        self.world
            .resource_mut::<Schedules>()
            .configure_schedules(schedule_build_settings);
        self
    }

    pub fn allow_ambiguous_component<T: Component>(&mut self) -> &mut Self {
        self.world.allow_ambiguous_component::<T>();
        self
    }

    pub fn allow_ambiguous_resource<T: Resource>(&mut self) -> &mut Self {
        self.world.allow_ambiguous_resource::<T>();
        self
    }
}

impl Debug for Application {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "App {{ SubApplication: ")?;
        f.debug_map()
            .entries(self.sub_applications.iter().map(|(k, v)| (k, v)))
            .finish()?;
        write!(f, "}}")
    }
}

impl Default for Application {
    fn default() -> Self {
        let mut app = Application::empty();
        app
    }
}

pub struct SubApplication {
    pub app: Application,

    extract: Box<dyn Fn(&mut World, &mut Application) + Send>,
}

impl Debug for SubApplication {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SubApplication {{ app: ")?;
        f.debug_map()
            .entries(self.app.sub_applications.iter().map(|(k, v)| (k, v)))
            .finish()?;
        write!(f, "}}")
    }
}

fn run_once(mut app: Application) -> () {
    app.finish();
    app.cleanup();

    app.update();
}
