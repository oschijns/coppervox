use crate::{
    base::{CompType, INIT, Value},
    runner::Operand,
};
use std::cmp::{max, min};

impl From<CompType> for Operand {
    /// Get the appropriate function for computing a new value for the given operation
    #[rustfmt::skip]
    fn from(comp_type: CompType) -> Self {
        match comp_type {
            CompType::Const(value) => Self::Const(value),
            CompType::Input { input, value } => Self::Input { input: input.0, value },
            CompType::Or   => Self::Gate(Box::new(op_or  )),
            CompType::And  => Self::Gate(Box::new(op_and )),
            CompType::Nor  => Self::Gate(Box::new(op_nor )),
            CompType::Nand => Self::Gate(Box::new(op_nand)),
            CompType::Add  => Self::Gate(Box::new(op_add )),
            CompType::Mul  => Self::Gate(Box::new(op_mul )),
            CompType::Min  => Self::Gate(Box::new(op_min )),
            CompType::Max  => Self::Gate(Box::new(op_max )),
        }
    }
}

/// no operation
pub fn op_nope(_values: &[Value]) -> Value {
    INIT
}

/// OR operation
pub fn op_or(values: &[Value]) -> Value {
    let mut result = INIT;
    for value in values {
        result |= value;
    }
    result
}

/// AND operation
pub fn op_and(values: &[Value]) -> Value {
    let mut any = false;
    let mut result = Value::MAX;
    for value in values {
        result &= value;
        any = true;
    }
    if any { result } else { INIT }
}

/// NOR operation
pub fn op_nor(values: &[Value]) -> Value {
    !op_or(values)
}

/// NAND operation
pub fn op_nand(values: &[Value]) -> Value {
    !op_and(values)
}

/// ADD operation
pub fn op_add(values: &[Value]) -> Value {
    let mut result = INIT;
    for value in values {
        result += value;
    }
    result
}

/// MUL operation
pub fn op_mul(values: &[Value]) -> Value {
    let mut any = false;
    let mut result = 1;
    for value in values {
        result *= value;
        any = true;
    }
    if any { result } else { INIT }
}

/// MIN operation
pub fn op_min(values: &[Value]) -> Value {
    let mut any = false;
    let mut result = Value::MAX;
    for value in values {
        result = min(result, *value);
        any = true;
    }
    if any { result } else { INIT }
}

/// MAX operation
pub fn op_max(values: &[Value]) -> Value {
    let mut result = INIT;
    for value in values {
        result = max(result, *value);
    }
    result
}
