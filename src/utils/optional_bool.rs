/// Enum that can have boolean variants
/// and also "undefined" variant if either "true" nor "false" are inappropriate
#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum BoolOptional {
    True = 1,
    False = 0,
    Undefined = 2,
}

impl From<Option<bool>> for BoolOptional {
    fn from(value: Option<bool>) -> Self {
        if value.is_none() {
            return Self::Undefined;
        }

        if unsafe { value.unwrap_unchecked() } {
            Self::True
        } else {
            Self::False
        }
    }
}

impl BoolOptional {
    pub fn is_true(&self) -> bool {
        *self as u8 == 1
    }

    pub fn is_false(&self) -> bool {
        *self as u8 == 0
    }

    pub fn is_undefined(&self) -> bool {
        *self as u8 == 2
    }

    pub fn is_undefined_or(&self, variant: bool) -> bool {
        match self {
            BoolOptional::True => variant == true,
            BoolOptional::False => variant == false,
            BoolOptional::Undefined => true,
        }
    }
}
