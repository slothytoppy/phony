macro_rules! keywords {
    ($($variant:ident, $amount:ident = $arg_amount:literal),* $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[repr(u8)]
        #[rustfmt::skip]
        pub enum KeyWord {
            $($variant),*
        }

        impl KeyWord {
            pub fn increment_amount(&self) -> u16 {
                match self {
                    $(KeyWord::$variant => $arg_amount,)*
                }
            }
        }

        impl FromStr for KeyWord {
            type Err = ParseError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                match value {
                    "mov" => Ok(KeyWord::Mov),
                    "add" => Ok(KeyWord::Add),
                    "pop" => Ok(KeyWord::Pop),
                    "ret" => Ok(KeyWord::Ret),
                    "halt" => Ok(KeyWord::Halt),
                    "jump" => Ok(KeyWord::Jump),
                    "load" => Ok(KeyWord::Load),
                    "push" => Ok(KeyWord::Push),
                    "call" => Ok(KeyWord::Call),

                    _ => Err(Self::Err::InvalidKeyWord(value.to_string()))
                }
            }
        }

        impl std::fmt::Display for KeyWord {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                $(Self::$variant { .. } => f.write_str(stringify!($variant))?,)*
            }

            write!(f, ": {self:?}")
            }
        }
    }
}
