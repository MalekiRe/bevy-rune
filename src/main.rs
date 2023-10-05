mod help;
mod magic;

use crate::help::{AddDynamicComponent, ComponentIdToFn, RuneModule, RunePlugin};
use bevy::app::{App, PostStartup, Update};
use bevy::ecs::component::ComponentId;
use bevy::log::error;
use bevy::prelude::{
    shape, warn, Assets, Camera2dBundle, Color, ColorMaterial, Commands, Component, FetchedTerms,
    Mesh, Mut, Query, QueryBuilder, Reflect, Res, ResMut, Resource, Startup, Transform, Vec3,
    World,
};
use bevy::ptr::{Ptr, PtrMut};
use bevy::reflect::TypePath;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::utils::default;
use bevy::{DefaultPlugins, MinimalPlugins};
use rune::compile::Named;
use rune::diagnostics::Diagnostic;
use rune::runtime::{
    Args, GuardedArgs, OwnedTuple, RawStr, SharedPointerGuard, Stack, Type, UnsafeToMut,
    UnsafeToValue, VmError, VmExecution, VmResult,
};
use rune::termcolor::{ColorChoice, StandardStream};
use rune::{Any, Context, Diagnostics, Hash, Source, Sources, ToTypeHash, ToValue, Value, Vm};
use std::cell::OnceCell;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};

fn main() {
    println!("Hello, world!");
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(RunePlugin);
    app.add_dynamic_component::<Stretch>();
    app.add_systems(Update, every_tick);
    app.add_systems(PostStartup, startup);
    app.run();
}
#[derive(Component, Any, TypePath, Debug)]
pub struct Stretch {
    #[rune(get, set, add_assign, copy)]
    x: f32,
    #[rune(get, set, add_assign, copy)]
    y: f32,
    #[rune(get, set)]
    test: TestStruct,
}

pub fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut sources: ResMut<RuneSources>,
) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(50.).into()).into(),
            material: materials.add(ColorMaterial::from(Color::PURPLE)),
            transform: Transform::from_translation(Vec3::new(-150., 0., 0.)),
            ..default()
        },
        Stretch {
            x: 1.0,
            y: 1.0,
            test: TestStruct { yo: 0 },
        },
    ));
    /*let mut s = Sources::new();
    s.insert(Source::from_path("./src/query.rune").unwrap())
        .unwrap();
    sources.0.push(s);*/
}

pub fn every_tick(q: Query<&Stretch>) {
    for s in q.iter() {
        //println!("{:#?}", s);
    }
}

#[derive(Any, TypePath, Debug, Clone)]
pub struct TestStruct {
    #[rune(get, set, add_assign, copy)]
    yo: i64,
}

#[derive(Resource)]
pub struct RuneContext(pub rune::Context);
#[derive(Resource)]
pub struct RuneRuntime(pub Arc<rune::runtime::RuntimeContext>);
#[derive(Resource)]
pub struct RuneDiagnostics(pub rune::diagnostics::Diagnostics);

#[derive(Resource)]
pub struct RuneSources(pub Vec<Sources>);
