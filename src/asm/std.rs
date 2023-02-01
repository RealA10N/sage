//! # Standard Assembly Variant
//!
//! This variant of the assembly language is intended to be used
//! with the standard variant of the virtual machine. It is very
//! portable, but probably not supported on older systems or
//! hardware implementations.
//!
//! [***Click here to view opcodes!***](./enum.StandardOp.html)
use super::{
    location::FP_STACK, AssemblyProgram, CoreOp, CoreProgram, Env, Error, Location, F, FP, SP,
};
use crate::vm::{self, VirtualMachineProgram};
use core::fmt;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct StandardProgram(pub Vec<StandardOp>);

impl StandardProgram {
    pub fn assemble(&self, allowed_recursion_depth: usize) -> Result<vm::StandardProgram, Error> {
        let mut result = vm::StandardProgram(vec![]);
        let mut env = Env::default();
        // Create the stack of frame pointers starting directly after the last register
        F.copy_address_to(&FP_STACK, &mut result);
        // Copy the address just after the allocated space to the stack pointer.
        FP_STACK
            .deref()
            .offset(allowed_recursion_depth as i64)
            .copy_address_to(&SP, &mut result);

        SP.copy_to(&FP, &mut result);
        for (i, op) in self.0.iter().enumerate() {
            op.assemble(i, &mut env, &mut result)?
        }

        if let Ok((unmatched, last_instruction)) = env.pop_matching(self.0.len()) {
            return Err(Error::Unmatched(unmatched, last_instruction));
        }

        Ok(result.flatten())
    }
}

impl fmt::Display for StandardProgram {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut indent = 0;
        let mut comment_count = 0;
        for (i, op) in self.0.iter().enumerate() {
            if f.alternate() {
                if let StandardOp::CoreOp(CoreOp::Comment(comment)) = op {
                    if f.alternate() {
                        write!(f, "{:4}  ", "")?;
                    }
                    comment_count += 1;
                    writeln!(f, "{}// {}", "   ".repeat(indent), comment,)?;
                    continue;
                }

                write!(f, "{:04x?}: ", i - comment_count)?;
            } else if let StandardOp::CoreOp(CoreOp::Comment(_)) = op {
                continue;
            }

            writeln!(
                f,
                "{}{}",
                match op {
                    StandardOp::CoreOp(CoreOp::Fn(_))
                    | StandardOp::CoreOp(CoreOp::If(_))
                    | StandardOp::CoreOp(CoreOp::While(_)) => {
                        indent += 1;
                        "   ".repeat(indent - 1)
                    }
                    StandardOp::CoreOp(CoreOp::Else) => {
                        "   ".repeat(indent - 1)
                    }
                    StandardOp::CoreOp(CoreOp::End) => {
                        indent -= 1;
                        "   ".repeat(indent)
                    }
                    _ => "   ".repeat(indent),
                },
                op
            )?
        }
        Ok(())
    }
}

impl AssemblyProgram for StandardProgram {
    fn op(&mut self, op: CoreOp) {
        self.0.push(StandardOp::CoreOp(op))
    }

    fn std_op(&mut self, op: super::StandardOp) -> Result<(), Error> {
        self.0.push(op);
        Ok(())
    }
}

/// A standard instruction of the assembly language. These are instructions
/// that should be implemented for every target possible. Standard instructions
/// should only not be implemented for targets like physical hardware, where the
/// program is executed on the bare metal (a custom CPU or FPGA).
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum StandardOp {
    /// Execute a core instruction.
    CoreOp(CoreOp),

    Set(Location, f64),

    ToFloat(Location),
    ToInt(Location),

    Pow {
        src: Location,
        dst: Location,
    },
    Sqrt(Location),

    Add {
        src: Location,
        dst: Location,
    },
    Sub {
        src: Location,
        dst: Location,
    },
    Mul {
        src: Location,
        dst: Location,
    },
    Div {
        src: Location,
        dst: Location,
    },
    Rem {
        src: Location,
        dst: Location,
    },
    Neg(Location),

    Sin(Location),
    Cos(Location),
    Tan(Location),
    ASin(Location),
    ACos(Location),
    ATan(Location),

    /// Perform dst = a > b.
    IsGreater {
        a: Location,
        b: Location,
        dst: Location,
    },
    /// Perform dst = a < b.
    IsLess {
        a: Location,
        b: Location,
        dst: Location,
    },

    Alloc(Location),
    Free(Location),

    Peek(Location),
    Poke(Location),
}

fn unsupported(op: StandardOp) -> Result<(), Error> {
    Err(Error::UnsupportedInstruction(op))
}

impl StandardOp {
    #[allow(unused_variables)]
    pub(super) fn assemble(
        &self,
        current_instruction: usize,
        env: &mut Env,
        result: &mut dyn VirtualMachineProgram,
    ) -> Result<(), Error> {
        match self {
            Self::CoreOp(op) => op.assemble(current_instruction, env, result)?,

            Self::Set(loc, val) => {
                if loc.set_float(*val, result).is_err() {
                    unsupported(self.clone())?
                }
            }

            Self::Sin(loc) => {
                if loc.sin(result).is_err() {
                    unsupported(self.clone())?
                }
            }
            Self::Cos(loc) => {
                if loc.cos(result).is_err() {
                    unsupported(self.clone())?
                }
            }
            Self::Tan(loc) => {
                if loc.tan(result).is_err() {
                    unsupported(self.clone())?
                }
            }
            Self::ASin(loc) => {
                if loc.asin(result).is_err() {
                    unsupported(self.clone())?
                }
            }
            Self::ACos(loc) => {
                if loc.acos(result).is_err() {
                    unsupported(self.clone())?
                }
            }
            Self::ATan(loc) => {
                if loc.atan(result).is_err() {
                    unsupported(self.clone())?
                }
            }

            Self::ToFloat(loc) => {
                if loc.to_float(result).is_err() {
                    unsupported(self.clone())?
                }
            }
            Self::ToInt(loc) => {
                if loc.to_int(result).is_err() {
                    unsupported(self.clone())?
                }
            }

            Self::Add { src, dst } => {
                if dst.add_float(src, result).is_err() {
                    unsupported(self.clone())?
                }
            }

            Self::Sub { src, dst } => {
                if dst.sub_float(src, result).is_err() {
                    unsupported(self.clone())?
                }
            }

            Self::Mul { src, dst } => {
                if dst.mul_float(src, result).is_err() {
                    unsupported(self.clone())?
                }
            }

            Self::Div { src, dst } => {
                if dst.div_float(src, result).is_err() {
                    unsupported(self.clone())?
                }
            }

            Self::Rem { src, dst } => {
                if dst.rem_float(src, result).is_err() {
                    unsupported(self.clone())?
                }
            }

            Self::Pow { src, dst } => {
                if dst.pow_float(src, result).is_err() {
                    unsupported(self.clone())?
                }
            }

            Self::IsLess { a, b, dst } => {
                if a.is_less_than_float(b, dst, result).is_err() {
                    unsupported(self.clone())?
                }
            }

            Self::IsGreater { a, b, dst } => {
                if a.is_greater_than_float(b, dst, result).is_err() {
                    unsupported(self.clone())?
                }
            }
            Self::Alloc(loc) => {
                if loc.alloc(result).is_err() {
                    unsupported(self.clone())?
                }
            }
            Self::Free(loc) => {
                if loc.free(result).is_err() {
                    unsupported(self.clone())?
                }
            }

            _ => {
                panic!("unimplemented {}", self)
            }
        }
        Ok(())
    }
}

impl fmt::Display for StandardOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::CoreOp(op) => write!(f, "{op}"),

            Self::Set(loc, n) => write!(f, "set-f {loc}, {n}"),

            Self::ToFloat(loc) => write!(f, "to-float {loc}"),
            Self::ToInt(loc) => write!(f, "to-int {loc}"),

            Self::Pow { src, dst } => write!(f, "pow {src}, {dst}"),
            Self::Sqrt(loc) => write!(f, "sqrt {loc}"),

            Self::Add { src, dst } => write!(f, "add-f {src}, {dst}"),
            Self::Sub { src, dst } => write!(f, "sub-f {src}, {dst}"),
            Self::Mul { src, dst } => write!(f, "mul-f {src}, {dst}"),
            Self::Div { src, dst } => write!(f, "div-f {src}, {dst}"),
            Self::Rem { src, dst } => write!(f, "rem-f {src}, {dst}"),
            Self::Neg(loc) => write!(f, "neg-f {loc}"),

            Self::Sin(loc) => write!(f, "sin {loc}"),
            Self::Cos(loc) => write!(f, "cos {loc}"),
            Self::Tan(loc) => write!(f, "tan {loc}"),
            Self::ASin(loc) => write!(f, "asin {loc}"),
            Self::ACos(loc) => write!(f, "acos {loc}"),
            Self::ATan(loc) => write!(f, "atan {loc}"),

            Self::IsGreater { a, b, dst } => write!(f, "gt-f {a}, {b}, {dst}"),
            Self::IsLess { a, b, dst } => write!(f, "lt-f {a}, {b}, {dst}"),

            Self::Alloc(loc) => write!(f, "alloc {loc}"),
            Self::Free(loc) => write!(f, "free {loc}"),

            Self::Peek(loc) => write!(f, "peek {loc}"),
            Self::Poke(loc) => write!(f, "poke {loc}"),
        }
    }
}

impl From<CoreProgram> for StandardProgram {
    fn from(core: CoreProgram) -> Self {
        Self(core.0.into_iter().map(StandardOp::CoreOp).collect())
    }
}
