use bevy::app::{App, Plugin, Startup};
use bevy::ecs::component::ComponentId;
use bevy::prelude::{FetchedTerms, ResMut, Resource, World};
use bevy::ptr::PtrMut;
use bevy::reflect::TypePath;
use rune::runtime::{SharedPointerGuard, UnsafeToValue};
use std::collections::HashMap;

pub trait AddDynamicComponent {
    fn add_dynamic_component<T>(&mut self)
    where
        T: bevy::reflect::TypePath
            + rune::Any
            + bevy::prelude::Component
            + rune::runtime::TypeOf
            + rune::module::InstallWith
            + Sized
            + rune::runtime::MaybeTypeOf,
        for<'a> &'a mut T: rune::runtime::UnsafeToValue<Guard = SharedPointerGuard>;
}
impl AddDynamicComponent for App {
    fn add_dynamic_component<T>(&mut self)
    where
        T: bevy::reflect::TypePath
            + rune::Any
            + bevy::prelude::Component
            + rune::runtime::TypeOf
            + rune::module::InstallWith
            + Sized
            + rune::runtime::MaybeTypeOf,
        for<'a> &'a mut T: rune::runtime::UnsafeToValue<Guard = SharedPointerGuard>,
    {
        self.add_systems(Startup, |world: &mut World| {
            let component_id = world.init_component::<T>();

            if let Some(mut rune_module) = world.get_resource_mut::<RuneModule>() {
                rune_module.ty::<T>().unwrap();
                /*rune_module
                    .associated_function("term_id", |value: &T| T::type_path().to_string())
                    .unwrap();*/
                rune_module.function_meta(|| {
                    Ok(rune::__private::FunctionMetaData { kind: rune::__private::FunctionMetaKind::function("term_id", || {
                        T::type_path()
                    })?.build_associated::<T>()?, name: "term_id", docs: &[][..], arguments: &[][..] })
                }).unwrap();
            } else {
                panic!("make sure to add the RunePlugin plugin");
            }

            if let Some(mut component_id_to_fn) = world.get_resource_mut::<ComponentIdToFn>() {
                component_id_to_fn.0.insert(
                    component_id,
                    Box::new(
                        |terms: &mut FetchedTerms,
                         index: usize|
                         -> (rune::Value, rune::runtime::SharedPointerGuard) {
                            unsafe {
                                terms
                                    .fetch::<&mut T>(index)
                                    .as_mut()
                                    .unsafe_to_value()
                                    .unwrap()
                            }
                        },
                    ),
                );
            } else {
                panic!("make sure to add the RunePlugin plugin");
            }

            if let Some(mut type_path_to_component_id) =
                world.get_resource_mut::<TypePathToComponentId>()
            {
                type_path_to_component_id
                    .0
                    .insert(T::type_path(), component_id);
            } else {
                panic!("make sure to add the RunePlugin plugin")
            }
        });
    }
}
#[derive(rune::Any)]
struct Temp {

}
impl Temp {
    #[rune::function(path = Self::new)]
    fn hi() {
        
    }
}/*
pub(crate) fn hi() -> rune::alloc::Result<rune::__private::FunctionMetaData> { 
    Ok(rune::__private::FunctionMetaData { kind: rune::__private::FunctionMetaKind::function("new", Self::__rune_fn__hi)?.build_associated::<Self>()?, name: "hi", docs: &[][..], arguments: &[][..] }) 
}*/

#[derive(Default, Resource)]
pub struct ComponentIdToFn(
    pub(crate) HashMap<
        ComponentId,
        Box<
            (dyn Fn(&mut FetchedTerms, usize) -> (rune::Value, rune::runtime::SharedPointerGuard)
                 + Sync
                 + Send),
        >,
    >,
);
#[derive(Default, Resource)]
pub struct TypePathToComponentId(pub(crate) HashMap<&'static str, ComponentId>);

pub struct RunePlugin;
impl Plugin for RunePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RuneModule(rune::Module::new()));
        app.insert_resource(TypePathToComponentId::default());
        app.insert_resource(ComponentIdToFn::default());
    }
}
#[derive(Resource)]
pub struct RuneModule(pub rune::Module);
impl std::ops::Deref for RuneModule {
    type Target = rune::Module;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for RuneModule {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
