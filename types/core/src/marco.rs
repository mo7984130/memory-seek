#[macro_export]
macro_rules! id_type {
    // i64 主键 ID:序列化为字符串,反序列化只接受字符串
    ($name:ident) => {
        #[derive(
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            Copy,
            Clone,
            Debug,
            derive_more::Display,
            derive_more::From,
            derive_more::Into,
        )]
        #[display("{}", _0)]
        #[cfg_attr(feature = "orm", derive(sea_orm::DeriveValueType))]
        pub struct $name(pub i64);

        /// 序列化为字符串
        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(&self.0.to_string())
            }
        }

        /// 反序列化只接受字符串
        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                struct Visitor;

                impl<'de> serde::de::Visitor<'de> for Visitor {
                    type Value = $name;

                    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                        f.write_str(concat!("a ", stringify!($name), " as a string"))
                    }

                    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<$name, E> {
                        v.parse::<$name>().map_err(E::custom)
                    }
                }

                d.deserialize_str(Visitor)
            }
        }

        impl std::str::FromStr for $name {
            type Err = std::num::ParseIntError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                s.parse::<i64>().map($name)
            }
        }
    };

    // String 主键 ID:序列化为字符串,反序列化只接受字符串
    ($name:ident, String) => {
        #[derive(
            PartialEq,
            Eq,
            Hash,
            Clone,
            Debug,
            derive_more::Display,
            derive_more::From,
            derive_more::Into,
        )]
        #[display("{}", _0)]
        #[cfg_attr(feature = "orm", derive(sea_orm::DeriveValueType))]
        pub struct $name(pub String);

        /// 序列化为字符串
        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(&self.0)
            }
        }

        /// 反序列化只接受字符串
        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                struct Visitor;

                impl<'de> serde::de::Visitor<'de> for Visitor {
                    type Value = $name;

                    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                        f.write_str(concat!("a ", stringify!($name), " as a string"))
                    }

                    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<$name, E> {
                        Ok($name(v.to_owned()))
                    }
                }

                d.deserialize_str(Visitor)
            }
        }

        impl std::str::FromStr for $name {
            type Err = std::convert::Infallible;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok($name(s.to_owned()))
            }
        }

        #[cfg(feature = "orm")]
        impl sea_orm::TryFromU64 for $name {
            fn try_from_u64(value: u64) -> Result<Self, sea_orm::DbErr> {
                Ok(Self(value.to_string()))
            }
        }
    };
}
