//! The core instructions of the virtual machine are defined here.
//!
//! Core instructions are instructions that **must** be implemented for
//! every target. Write programs in the core variant to guarantee ports
//! for ***every*** target.
use crate::side_effects::{Input, Output};

use super::{Error, StandardOp, StandardProgram, VirtualMachineProgram};
use core::fmt;
use std::{collections::HashMap, hash::Hash};

impl VirtualMachineProgram for CoreProgram {
    fn op(&mut self, op: CoreOp) {
        self.0.push(op);
    }

    fn std_op(&mut self, op: StandardOp) -> Result<(), Error> {
        Err(Error::UnsupportedInstruction(op))
    }

    fn code(&self) -> Result<CoreProgram, StandardProgram> {
        Ok(self.clone())
    }
}

/// A program of only core virtual machine instructions.
#[derive(Default, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CoreProgram(pub Vec<CoreOp>);

impl CoreProgram {
    /// Flatten a core program so that all of its functions
    /// are defined sequentially at the beginning.
    pub fn flatten(self) -> Self {
        Self(flatten(self.0).0)
    }

    /// Get the code outside of any functions.
    pub fn get_main(&self) -> Vec<CoreOp> {
        flatten(self.0.clone()).2
    }

    /// Get the code for each function.
    pub fn get_functions(&self) -> HashMap<i32, Vec<CoreOp>> {
        flatten(self.0.clone()).1
    }

    /// Get the code outside of any functions, and the code for each function.
    pub fn get_main_and_functions(self) -> (Vec<CoreOp>, HashMap<i32, Vec<CoreOp>>) {
        let (_, functions, main) = flatten(self.0);
        (main, functions)
    }
}

/// Take all of the functions defined in a list of CoreOps,
/// and flatten their definitions. This will take nested functions
/// and un-nest them while preserving the order in which functions are defined.
///
/// All the function definitions will be placed at the top of the returned list.
fn flatten(code: Vec<CoreOp>) -> (Vec<CoreOp>, HashMap<i32, Vec<CoreOp>>, Vec<CoreOp>) {
    let mut functions: HashMap<i32, Vec<CoreOp>> = HashMap::new();

    // The current function body we are in.
    let mut fun = -1;
    // Keep track of when we end the current function,
    // instead of just an if-else-conditional or a while loop.
    // This is essentially the number of end statements remaining before
    // we can end the scope.
    let mut matching_end = 0;
    // Keep track of each `matching_end`, and the scope we were previously in, for each nested scope.
    let mut scope_stack = vec![];
    // All of the instructions which are not part of a function definition.
    let mut main_instructions = vec![];
    for op in code {
        match &op {
            CoreOp::Function => {}
            _ => {
                if scope_stack.is_empty() {
                    // If we are not defining a function,
                    // push the instruction to the main instructions.
                    main_instructions.push(op.clone());
                }
            }
        }

        match op {
            CoreOp::Function => {
                // If we are declaring a new function,
                // push the info about the current scope onto the scope
                // stack to resume later.
                scope_stack.push((fun, matching_end));
                // Reset the matching-end counter for the new scope.
                matching_end = 0;
                // Start defining the next function.
                fun += 1;
                // If that function is already defined,
                // just go past the last function defined.
                if functions.contains_key(&fun) {
                    fun = functions.len() as i32
                }
            }
            CoreOp::If | CoreOp::While => {
                // Increment the number of matching `End`
                // instructions to end the scope.
                matching_end += 1
            }
            CoreOp::End => {
                // If the scope has ended
                if matching_end == 0 {
                    // Get the function body we're defining.
                    functions.entry(fun).or_default().push(op);
                    // Resume flattening the previous scope.
                    (fun, matching_end) = scope_stack.pop().unwrap();
                    continue;
                } else {
                    // Otherwise, the scope is still going.
                    // Decrement the counter and continue.
                    matching_end -= 1;
                }
            }
            _ => {}
        }

        // Insert the current instruction to the right function's definition.
        functions.entry(fun).or_default().push(op);
    }

    // Clone the functions so that we can remove them from the map.
    let mut result_functions = functions.clone();
    result_functions.remove(&-1);

    // The final output code.
    let mut result = vec![];
    // For every function, insert its body into the resulting output code.
    for i in 0..=functions.len() as i32 {
        if let Some(body) = functions.remove(&i) {
            result.extend(body);
        }
    }

    // Insert the remaining code into the output code.
    if let Some(body) = functions.remove(&-1) {
        result.extend(body);
    }

    // Return the output code
    (result, result_functions, main_instructions)
}

impl fmt::Display for CoreProgram {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut comment_count = 0;
        let mut indent = 0;
        for (i, op) in self.0.iter().enumerate() {
            if f.alternate() {
                if let CoreOp::Comment(comment) = op {
                    if f.alternate() {
                        write!(f, "{:8}  ", "")?;
                    }
                    comment_count += 1;
                    writeln!(f, "{}// {}", "   ".repeat(indent), comment,)?;
                    continue;
                }

                write!(f, "{:08x?}: ", i - comment_count)?;
            } else if let CoreOp::Comment(_) = op {
                continue;
            }

            writeln!(
                f,
                "{}{}",
                match op {
                    CoreOp::Function | CoreOp::If | CoreOp::While => {
                        indent += 1;
                        "   ".repeat(indent - 1)
                    }
                    CoreOp::Else => {
                        "   ".repeat(indent - 1)
                    }
                    CoreOp::End => {
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

/// An individual core virtual machine instruction.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum CoreOp {
    /// A comment in the machine code (not in the compiled output).
    Comment(String),

    /// Set the register equal to a constant value.
    Set(Vec<i64>),

    /// Create a new function.
    Function,
    /// Calls the nth function defined in the program, where n is the value of the register.
    Call,
    /// Return from the current function.
    Return,

    /// Begin a "while the register is not zero" loop.
    While,
    /// Begin an "if the register is not zero" conditional.
    ///
    /// If statements are structured like so:
    /// ```hs
    /// if
    ///     ...
    /// end
    ///
    /// if
    ///     ...
    /// else
    ///     ...
    /// end
    /// ```
    If,
    /// Begin an "else" conditional.
    Else,
    /// End a conditional.
    End,

    /// Store the register to the value pointed to on the tape.
    Store(usize),
    /// Store the value pointed to on the tape to the register.
    Load(usize),

    /// Move the pointer on the tape by a number of cells.
    Move(isize),

    /// Store the value of the pointer to the register.
    Where,
    /// The pointer is made equal to the value pointed to on the tape.
    Deref,
    /// The last "deref" operation is undone; the pointer is made equal to the value it was before the last "deref" operation.
    Refer,

    /// Interpret the register's value as a pointer to a cell.
    /// Index that pointer by the value on the tape. Store the address
    /// of the index into the register.
    Index,

    /// Perform bitwise nand on the cell and the value pointed to on the tape,
    /// and store the result in the register.
    BitwiseNand,

    /// Add the value pointed to on the tape to the register.
    Add,
    /// Subtract the value pointed to on the tape from the register.
    Sub,
    /// Multiply the register by the value pointed to on the tape.
    Mul,
    /// Divide the register by the value pointed to on the tape.
    Div,
    /// Store the remainder of the register and the value pointed to in the tape into the register.
    Rem,

    /// Make the register equal to 1 if the register is non-negative, otherwise make it equal to 0.
    IsNonNegative,

    /// Get a value from an input source and store it in the register.
    Get(Input),
    /// Write the value of the register to an output source.
    Put(Output),
}

impl fmt::Display for CoreOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreOp::Comment(s) => write!(f, "// {}", s),
            CoreOp::Set(n) => write!(f, "set {:?}", n),
            CoreOp::Function => write!(f, "fun"),
            CoreOp::Call => write!(f, "call"),
            CoreOp::Return => write!(f, "ret"),
            CoreOp::While => write!(f, "while"),
            CoreOp::If => write!(f, "if"),
            CoreOp::Else => write!(f, "else"),
            CoreOp::End => write!(f, "end"),
            CoreOp::Store(n) => write!(f, "store {n}"),
            CoreOp::Load(n) => write!(f, "load {n}"),
            CoreOp::Move(n) => write!(f, "mov {}", n),
            CoreOp::Where => write!(f, "where"),
            CoreOp::Deref => write!(f, "deref"),
            CoreOp::Refer => write!(f, "ref"),
            CoreOp::Index => write!(f, "index"),
            CoreOp::BitwiseNand => write!(f, "bitwise-nand"),
            CoreOp::Add => write!(f, "add"),
            CoreOp::Sub => write!(f, "sub"),
            CoreOp::Mul => write!(f, "mul"),
            CoreOp::Div => write!(f, "div"),
            CoreOp::Rem => write!(f, "rem"),
            CoreOp::IsNonNegative => write!(f, "gez"),
            CoreOp::Get(i) => write!(f, "get {i}"),
            CoreOp::Put(o) => write!(f, "put {o}"),
        }
    }
}
