#[derive(Copy, Clone)]
pub enum __Void {}

impl From<__Void> for ! {
    fn from(value: __Void) -> Self {
        match value {}
    }
}

#[macro_export]
macro_rules! instr_set {
    {
        $(#[$meta:meta])*
        instructions $enum_name:ident $([$($extra_properties_names:ident : $extra_properties_tys:ty $(= $extra_properties_defauit:expr)? ),* $(,)?])? {

        }
    } => {

    };
    {
        $(#[$meta:meta])*
        encoding $enum_name:ident $([$($extra_properties_names:ident : $extra_properties_tys:ty $(= $extra_properties_defauit:expr)? ),* $(,)?])? {
            $($encoding_name:ident $(($($extra_properties_value_names:ident : $extra_properties_value:expr),* $(,)?))?),*
        }
    } => {
        $(#[$meta])*
        #[derive($crate::instr::EncodingId, Copy, Clone)]
        pub enum $enum_name {
            $($encoding_name),*
        }



        const _: () = {

            struct __StaticProperties {
                $($extra_properties_names: ::core::result::Result<$extra_properties_tys, ($($crate::instr::macros::__Void ${ignore($extra_properties_defauit)})?)>),*
            }

            const __DEFAULT: __StaticProperties = __StaticProperties {
                $($extra_properties_names: ($(Ok($extra_properties_default),)? Err::<$extra_properties_tys, ()>(())).0),*
            };

            impl $enum_name {

            }
        };
    };
    {
        $(
            $(#[$meta:meta])*
            $kind:ident $enum_name:ident $([$($extra_properties_names:ident : $extra_properties_tys:ty $(= $extra_properties_defauit:expr)? ),* $(,)?])? {
                $($tt:tt)*
            }
        )*
    } => {
        $($crate::instr_set!{
            $(#[$meta])*
            $kind $enum_name $([$($extra_properties_names : $extra_properties_tys $(= $extra_properties_defauit)? ),*])? {
                $($tt)*
            }
        })*
    }
}
