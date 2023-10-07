use crate::rune_asset_loader::{RuneAssetLoader, RuneVm};
use crate::{RuneContext, RuneDiagnostics, RuneRuntime, RuneSources};
use bevy::app::{App, Plugin, PostStartup, PostUpdate, PreStartup, Startup};
use bevy::ecs::component::{ComponentDescriptor, ComponentId, StorageType};
use bevy::prelude::{error, info, AssetApp, Commands, Component, FetchedTerms, IntoSystemConfigs, QueryBuilder, Res, ResMut, Resource, Update, World, DetectChangesMut, Mut};
use bevy::ptr::{OwningPtr, PtrMut};
use bevy::reflect::TypePath;
use rune::__private::FunctionMetaKind;
use rune::alloc::prelude::TryClone;
use rune::runtime::{
    FullTypeOf, OwnedTuple, Shared, SharedPointerGuard, TypeInfo, UnsafeToValue, VmError, VmResult,
};
use rune::termcolor::{ColorChoice, StandardStream};
use rune::{Any, Context, Diagnostics, Hash, Module, Source, Sources, ToValue, Value, Vm};
use std::alloc::Layout;
use std::any::{TypeId};
use std::collections::HashMap;
use std::mem::size_of;
use std::ptr::NonNull;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use bevy::ecs::change_detection::MutUntyped;
use rune::bevy_related::UnsafeToValue2;
use crate::rune_systems::update_system;

#[derive(Any, Clone, Copy)]
pub enum QueryType {
    Ref = 0,
    Mut = 1,
}

pub trait AddDynamicComponent {
    fn add_dynamic_component<T>(&mut self)
    where
        T: rune::Any
            + bevy::prelude::Component
            + rune::runtime::TypeOf
            + rune::module::InstallWith
            + Sized
            + rune::runtime::MaybeTypeOf,
        for<'a> &'a mut T: rune::runtime::UnsafeToValue<Guard = SharedPointerGuard>,
        for<'a> &'a T: rune::runtime::UnsafeToValue<Guard = SharedPointerGuard>,
    ;
}
impl AddDynamicComponent for App {
    fn add_dynamic_component<T>(&mut self)
    where
        T: rune::Any
            + bevy::prelude::Component
            + rune::runtime::TypeOf
            + rune::module::InstallWith
            + Sized
            + rune::runtime::MaybeTypeOf,
        for<'a> &'a mut T: rune::runtime::UnsafeToValue<Guard = SharedPointerGuard>,
        for<'a> &'a T: rune::runtime::UnsafeToValue<Guard = SharedPointerGuard>,
    {
        self.add_systems(PreStartup, |world: &mut World| {
            let component_id = world.init_component::<T>();
            if let Some(mut rune_module) = world.get_resource_mut::<RuneModule>() {
                let mut rune_module = rune_module.0.lock().unwrap();
                rune_module.ty::<T>().unwrap();
                rune_module.constant((String::from("Ref")+ &*T::full_name()).as_str(), (component_id.index(), QueryType::Ref as u8)).build().unwrap();
                rune_module.constant((String::from("Mut")+ &*T::full_name()).as_str(), (component_id.index(), QueryType::Mut as u8)).build().unwrap();
            } else {
                panic!("make sure to add the RunePlugin plugin");
            }

            if let Some(mut component_id_to_fn) = world.get_resource_mut::<ComponentIdToFn>() {
                component_id_to_fn.0.insert(
                    component_id,
                    Box::new(
                        |terms: &mut FetchedTerms, index: usize, query_type: QueryType| -> (rune::Value, _) {
                            unsafe {
                                match query_type {
                                    QueryType::Ref => {
                                        let (value, guard) = terms
                                            .fetch::<&T>(index)
                                            .unsafe_to_value()
                                            .unwrap();
                                        (value, Guards::Shared(guard))
                                    }
                                    QueryType::Mut => {
                                        let mut thing = terms.fetch::<&mut T>(index);
                                        let (value, guard) = UnsafeToValue2::unsafe_to_value(thing).unwrap();
                                        (value, Guards::Shared(guard))
                                    }
                                }
                            }
                        },
                    ),
                );
            } else {
                panic!("make sure to add the RunePlugin plugin");
            }
        });
    }
}

#[derive(Default, Resource)]
pub struct ComponentIdToFn(
    pub(crate)  HashMap<
        ComponentId,
        Box<(dyn Fn(&mut FetchedTerms, usize, QueryType) -> (rune::Value, Guards) + Sync + Send)>,
    >,
);

pub struct RunePlugin;
impl Plugin for RunePlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<RuneAssetLoader>()
            .init_asset::<RuneVm>();
        app.insert_resource(ComponentIdToFn::default());
        app.add_systems(Update, update_system);
        /*app.add_systems(PreStartup, setup_sources);
        app.add_systems(Startup, setup_dynamic_queries);
        app.add_systems(Update, dynamic_queries);
        app.add_systems(PostStartup, post_setup_add_dynamic_components);*/
    }
}
#[derive(Resource, Default, Clone)]
pub struct RuneModule(pub Arc<Mutex<rune::Module>>);
impl RuneModule {
    pub fn new() -> Self {
        Self::default()
    }
}
pub enum Guards {
    Nil,
    Shared(SharedPointerGuard),
}

fn post_setup_add_dynamic_components(world: &mut World) {
    let mut component_id_to_fn = world.remove_resource::<ComponentIdToFn>().unwrap();
    let mut sources = world.remove_resource::<RuneSources>().unwrap();
    let mut s = Sources::new();
    //s.insert(Source::from_path("./src/query.rune").unwrap()).unwrap();
    s.insert(Source::from_path("./src/query.rune").unwrap())
        .unwrap();
    sources.0.push(s);
    let mut context = world.remove_resource::<RuneContext>().unwrap();
    let mut diagnostics = world.remove_resource::<RuneDiagnostics>().unwrap();
    let mut runtime = world.remove_resource::<RuneRuntime>().unwrap();
    let mut things_to_add = vec![];
    let mut func = || {
        for source in &mut sources.0 {
            let result = rune::prepare(source)
                .with_context(&context.0)
                .with_diagnostics(&mut diagnostics.0)
                .build();

            if !diagnostics.0.is_empty() {
                let mut writer = StandardStream::stderr(ColorChoice::Always);
                diagnostics.0.emit(&mut writer, &source).unwrap();
                error!("diagnostics failed");
                return;
            }

            let result = result.unwrap();
            let mut vm = Vm::new(runtime.0.clone(), Arc::new(result));
            let vec = match vm.call(["dynamic_components"], ()) {
                Ok(output) => match output.into_vec() {
                    VmResult::Ok(vec) => vec,
                    VmResult::Err(err) => return error!("AAA {}", err),
                },
                Err(err) => return error!("AAA {}", err),
            };
            let mut module = Module::new();
            for dynamic_component in vec.take().unwrap() {
                let name = dynamic_component.type_info().unwrap();
                let (name, hash, item) = match name {
                    TypeInfo::Typed(a) => {
                        (a.item.to_string(), { a.hash }, a.item.try_clone().unwrap())
                    }
                    TypeInfo::Variant(a) => {
                        (a.item.to_string(), { a.hash }, a.item.try_clone().unwrap())
                    }
                    a => return error!("wrong type of type trying to register, {:#?}", a),
                };
                let component_id = world.init_component_with_descriptor(unsafe {
                    ComponentDescriptor::new_with_layout(
                        name.clone(),
                        StorageType::Table,
                        Layout::array::<u64>(size_of::<Value>()).unwrap(),
                        None,
                    )
                });
                things_to_add.push((dynamic_component.clone(), component_id));
                /*module
                    .constant("TERM", component_id.index())
                    .build()
                    .unwrap();
                component_id_to_fn.0.insert(
                    component_id,
                    Box::new(|terms, index| unsafe {
                        (
                            terms.fetch::<&mut ValueWrapper>(index).as_mut().0.clone(),
                            Guards::Nil,
                        )
                    }),
                );*/
                // module
                //     .dynamic_ty(
                //         hash,
                //         Hash::EMPTY,
                //         dynamic_component.type_info().unwrap(),
                //         item,
                //         |module| Ok(()),
                //     )
                //     .unwrap();
                // module
                //     .function2("term_id", move || component_id.index())
                //     .unwrap()
                //     .build_associated_with(
                //         FullTypeOf::new(hash),
                //         dynamic_component.type_info().unwrap(),
                //     )
                //     .unwrap();
                context.0.install(&module).unwrap();
                runtime = RuneRuntime(Arc::new(context.0.runtime().unwrap()));
                error!("Bevy succeeded in adding the thing maybe?");
                source
                    .insert(Source::from_path("./src/dynamic_stuff.rn").unwrap())
                    .unwrap();
                //module.ty();
            }
        }
    };
    func();
    let mut entity = world.spawn_empty();
    unsafe {
        let t = things_to_add.remove(0);
        OwningPtr::make(t.0, |a| {
            entity.insert_by_id(t.1, a);
        });
    }
    /*let mut entity = world.spawn_empty();
    unsafe {
        let data = std::alloc::alloc_zeroed(Layout::array::<u64>(size_of::<Value>()).unwrap());
        let mut val = things_to_add.first().unwrap().0.clone();
        data.copy_from((&mut val) as *mut Value as *mut _ as *mut u8, 1);
        let a = NonNull::new_unchecked(data);
        entity.insert_by_id(things_to_add.first().unwrap().1, OwningPtr::new(a));
    }*/
    world.insert_resource(runtime);
    world.insert_resource(sources);
    world.insert_resource(diagnostics);
    world.insert_resource(context);
    world.insert_resource(component_id_to_fn);
}

fn setup_sources(mut commands: Commands) {
    let diagnostics = Diagnostics::new();
    commands.insert_resource(RuneSources(vec![]));
    commands.insert_resource(RuneDiagnostics(diagnostics));
}

#[derive(Component)]
pub struct ValueWrapper(pub Value);

unsafe impl Send for ValueWrapper {}
unsafe impl Sync for ValueWrapper {}

fn setup_dynamic_queries(mut commands: Commands, rune_module: Res<RuneModule>) {
    let mut context = Context::with_default_modules().unwrap();
    context.install(&*rune_module.0.lock().unwrap()).unwrap();
    let runtime = context.runtime().unwrap();
    commands.insert_resource(RuneContext(context));
    commands.insert_resource(RuneRuntime(Arc::new(runtime)));
}

/*pub fn dynamic_queries(world: &mut World) {
    let component_id_to_fn = world.remove_resource::<ComponentIdToFn>().unwrap();
    let mut sources = world.remove_resource::<RuneSources>().unwrap();
    let context = world.remove_resource::<RuneContext>().unwrap();
    let mut diagnostics = world.remove_resource::<RuneDiagnostics>().unwrap();
    let runtime = world.remove_resource::<RuneRuntime>().unwrap();
    let mut func = || {
        for sources in sources.0.iter_mut() {
            let result = rune::prepare(sources)
                .with_context(&context.0)
                .with_diagnostics(&mut diagnostics.0)
                .build();

            if !diagnostics.0.is_empty() {
                let mut writer = StandardStream::stderr(ColorChoice::Always);
                diagnostics.0.emit(&mut writer, &sources).unwrap();
                return;
            }

            let result = result.unwrap();
            let mut vm = Vm::new(runtime.0.clone(), Arc::new(result));
            let output = match vm.call(["get_query_terms"], ()) {
                Ok(output) => output,
                Err(err) => return error!("{}", err),
            };

            let mut query = QueryBuilder::<()>::new(world);

            let mut query_component_ids = vec![];
            for i in output.into_vec().unwrap().take().unwrap() {
                match i.into_usize() {
                    VmResult::Ok(i) => {
                        query_component_ids.push(ComponentId::new(i));
                    }
                    VmResult::Err(err) => return error!("{}", err),
                }
            }
            for component_id in &query_component_ids {
                query.ref_by_id(*component_id);
            }

            let mut query = query.build();

            let mut query_iter = vec![];
            let mut guards = vec![];

            let result = rune::prepare(sources)
                .with_context(&context.0)
                .with_diagnostics(&mut diagnostics.0)
                .build();

            if !diagnostics.0.is_empty() {
                let mut writer = StandardStream::stderr(ColorChoice::Always);
                diagnostics.0.emit(&mut writer, &sources).unwrap();
                return;
            }

            let result = result.unwrap();

            query.iter_raw(world).for_each(|mut terms| {
                /*let mut v = vec![];
                for (i, component_id) in query_component_ids.iter().enumerate() {
                    let (value, guard) = (match component_id_to_fn.0.get(component_id) {
                        None => return error!("missing component ID in map of component_id_to_fn"),
                        Some(res) => res,
                    }(&mut terms, i));
                    v.push(value);
                    guards.push(guard);
                }
                let v = OwnedTuple::try_from(v).unwrap();
                query_iter.push(v.to_value().unwrap());*/
            });

            let query_iter = query_iter.to_value().unwrap();

            let mut vm = Vm::new(runtime.0.clone(), Arc::new(result));
            match vm.call(["query"], (query_iter,)) {
                Err(err) => error!("error running query: {}", err),
                _ => {}
            };
        }
    };
    func();
    world.insert_resource(runtime);
    world.insert_resource(sources);
    world.insert_resource(diagnostics);
    world.insert_resource(context);
    world.insert_resource(component_id_to_fn);
}*/

pub fn all_modules(#[allow(unused)] stdio: bool) -> Result<Context, rune::ContextError> {
    let mut this = Context::new();
    // This must go first, because it includes types which are used in other modules.
    this.install(rune::modules::core::module()?)?;

    this.install(rune::modules::num::module()?)?;
    //this.install(rune::modules::any::module()?)?;
    this.install(rune::modules::bytes::module()?)?;
    this.install(rune::modules::char::module()?)?;
    this.install(rune::modules::hash::module()?)?;
    this.install(rune::modules::cmp::module()?)?;
    this.install(rune::modules::collections::module()?)?;
    this.install(rune::modules::f64::module()?)?;
    this.install(rune::modules::tuple::module()?)?;
    this.install(rune::modules::fmt::module()?)?;
    this.install(rune::modules::future::module()?)?;
    this.install(rune::modules::i64::module()?)?;
    #[cfg(feature = "std")]
    this.install(crate::modules::io::module(stdio)?)?;
    this.install(rune::modules::iter::module()?)?;
    this.install(rune::modules::macros::module()?)?;
    this.install(rune::modules::mem::module()?)?;
    this.install(rune::modules::object::module()?)?;
    this.install(rune::modules::ops::module()?)?;
    this.install(rune::modules::option::module()?)?;
    this.install(rune::modules::result::module()?)?;
    this.install(rune::modules::stream::module()?)?;
    this.install(rune::modules::string::module()?)?;
    this.install(rune::modules::test::module()?)?;
    this.install(rune::modules::vec::module()?)?;
    /*this.has_default_modules = true;*/
    Ok(this)
}
