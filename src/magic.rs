use bevy::prelude::Transform;
use bevy::reflect::Typed;
use rune::compile::Named;
use rune::runtime::RawStr;
use rune::{Any, Hash};

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
