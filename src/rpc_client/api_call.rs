use enum_primitive_derive::Primitive;
use num_traits::FromPrimitive;

macro_rules! enum_number {
    ($name:ident { $($variant:ident = $value:expr, )* }) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq, Primitive)]
        pub enum $name {
            $($variant = $value,)*
        }

        impl ::serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                // Serialize the enum as a u64.
                serializer.serialize_u64(*self as u64)
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                struct Visitor;

                impl<'de> ::serde::de::Visitor<'de> for Visitor {
                    type Value = $name;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str("positive integer")
                    }

                    fn visit_u64<E>(self, value: u64) -> Result<$name, E>
                    where
                        E: ::serde::de::Error,
                    {
                        // Rust does not come with a simple way of converting a
                        // number to an enum, so use a big `match`.
                        match value {
                            $( $value => Ok($name::$variant), )*
                            _ => Err(E::custom(
                                format!("unknown {} value: {}",
                                stringify!($name), value))),
                        }
                    }
                }

                // Deserialize the enum from a u64.
                deserializer.deserialize_u64(Visitor)
            }
        }
    }
}

enum_number!(ApiCall {
    Nil = 0,
    EthBlockNumber = 1, // eth_blockNumber
    EthGetBlockByNumber = 2, // eth_getBlockByNumber
});

impl std::fmt::Display for ApiCall {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Call: {}, representing: {}", self.to_str(), self.method_info())
    }

}

impl ApiCall {

    pub fn method_info(&self) -> String {
        match self {
            ApiCall::EthBlockNumber => "eth_blockNumber".to_string(),
            ApiCall::EthGetBlockByNumber => "eth_getBlockByNumber".to_string(),
            _=> panic!("Api Call Does not exist")
        }
    }

    pub fn to_str(&self) -> String {
        match self {
            ApiCall::EthBlockNumber => "EthBlockNumber".to_owned(),
            ApiCall::EthGetBlockByNumber => "EthGetBlockByNumber".to_owned(),
            _=> panic!("Api Call does not exist!")
        }
    }

    pub fn from_id_and<F,T>(id: usize, fun: F) -> T
        where
            F: FnOnce(ApiCall) -> T
    {
        match Self::from_usize(id).unwrap() { // TODO remove unwrap #p3
            c @ ApiCall::EthBlockNumber => fun(c),
            c @ ApiCall::EthGetBlockByNumber => fun(c),
            _ => panic!("Api Call does not exist!")
        }
        
    }
}

