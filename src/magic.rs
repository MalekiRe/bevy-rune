use bevy::prelude::Transform;
use bevy::reflect::Typed;
use rune::compile::Named;
use rune::macros::{quote, MacroContext, TokenStream};
use rune::parse::Parser;
use rune::runtime::RawStr;
use rune::{ast, compile, Any, Hash};

pub enum QueryType<'a> {
    Ref(&'a str),
    Mut(&'a str),
}

pub struct Query<'a> {
    pub param_name: &'a str,
    pub query_types: Vec<QueryType<'a>>,
}
impl Query<'_> {
    pub fn new<'a>(param_name: &'a str) -> Query<'a> {
        Query {
            param_name,
            query_types: vec![],
        }
    }
}

#[rune::attribute_macro]
fn system(
    cx: &mut MacroContext<'_, '_, '_>,
    stream1: &TokenStream,
    stream2: &TokenStream,
) -> compile::Result<TokenStream> {
    let mut parser = Parser::from_token_stream(stream2, cx.input_span());

    parser.parse::<ast::Fn>()?;
    let system_name = parser.parse::<ast::Ident>()?;
    parser.parse::<ast::OpenParen>()?;
    let mut queries = vec![];
    while !parser.peek::<ast::CloseParen>()? {
        let param_name = parser.parse::<ast::Ident>().unwrap();
        parser.parse::<ast::Colon>().unwrap();
        let param_type_name = parser.parse::<ast::Ident>().unwrap();
        let param_type_name = cx.resolve(param_type_name).unwrap();
        let mut query = Query::new(param_type_name);
        match param_type_name {
            "Query" => {
                parser.parse::<ast::OpenBrace>().unwrap();
                while !parser.peek::<ast::CloseBrace>()? {
                    let query_type = if parser.peek::<ast::Mut>()? {
                        parser.parse::<ast::Mut>().unwrap();
                        QueryType::Mut(cx.resolve(parser.parse::<ast::Ident>().unwrap()).unwrap())
                    } else {
                        QueryType::Ref(cx.resolve(parser.parse::<ast::Ident>().unwrap()).unwrap())
                    };
                    query.query_types.push(query_type);
                }
                parser.parse::<ast::CloseBrace>().unwrap();
            }
            "Commands" => {}
            "Res" => {}
            "ResMut" => {}
            &_ => {
                panic!("param type name was not Query, Commands, Res or ResMut");
            }
        }
        queries.push(query);
    }
    parser.parse::<ast::CloseParen>()?;

    let mut output = quote!(0);
    Ok(output.into_token_stream(cx)?)
}

struct TransformWrapper(pub Transform);

impl Named for TransformWrapper {
    const BASE_NAME: RawStr = RawStr::from_str("Transform");
}

/*impl rune::__private::InstallWith for TransformWrapper {
    fn install_with(#[allow(unused)] module: &mut rune::__private::Module) -> core::result::Result<(), rune::compile::ContextError> {
        module.field_function(rune::runtime::Protocol::GET, "x", |s: &Self| s.0.0)?;
        module.field_function(rune::runtime::Protocol::SET, "x", |s: &mut Self, value: f32| { s.0.0 = value; })?;
        module.field_function(rune::runtime::Protocol::ADD_ASSIGN, "x", |s: &mut Self, value: f32| { s.x += value; })?;
        module.type_meta::<Self>()?.make_named_struct(&["x", "y", "test", ])?.static_docs(&[])?;
        Ok(())
    }
}*/

impl Any for TransformWrapper {
    fn type_hash() -> Hash {
        crate::Hash::new(
            unsafe { std::mem::transmute::<_, u128>(Transform::type_info().type_id()) } as u64,
        )
    }
}
impl rune::runtime::TypeOf for TransformWrapper {
    #[inline]
    fn type_parameters() -> rune::Hash {
        rune::Hash::EMPTY
    }
    #[inline]
    fn type_hash() -> rune::Hash {
        <Self as rune::Any>::type_hash()
    }
    #[inline]
    fn type_info() -> rune::runtime::TypeInfo {
        rune::runtime::TypeInfo::Any(rune::runtime::AnyTypeInfo::__private_new(
            rune::runtime::RawStr::from_str(core::any::type_name::<Self>()),
            <Self as rune::runtime::TypeOf>::type_hash(),
        ))
    }
}
impl rune::runtime::MaybeTypeOf for TransformWrapper {
    #[inline]
    fn maybe_type_of() -> Option<rune::runtime::FullTypeOf> {
        Some(<Self as rune::runtime::TypeOf>::type_of())
    }
}
impl rune::runtime::UnsafeToRef for TransformWrapper {
    type Guard = rune::runtime::RawRef;
    unsafe fn unsafe_to_ref<'a>(
        value: rune::runtime::Value,
    ) -> rune::runtime::VmResult<(&'a Self, Self::Guard)> {
        let (value, guard) = match ::rune::runtime::try_result((value.into_any_ptr())) {
            ::rune::runtime::VmResult::Ok(value) => value,
            ::rune::runtime::VmResult::Err(err) => {
                return ::rune::runtime::VmResult::Err(::rune::runtime::VmError::from(err));
            }
        };
        rune::runtime::VmResult::Ok((::core::ptr::NonNull::as_ref(&value), guard))
    }
}
impl rune::runtime::UnsafeToMut for TransformWrapper {
    type Guard = rune::runtime::RawMut;
    unsafe fn unsafe_to_mut<'a>(
        value: rune::runtime::Value,
    ) -> rune::runtime::VmResult<(&'a mut Self, Self::Guard)> {
        let (mut value, guard) = match ::rune::runtime::try_result((value.into_any_mut())) {
            ::rune::runtime::VmResult::Ok(value) => value,
            ::rune::runtime::VmResult::Err(err) => {
                return ::rune::runtime::VmResult::Err(::rune::runtime::VmError::from(err));
            }
        };
        rune::runtime::VmResult::Ok((::core::ptr::NonNull::as_mut(&mut value), guard))
    }
}
impl rune::runtime::UnsafeToValue for &TransformWrapper {
    type Guard = rune::runtime::SharedPointerGuard;
    unsafe fn unsafe_to_value(
        self,
    ) -> rune::runtime::VmResult<(rune::runtime::Value, Self::Guard)> {
        let (shared, guard) =
            match ::rune::runtime::try_result((rune::runtime::Shared::from_ref(self))) {
                ::rune::runtime::VmResult::Ok(value) => value,
                ::rune::runtime::VmResult::Err(err) => {
                    return ::rune::runtime::VmResult::Err(::rune::runtime::VmError::from(err));
                }
            };
        rune::runtime::VmResult::Ok((rune::runtime::Value::from(shared), guard))
    }
}
impl rune::runtime::UnsafeToValue for &mut TransformWrapper {
    type Guard = rune::runtime::SharedPointerGuard;
    unsafe fn unsafe_to_value(
        self,
    ) -> rune::runtime::VmResult<(rune::runtime::Value, Self::Guard)> {
        let (shared, guard) =
            match ::rune::runtime::try_result((rune::runtime::Shared::from_mut(self))) {
                ::rune::runtime::VmResult::Ok(value) => value,
                ::rune::runtime::VmResult::Err(err) => {
                    return ::rune::runtime::VmResult::Err(::rune::runtime::VmError::from(err));
                }
            };
        rune::runtime::VmResult::Ok((rune::runtime::Value::from(shared), guard))
    }
}
