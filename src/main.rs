mod help;
mod magic;
mod rune_asset_loader;
mod rune_systems;

use crate::help::{AddDynamicComponent, ComponentIdToFn, RuneModule, RunePlugin};
use crate::rune_asset_loader::{RuneAssetLoader, RuneVm};
use bevy::app::{App, PostStartup, Update};
use bevy::ecs::component::ComponentId;
use bevy::log::error;
use bevy::prelude::{shape, warn, AssetApp, AssetServer, Assets, Camera2dBundle, Color, ColorMaterial, Commands, Component, FetchedTerms, Handle, Mesh, Mut, Query, QueryBuilder, Reflect, Res, ResMut, Resource, Startup, Transform, Vec3, World, AssetEvent, Changed, DetectChangesMut, PluginGroup, AssetPlugin};
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
use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};
use std::sync::mpsc::Receiver;
use bevy::ecs::change_detection::MutUntyped;

fn main() {
    println!("Hello, world!");
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(AssetPlugin::Unprocessed {
        source: Default::default(),
        watch_for_changes: true,
    }));
    app.add_systems(Startup, |asset_server: Res<AssetServer>, mut commands: Commands| {
        let handle: Handle<RuneVm> = asset_server.load("foo.rune");
        commands.spawn(handle);
    });
    /*app.add_systems(Update, |mut events: bevy::prelude::EventReader<AssetEvent<RuneVm>>| {
        for ev in events.read() {
            warn!("{:#?}", ev);
        }
    });*/
    app.add_plugins(RunePlugin);
    app.add_dynamic_component::<Stretch>();
    app.add_systems(Update, every_tick);
    app.add_systems(Startup, startup);
    app.run();
}
#[derive(Component, TypePath, Debug)]
pub struct Stretch {
    //#[rune(get, set, add_assign, copy)]
    x: f32,
    //#[rune(get, set, add_assign, copy)]
    y: f32,
    test: Option<Wrapper<'static>>,
}


impl TriggerChangeDetection for Stretch {
    fn trigger_change_detection(&mut self) {
        self.test.as_mut().unwrap().0.set_changed();
    }
    fn add_thing(&mut self, mut thing: Wrapper) {
        let thing = unsafe {
            std::mem::transmute(thing)
        };
        self.test.replace(thing);
    }
}

pub trait TriggerChangeDetection {
    fn trigger_change_detection(&mut self);
    fn add_thing(&mut self, thing: Wrapper);
}

#[automatically_derived]
impl rune::__private::InstallWith for Stretch {
    fn install_with(#[allow(unused)] module: &mut rune::__private::Module) -> core::result::Result<(), rune::compile::ContextError> {
        module.field_function(rune::runtime::Protocol::GET, "x", |s: &Self| s.x)?;
        module.field_function(rune::runtime::Protocol::SET, "x", |s: &mut Self, value: f32| { s.x = value; })?;
        module.field_function(rune::runtime::Protocol::ADD_ASSIGN, "x", |s: &mut Self, value: f32| { s.x += value; })?;
        module.field_function(rune::runtime::Protocol::GET, "y", |s: &Self| s.y)?;
        module.field_function(rune::runtime::Protocol::SET, "y", |s: &mut Self, value: f32| { s.y = value; })?;
        module.field_function(rune::runtime::Protocol::ADD_ASSIGN, "y", |s: &mut Self, value: f32| { s.y += value; })?;
        module.type_meta::<Self>()?.make_named_struct(&["x", "y", "test", ])?.static_docs(&[])?;
        Ok(())
    }
}
#[automatically_derived]
impl rune::compile::Named for Stretch {
    const BASE_NAME: rune::runtime::RawStr = rune::runtime::RawStr::from_str("Stretch");
}
#[automatically_derived]
impl rune::runtime::TypeOf for Stretch {
    #[inline]
    fn type_hash() -> rune::Hash { <Self as rune::Any>::type_hash() }
    #[inline]
    fn type_parameters() -> rune::Hash { rune::Hash::EMPTY }
    #[inline]
    fn type_info() -> rune::runtime::TypeInfo { rune::runtime::TypeInfo::Any(rune::runtime::AnyTypeInfo::__private_new(rune::runtime::RawStr::from_str(core::any::type_name::<Self>()), <Self as rune::runtime::TypeOf>::type_hash())) }
}
#[automatically_derived]
impl rune::runtime::MaybeTypeOf for Stretch {
    #[inline]
    fn maybe_type_of() -> Option<rune::runtime::FullTypeOf> { Some(<Self as rune::runtime::TypeOf>::type_of()) }
}
#[automatically_derived]
impl rune::Any for Stretch {
    fn type_hash() -> rune::Hash { rune::Hash::new(16272786712427842834u64) }
}
#[automatically_derived]
impl rune::runtime::UnsafeToRef for Stretch {
    type Guard = rune::runtime::RawRef;
    unsafe fn unsafe_to_ref<'a>(value: rune::runtime::Value) -> rune::runtime::VmResult<(&'a Self, Self::Guard)> {
        let (value, guard) = match ::rune::runtime::try_result((value.into_any_ptr())) {
            ::rune::runtime::VmResult::Ok(value) => value,
            ::rune::runtime::VmResult::Err(err) => {
                return ::rune::runtime::VmResult::Err(::rune::runtime::VmError::from(err));
            }
        };
        rune::runtime::VmResult::Ok((::core::ptr::NonNull::as_ref(&value), guard))
    }
}
#[automatically_derived]
impl rune::runtime::UnsafeToMut for Stretch {
    type Guard = rune::runtime::RawMut;
    unsafe fn unsafe_to_mut<'a>(value: rune::runtime::Value) -> rune::runtime::VmResult<(&'a mut Self, Self::Guard)> {
        let (mut value, guard) = match ::rune::runtime::try_result((value.into_any_mut())) {
            ::rune::runtime::VmResult::Ok(value) => value,
            ::rune::runtime::VmResult::Err(err) => {
                return ::rune::runtime::VmResult::Err(::rune::runtime::VmError::from(err));
            }
        };
        let val: &mut Self = ::core::ptr::NonNull::as_mut(&mut value);
        val.trigger_change_detection();
        rune::runtime::VmResult::Ok((val, guard))
    }
}
#[automatically_derived]
impl rune::runtime::UnsafeToValue for &Stretch {
    type Guard = rune::runtime::SharedPointerGuard;
    unsafe fn unsafe_to_value(self) -> rune::runtime::VmResult<(rune::runtime::Value, Self::Guard)> {
        let (shared, guard) = match ::rune::runtime::try_result((rune::runtime::Shared::from_ref(self))) {
            ::rune::runtime::VmResult::Ok(value) => value,
            ::rune::runtime::VmResult::Err(err) => {
                return ::rune::runtime::VmResult::Err(::rune::runtime::VmError::from(err));
            }
        };
        rune::runtime::VmResult::Ok((rune::runtime::Value::from(shared), guard))
    }
}
#[automatically_derived]
impl rune::runtime::UnsafeToValue for &mut Stretch {
    type Guard = rune::runtime::SharedPointerGuard;
    unsafe fn unsafe_to_value(self) -> rune::runtime::VmResult<(rune::runtime::Value, Self::Guard)> {
        let (shared, guard) = match ::rune::runtime::try_result((rune::runtime::Shared::from_mut(self))) {
            ::rune::runtime::VmResult::Ok(value) => value,
            ::rune::runtime::VmResult::Err(err) => {
                return ::rune::runtime::VmResult::Err(::rune::runtime::VmError::from(err));
            }
        };
        rune::runtime::VmResult::Ok((rune::runtime::Value::from(shared), guard))
    }
}

pub struct Wrapper<'a>(MutUntyped<'a>);
impl Debug for Wrapper<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
unsafe impl Send for Wrapper<'_>{}
unsafe impl Sync for Wrapper<'_>{}

pub fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
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
            test: None,
        },
    ));
}

pub fn every_tick(mut q: Query<(&Stretch, &mut Transform), Changed<Stretch>>) {
    for (s, mut t) in q.iter_mut() {
        println!("stretch changed: {:#?}", s);
        t.scale.x = s.x;
        t.scale.y = s.y;
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
