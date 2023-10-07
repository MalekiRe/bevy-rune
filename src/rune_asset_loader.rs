use crate::help::RuneModule;
use crate::magic::{Query, QueryType};
use bevy::asset::anyhow::Error;
use bevy::asset::io::{Reader, Writer};
use bevy::asset::{
    Asset, AssetLoader, AsyncReadExt, LoadContext, UntypedAssetId, VisitAssetDependencies,
};
use bevy::log::error;
use bevy::prelude::{FromWorld, PostStartup, PostUpdate, PreStartup, PreUpdate, Startup, Update, World};
use bevy::reflect::TypePath;
use bevy::utils::BoxedFuture;
use rune::macros::{quote, MacroContext, Quote, TokenStream};
use rune::parse::Parser;
use rune::termcolor::{ColorChoice, NoColor};
use rune::{ast, Context, Diagnostics, Module, Source, Sources, Vm};
use std::ops::Deref;
use std::str::FromStr;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use anyhow::bail;
use bevy::asset::meta::AssetMeta;
use bevy::asset::processor::{Process, ProcessContext, ProcessError};

pub struct RuneAssetLoader {
    module: RuneModule,
}

#[derive(TypePath)]
pub struct RuneVm {
    pub vm: Vm,
    pub query_systems: Vec<QuerySystem>,
}

pub struct QuerySystem {
    pub system_fn_name: String,
    pub term_fn_name: String,
    pub schedule_type: ScheduleTypes,
}

impl VisitAssetDependencies for RuneVm {
    fn visit_dependencies(&self, visit: &mut impl FnMut(UntypedAssetId)) {}
}

impl Asset for RuneVm {}

unsafe impl Sync for RuneVm {}
unsafe impl Send for RuneVm {}

impl FromWorld for RuneAssetLoader {
    fn from_world(world: &mut World) -> Self {
        let module = RuneModule::new();
        world.insert_resource(module.clone());
        Self { module }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ScheduleTypes {
    PreStartup(PreStartup),
    Startup(Startup),
    PostStartup(PostStartup),
    PreUpdate(PreUpdate),
    Update(Update),
    PostUpdate(PostUpdate),
}

impl FromStr for ScheduleTypes {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "PreStartup" => Self::PreStartup(PreStartup),
            "Startup" => Self::Startup(Startup),
            "PostStartup" => Self::PostStartup(PostStartup),
            "PreUpdate" => Self::PreUpdate(PreUpdate),
            "Update" => Self::Update(Update),
            "PostUpdate" => Self::PostUpdate(PostUpdate),
            _ => bail!("incorrect schedule type: {}", s),
        })
    }
}


impl AssetLoader for RuneAssetLoader {
    type Asset = RuneVm;
    type Settings = ();

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Error>> {
        let mut diagnostics = Diagnostics::new();
        Box::pin(async move {
            let mut context = Context::with_default_modules()?;
            let mut macro_module = Module::new();
            let (query_system_tx, query_system_rx) = channel();
            macro_module.attribute_macro(
                ["system"],
                move |cx: &mut MacroContext<'_, '_, '_>,
                 stream1: &TokenStream,
                 stream2: &TokenStream| {
                    let mut parser2 = Parser::from_token_stream(stream1, cx.input_span());

                    parser2.parse::<ast::OpenParen>()?;

                    let schedule_name = parser2.parse::<ast::Ident>()?;
                    let schedule_name = cx.resolve(schedule_name)?;
                    let schedule_name: ScheduleTypes = FromStr::from_str(schedule_name).unwrap();

                    parser2.parse::<ast::Comma>()?;



                    // now we parse the actual queries, commands, and resources
                    let mut queries = vec![];
                    while !parser2.peek::<ast::CloseParen>()? {
                        let param_ident = cx.resolve(parser2.parse::<ast::Ident>()?)?;
                        parser2.parse::<ast::Colon>()?;
                        let param_type = cx.resolve(parser2.parse::<ast::Ident>()?)?;
                        let mut query = Query::new(param_ident);
                        match param_type {
                            "Query" => {
                                parser2.parse::<ast::Lt>()?;
                                while !parser2.peek::<ast::Gt>()? {
                                    parser2.parse::<ast::Amp>()?;
                                    let query_type = if parser2.peek::<ast::Mut>()? {
                                        parser2.parse::<ast::Mut>()?;
                                        QueryType::Mut(cx.resolve(parser2.parse::<ast::Ident>()?)?)
                                    } else {
                                        QueryType::Ref(cx.resolve(parser2.parse::<ast::Ident>()?)?)
                                    };
                                    query.query_types.push(query_type);
                                    if parser2.peek::<ast::Comma>()? {
                                        parser2.parse::<ast::Comma>()?;
                                    }
                                }
                                parser2.parse::<ast::Gt>()?;
                                queries.push(query);
                                if parser2.peek::<ast::Comma>()? {
                                    parser2.parse::<ast::Comma>()?;
                                }
                            },
                            otherwise => {
                                panic!("wrong: {}", otherwise);
                            }
                        }
                    }


                    parser2.parse::<ast::CloseParen>()?;

                    let mut parser = Parser::from_token_stream(stream2, cx.input_span());

                    let item_fn = parser.parse::<ast::ItemFn>()?;

                    let system_name = cx.resolve(item_fn.name)?;
                    let params = queries
                        .iter()
                        .map(|a| a.param_name.to_string())
                        .collect::<Vec<_>>();
                    let param_types = queries
                        .iter()
                        .map(|a| {
                            let mut query_types = a
                                .query_types
                                .iter()
                                .map(|a| match a {
                                    QueryType::Ref(r) => String::from("Ref") + r,
                                    QueryType::Mut(m) => String::from("Mut") + m,
                                })
                                .collect::<Vec<_>>();
                            let thing = quote_strings_comma(query_types);

                            quote!([#thing])
                        })
                        .collect::<Vec<_>>();

                    let query_system_name = string_to_quote(String::from("query_") + system_name);
                    let params = quote_strings_comma(params);
                    let param_types = quote_quotes_comma(param_types);

                    query_system_tx.clone().send(QuerySystem {
                        system_fn_name: cx.resolve(item_fn.name)?.to_string(),
                        term_fn_name: String::from("query_") + system_name,
                        schedule_type: schedule_name,
                    }).unwrap();

                    let output = quote!(
                        #item_fn
                        pub fn #query_system_name() {
                            [#param_types]
                        }
                    );
                    let stream = output.into_token_stream(cx)?;
                    error!("{}", cx.stringify(&stream).unwrap());
                    Ok(stream)
                },
            )?;

            context.install(macro_module)?;
            context.install(&*self.module.0.lock().unwrap())?;

            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let mut sources = Sources::new();
            sources.insert(Source::memory(String::from_utf8(bytes)?)?)?;
            let result = rune::prepare(&mut sources)
                .with_context(&context)
                .with_diagnostics(&mut diagnostics)
                .build();

            if diagnostics.has_error() {
                let mut output = NoColor::new(vec![]);
                diagnostics.emit(&mut output, &sources).unwrap();
                anyhow::bail!(
                    "Build Failure for rune\n{}",
                    String::from_utf8(output.into_inner())?
                );
            } else if diagnostics.has_warning() {
                let mut output = NoColor::new(vec![]);
                diagnostics.emit(&mut output, &sources).unwrap();
            }
            let result = result?;
            let vm = Vm::new(Arc::new(context.runtime()?), Arc::new(result));
            let vm = Ok(RuneVm {
                vm,
                query_systems: query_system_rx.try_iter().map(|a| a).collect()
            });
            error!("added vm");
            vm
        })
    }

    fn extensions(&self) -> &[&str] {
        &["rune", "rn"]
    }
}

pub fn string_to_quote<T: Deref<Target = str> + std::marker::Send + std::marker::Sync + 'static>(
    thing: T,
) -> Quote<'static> {
    rune::macros::quote_fn(move |macro_ctx, macro_stream| {
        rune::macros::ToTokens::to_tokens(&macro_ctx.ident(&thing)?, macro_ctx, macro_stream)?;
        Ok(())
    })
}

pub fn quote_strings_comma<
    T: Deref<Target = str> + std::marker::Send + std::marker::Sync + 'static,
>(
    things: Vec<T>,
) -> Quote<'static> {
    let quotes = things
        .into_iter()
        .map(|a| string_to_quote(a))
        .collect::<Vec<_>>();
    quote_quotes_comma(quotes)
}
pub fn quote_quotes_comma(mut things: Vec<Quote>) -> Quote {
    if things.is_empty() {
        return quote!();
    }
    let mut thing = {
        let q = things.pop().unwrap();
        quote!(#q)
    };
    while !things.is_empty() {
        let q = things.pop().unwrap();
        thing = quote!(#thing, #q)
    }
    thing
}