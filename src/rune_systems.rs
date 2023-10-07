use bevy::ecs::component::ComponentId;
use bevy::prelude::{Assets, error, FetchedTerms, info, QueryBuilder, World};
use rune::runtime::{OwnedTuple, VmError};
use rune::{ToValue, Value};
use crate::help::{ComponentIdToFn, Guards, QueryType};
use crate::rune_asset_loader::{RuneVm, ScheduleTypes};

pub fn update_system(world: &mut World) {
    let mut component_id_to_fn = world.remove_resource::<ComponentIdToFn>().unwrap();
    let mut vms = match world.remove_resource::<Assets<RuneVm>>() {
        None => return,
        Some(vms) => vms,
    };
    let mut func = || {
        for (_, vm) in vms.iter_mut() {
            let RuneVm{ vm, query_systems } = vm;
            for query_system in query_systems {
                match query_system.schedule_type {
                    ScheduleTypes::Update(_) => {
                    }
                    _ => continue,
                };
                let system_terms = match vm.call([query_system.term_fn_name.as_str()], ()) {
                    Ok(output) => output,
                    Err(err) => return error!("query_term_fn failed: {} with {}", query_system.term_fn_name, err),
                };

                let mut all_system_params = vec![];
                for system_param in system_terms.into_vec().unwrap().take().unwrap() {
                    //for now we just assume it's a query, later we figure it out for other things
                    let mut all_query_terms = vec![];
                    for query_terms in system_param.into_vec().unwrap().take().unwrap() {
                        let query_terms = query_terms.into_tuple().unwrap().take().unwrap();
                        let component_id = query_terms.get(0).unwrap().as_usize().unwrap();
                        let query_type = query_terms.get(1).unwrap().as_byte().unwrap();
                        all_query_terms.push((ComponentId::new(component_id),
                                              match query_type {
                                                  0 => QueryType::Ref,
                                                  1 => QueryType::Mut,
                                                  _ => panic!("invalid not ref or mut should be impossible"),
                                              }
                        ));
                    }
                    all_system_params.push(all_query_terms);
                }

                for system_param in all_system_params {
                    let mut query_builder = QueryBuilder::<()>::new(world);
                    for query_term in &system_param {
                        let (component_id, query_type) = query_term;
                        match query_type {
                            QueryType::Ref => query_builder.ref_by_id(*component_id),
                            QueryType::Mut => query_builder.mut_by_id(*component_id),
                        };
                    }
                    let mut query = query_builder.build();

                    let mut full_query_values = vec![];
                    let mut guards = vec![];
                    query.iter_raw(world).for_each(|mut terms| {
                        let mut values = vec![];
                        for (i,( component_id, query_type)) in system_param.iter().enumerate() {
                            let (value, guard) = (match component_id_to_fn.0.get(&component_id) {
                                None => return error!("missing component ID in map of component_id_to_fn"),
                                Some(res) => res,
                            })(&mut terms, i, *query_type);
                            values.push(value);
                            guards.push(guard);
                        }
                        let values = OwnedTuple::try_from(values).unwrap();
                        full_query_values.push(values.to_value().unwrap());
                    });
                    let full_query_values = full_query_values.to_value().unwrap();
                    match vm.call([query_system.system_fn_name.as_str()], (full_query_values, )) {
                        Err(err) => error!("error running system: {}, {:?}", query_system.system_fn_name, err),
                        _ => {}
                    }
                }

            }
        }
    };
    func();
    world.insert_resource(component_id_to_fn);
    world.insert_resource(vms);
}