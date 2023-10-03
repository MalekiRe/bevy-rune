mod help;

use bevy::app::{App, PostStartup, Update};
use bevy::ecs::component::ComponentId;
use bevy::prelude::{
    shape, Assets, Camera2dBundle, Color, ColorMaterial, Commands, Component, FetchedTerms, Mesh,
    Mut, Query, QueryBuilder, Reflect, Res, ResMut, Resource, Startup, Transform, Vec3, World,
};
use bevy::ptr::{Ptr, PtrMut};
use bevy::reflect::TypePath;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::utils::default;
use bevy::{DefaultPlugins, MinimalPlugins};
use rune::compile::Named;
use rune::diagnostics::Diagnostic;
use rune::runtime::{Args, GuardedArgs, OwnedTuple, RawStr, SharedPointerGuard, Stack, Type, UnsafeToMut, UnsafeToValue, VmError, VmExecution, VmResult};
use rune::termcolor::{ColorChoice, StandardStream};
use rune::{Any, Context, Diagnostics, Hash, Source, Sources, ToTypeHash, ToValue, Value, Vm};
use std::cell::OnceCell;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::sync::{Arc, Mutex};
use crate::help::{AddDynamicComponent, ComponentIdToFn, RuneModule, RunePlugin, TypePathToComponentId};

fn main() {
    println!("Hello, world!");
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(RunePlugin);
    app.add_dynamic_component::<Stretch>();
    app.add_systems(PostStartup, startup);
    app.add_systems(Update, every_tick);
    app.add_systems(Update, query_test);
    // app.insert_resource(IdToValueMap::default());
    // app.insert_resource(ComponentIdToNameMap::default());
    // app.add_systems(Startup, other_startup);
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
// pub fn other_startup(world: &mut World) {
//     let stretch_id = world.init_component::<Stretch>();
//     world
//         .get_resource_mut::<IdToValueMap>()
//         .unwrap()
//         .as_mut()
//         .0
//         .insert(
//             stretch_id,
//             Box::new(|terms: &mut FetchedTerms, index: usize| unsafe {
//                 terms
//                     .fetch::<&mut Stretch>(index)
//                     .as_mut()
//                     .unsafe_to_value()
//                     .unwrap()
//             }),
//         );
//     world
//         .get_resource_mut::<ComponentIdToNameMap>()
//         .unwrap()
//         .0
//         .insert("Stretch".to_string(), stretch_id);
// }

pub fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut rune_module: ResMut<RuneModule>,
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
    let mut context = Context::with_default_modules().unwrap();
    context.install(&rune_module.0).unwrap();
    let runtime = context.runtime().unwrap();
    let mut source = Sources::new();
    source
        .insert(Source::from_path(Path::new("./src/test.rune")).unwrap())
        .unwrap();
    let diagnostics = Diagnostics::new();

    commands.insert_resource(RuneContext(context));
    commands.insert_resource(RuneRuntime(Arc::new(runtime)));
    commands.insert_resource(RuneSources(source));
    commands.insert_resource(RuneDiagnostics(diagnostics));
}

pub fn query_test(world: &mut World) {
    let component_id_to_fn = world.remove_resource::<ComponentIdToFn>().unwrap();
    let type_path_to_component_id = world.remove_resource::<TypePathToComponentId>().unwrap();
    let mut sources = world.remove_resource::<RuneSources>().unwrap();
    let context = world.remove_resource::<RuneContext>().unwrap();
    let mut diagnostics = world.remove_resource::<RuneDiagnostics>().unwrap();
    let runtime = world.remove_resource::<RuneRuntime>().unwrap();
    let result = rune::prepare(&mut sources)
        .with_context(&context.0)
        .with_diagnostics(&mut diagnostics.0)
        .build();

    if !diagnostics.0.is_empty() {
        let mut writer = StandardStream::stderr(ColorChoice::Always);
        diagnostics.0.emit(&mut writer, &sources).unwrap();
    }

    let result = result.unwrap();
    let mut vm = Vm::new(runtime.0.clone(), Arc::new(result));
    let output = vm.call(["get_query_terms"], ()).unwrap();

    let mut query = QueryBuilder::<()>::new(world);

    let mut query_names = vec![];
    for i in output.into_vec().unwrap().take().unwrap() {
        let i = i.into_string();
        let i = i.unwrap();
        let i = i.take();
        let i = i.unwrap();
        let val = i.as_str();
        query_names.push(val.to_string());
    }
    for s in &query_names {
        query.ref_by_id(*type_path_to_component_id.0.get(s.as_str()).unwrap());
    }

    let mut query = query.build();

    let mut query_iter = vec![];
    let mut guards = vec![];

    let result = rune::prepare(&mut sources)
        .with_context(&context.0)
        .with_diagnostics(&mut diagnostics.0)
        .build()
        .unwrap();

    query.iter_raw(world).for_each(|mut terms| {

        let mut v = vec![];
        for (i, s) in query_names.iter().enumerate() {
            let component_id = type_path_to_component_id.0.get(s.as_str()).unwrap();
            let (value, guard) = component_id_to_fn.0.get(component_id).unwrap()(&mut terms, i);
            v.push(value);
            guards.push(guard);
        }
        let v = OwnedTuple::try_from(v).unwrap();
        query_iter.push(v.to_value().unwrap());
    });

    let query_iter = query_iter.to_value().unwrap();

    let mut vm = Vm::new(runtime.0.clone(), Arc::new(result));
    let output = vm.call(["query"], (query_iter, )).unwrap();

    world.insert_resource(runtime);
    world.insert_resource(sources);
    world.insert_resource(diagnostics);
    world.insert_resource(context);
    world.insert_resource(component_id_to_fn);
    world.insert_resource(type_path_to_component_id);
}

pub fn every_tick(q: Query<&Stretch>) {
    for s in q.iter() {
        println!("{:#?}", s);
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
pub struct RuneSources(pub Sources);
impl Deref for RuneSources {
    type Target = Sources;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for RuneSources {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
