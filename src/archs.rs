//! Submodule for all architectures

macro_rules! def_features {
    ($(#[$meta:meta])* $vis:vis enum $feature_enum:ident {
        $($(#[$vmeta:meta])* $feature_variant:ident $feature_name:literal $(|$alias_name:literal)*),*
        $(,)?
    }) => {
        $(#[$meta])*
        #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
        $vis enum $feature_enum {
            $($(#[$vmeta])* $feature_variant),*
        }

        impl Name for $feature_enum {
            fn name(&self) -> &'static str {
                match *self {
                    $(Self:: $feature_variant => $feature_name),*
                }
            }
        }

        impl TargetFeatureSpec for $feature_enum {
            fn feature_to_bit(&self) -> u32 {
                *self as u32
            }

            fn from_name(name: &str) -> Option<Self> {
                match name {
                    $($feature_name $(|$alias_name)* => Some(Self::$feature_variant),)*
                    _ => None
                }
            }

            fn feature_from_bit(bit: u32) -> Option<Self> {
                $(
                    #[allow(non_upper_case_globals)] 
                    const $feature_variant: u32 = <$feature_enum>::$feature_variant as u32;
                )*
                #[allow(non_upper_case_globals)] 
                match bit {
                    $($feature_variant => Some(Self::$feature_variant),)*
                    _ => None
                }
            }
        }
    };
}

#[cfg(feature = "x86")]
pub mod x86;

#[cfg(feature = "skyarch")]
pub mod skyarch;