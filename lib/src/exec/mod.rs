// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![allow(clippy::module_name_repetitions)]

use crate::inst::{InstSet, Op};
use std::{
    collections::BTreeMap,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    io::{stdin, stdout, BufReader, Read, Write},
    str::FromStr,
};

/// # Arithmetic
/// Arithmetic instructions
#[allow(clippy::needless_pass_by_value, clippy::enum_glob_use)]
pub mod arith;

/// # I/O
/// I/O, debugging, function call and return instructions
#[allow(clippy::needless_pass_by_value, clippy::enum_glob_use)]
pub mod io;

/// # Data movement
/// Instructions for moving data between registers and memory addresses
#[allow(clippy::needless_pass_by_value, clippy::enum_glob_use)]
pub mod mov;

/// # Comparison
/// Instructions for making logical comparisons
#[allow(clippy::needless_pass_by_value, clippy::enum_glob_use)]
pub mod cmp;

/// # Bit manipulation
/// Instructions for logical bit manipulation
#[allow(clippy::needless_pass_by_value, clippy::enum_glob_use)]
pub mod bitman;

#[allow(clippy::enum_glob_use)]
mod error;

mod memory;

mod debug;

#[allow(clippy::enum_glob_use)]
mod inst;

pub use error::{RtError, RtResult, Source};

pub use memory::Memory;

pub use inst::{ExecFunc, ExecInst};

pub use debug::DebugInfo;

/// For platform independent I/O
///
/// Boxed for convenience.
pub struct Io {
    pub read: BufReader<Box<dyn Read + Send + Sync>>,
    pub write: Box<dyn Write + Send + Sync>,
}

/// Quickly makes an [`Io`] struct
///
/// # Arguments (optional)
///
/// * `$read`: must implement [`Read`].
/// * `$write`: must implement [`Write`].
///
/// # Example
/// ```
/// use cambridge_asm::make_io;
///
/// let default_io = make_io!(); // no macro arguments will give the default I/O provider, i.e. stdio
/// let io = make_io!(std::io::stdin(), std::io::sink()); // you can use your own providers too
/// ```
#[macro_export]
macro_rules! make_io {
    () => {
        $crate::exec::Io::default()
    };
    ($read:expr, $write:expr) => {{
        $crate::exec::Io {
            read: std::io::BufReader::new(Box::new($read)),
            write: Box::new($write),
        }
    }};
}

impl Debug for Io {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("<struct Io>")
    }
}

impl Default for Io {
    fn default() -> Self {
        Self {
            read: BufReader::new(Box::new(stdin())),
            write: Box::new(stdout()),
        }
    }
}

/// Tracks state of the registers and memory during execution
#[derive(Debug, Default)]
pub struct Context {
    pub cmp: bool,
    pub mar: usize,
    pub acc: usize,
    pub ix: usize,
    pub flow_override_reg: bool,
    pub mem: Memory,
    pub ret: usize,
    pub gprs: [usize; 30],
    pub end: bool,
    pub io: Io,
}

impl Context {
    pub fn new(mem: Memory) -> Self {
        Self {
            mem,
            ..Self::default()
        }
    }

    pub fn with_io(mem: Memory, io: Io) -> Self {
        Self {
            mem,
            io,
            ..Self::default()
        }
    }

    #[inline]
    pub fn override_flow_control(&mut self) {
        self.flow_override_reg = true;
    }

    /// # Panics
    /// If `op` is not a `usize` register. To avoid this, check `op` using [`Op::is_register`].
    #[inline]
    pub fn get_mut_register(&mut self, op: &Op) -> &mut usize {
        match op {
            Op::Acc => &mut self.acc,
            Op::Ix => &mut self.ix,
            Op::Ar => &mut self.ret,
            Op::Gpr(x) => &mut self.gprs[*x],
            _ => unreachable!(),
        }
    }

    /// # Panics
    /// If `op` is not a `usize` register. To avoid this, check `op` using [`Op::is_register`].
    #[inline]
    pub fn get_register(&self, op: &Op) -> usize {
        match op {
            Op::Acc => self.acc,
            Op::Ix => self.ix,
            Op::Ar => self.ret,
            Op::Gpr(x) => self.gprs[*x],
            _ => unreachable!(),
        }
    }

    /// Read the given operand from the context
    ///
    /// # Arguments
    ///
    /// * `op`:
    ///
    /// returns: `RtResult`
    ///
    /// # Panics
    ///
    /// If `op` is not usizeable. To avoid this, check `op` using [`Op::is_usizeable`]
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use cambridge_asm::inst;
    /// inst!(print (ctx, op) {
    ///     if op.is_usizeable() {
    ///         println!("{}", ctx.read(op)?);
    ///     }
    /// });
    /// ```
    #[inline]
    pub fn read(&self, op: &Op) -> RtResult<usize> {
        match op {
            &Op::Literal(val) => Ok(val),
            Op::Addr(addr) => self.mem.get(addr).copied(),
            Op::Indirect(op) if op.is_usizeable() => {
                let addr = self.read(op)?;
                self.mem.get(&addr).copied()
            }
            reg if reg.is_register() => Ok(self.get_register(reg)),
            _ => unreachable!(),
        }
    }

    /// Modify the given operand in the context if it is writeable
    ///
    /// # Arguments
    ///
    /// * `op`: operand
    /// * `f`: closure to modify the value
    ///
    /// returns: [`RtResult`]
    ///
    /// # Panics
    ///
    /// If `op` is not writeable. To avoid this, check `op` using [`Op::is_read_write`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use cambridge_asm::inst;
    /// inst!(double_inc (ctx, op) {
    ///     if op.is_read_write() {
    ///         ctx.modify(op, |val| *val += 2)?;
    ///     }
    /// });
    /// ```
    #[inline]
    pub fn modify(&mut self, op: &Op, f: impl Fn(&mut usize)) -> RtResult {
        match op {
            Op::Addr(x) => f(self.mem.get_mut(x)?),
            Op::Indirect(op) if op.is_usizeable() => {
                let addr = self.read(op)?;
                f(self.mem.get_mut(&addr)?);
            }
            op if op.is_register() => f(self.get_mut_register(op)),
            _ => unreachable!(),
        }

        Ok(())
    }
}

impl Display for Context {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("Context {\n")?;
        writeln!(f, "{:>6}: {}", "mar", self.mar)?;
        writeln!(f, "{:>6}: {}", "acc", self.acc)?;
        writeln!(f, "{:>6}: {}", "ix", self.ix)?;
        writeln!(f, "{:>6}: {}", "cmp", self.cmp)?;
        write!(f, "{:>6}: [", "gprs")?;

        for (idx, val) in self.gprs.iter().enumerate() {
            if idx == self.gprs.len() - 1 {
                writeln!(f, "r{idx} = {val}]")?;
            } else {
                write!(f, "r{idx} = {val}, ")?;
            }
        }

        writeln!(f, "{:>6}: Memory {{", "mem")?;

        for (addr, entry) in &self.mem {
            writeln!(f, "{addr:>8}: {entry},")?;
        }

        writeln!(f, "{:>6}}}", "")?;

        f.write_str("}")
    }
}

/// Runtime representation of a program
pub type ExTree = BTreeMap<usize, ExecInst>;

/// Executes a program
pub struct Executor {
    pub debug_info: DebugInfo,
    pub source: Source,
    pub prog: ExTree,
    pub ctx: Context,
    count: u64,
}

/// Shows execution status
pub enum Status {
    /// Program has finished execution
    Complete,
    /// Program has not finished execution
    Continue,
    /// An error has been encountered during execution
    Error(RtError),
}

impl Executor {
    pub fn new(
        source: impl Into<Source>,
        prog: ExTree,
        ctx: Context,
        debug_info: DebugInfo,
    ) -> Self {
        Self {
            debug_info,
            source: source.into(),
            prog,
            ctx,
            count: 0,
        }
    }

    /// Advance execution by one instruction
    ///
    /// # Example
    /// ```no_run
    ///
    /// ```
    pub fn step<T>(&mut self) -> Status
    where
        T: InstSet,
        <T as FromStr>::Err: Display,
    {
        if self.ctx.mar == self.prog.len() || self.ctx.end {
            Status::Complete
        } else {
            self.count += 1;

            let inst = if let Some(inst) = self.prog.get(&self.ctx.mar) {
                inst
            } else {
                panic!("Unable to fetch instruction. Please report this as a bug with full debug logs attached.")
            };

            trace!(
                "Executing instruction {} {}",
                T::from_id(inst.id).unwrap_or_else(|msg| panic!("{msg}")),
                inst.op
            );

            match (inst.func)(&mut self.ctx, &inst.op) {
                Ok(()) => {
                    if self.ctx.flow_override_reg {
                        self.ctx.flow_override_reg = false;
                    } else {
                        self.ctx.mar += 1;
                    }

                    Status::Continue
                }
                Err(e) => Status::Error(e),
            }
        }
    }

    pub fn exec<T>(&mut self)
    where
        T: InstSet,
        <T as FromStr>::Err: Display,
    {
        let err = loop {
            match self.step::<T>() {
                Status::Complete => break None,
                Status::Continue => continue,
                Status::Error(e) => break Some(e),
            }
        };

        if let Some(e) = err {
            self.source
                .handle_err(&mut self.ctx.io.write, &e, self.ctx.mar)
                .unwrap();
        } else {
            info!("Total instructions executed: {}", self.count);
        }
    }

    pub fn display_with_opcodes<T>(&self) -> Result<String, <T as FromStr>::Err>
    where
        T: InstSet,
        <T as FromStr>::Err: Display,
    {
        use std::fmt::Write;

        let mut s = String::new();

        s.reserve(self.prog.len() * 15);

        writeln!(s, "Executor {{").unwrap();

        for (addr, ExecInst { id, op, .. }) in &self.prog {
            writeln!(s, "{addr:>6}: {func} {op}", func = T::from_id(*id)?).unwrap();
        }

        s.push('}');

        Ok(s)
    }
}

impl Display for Executor {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("Executor {")?;
        for (addr, ExecInst { op, .. }) in &self.prog {
            writeln!(f, "{addr:>6}: {op}")?;
        }
        f.write_str("}")
    }
}

impl Debug for Executor {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("Executor")
            .field("source", &self.source)
            .field(
                "prog",
                &self
                    .prog
                    .iter()
                    .map(|(addr, ExecInst { op, .. })| (addr, op))
                    .collect::<Vec<_>>(),
            )
            .field("ctx", &self.ctx)
            .field("count", &self.count)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Executor>();
    }

    #[test]
    fn exec() {
        let prog =
            // Division algorithm from examples/division.pasm
            [
                (0, ExecInst::new(0, arith::inc, "202".into())),
                (1, ExecInst::new(0, arith::add, "203,201".into())),
                (2, ExecInst::new(0, cmp::cmp, "203,204".into())),
                (3, ExecInst::new(0, cmp::jpn, "0".into())),
                (4, ExecInst::new(0, mov::ldd, "202".into())),
                (5, ExecInst::new(0, io::end, "".into())),
            ].into();

        let mem = [(200, 0), (201, 5), (202, 0), (203, 0), (204, 15)].into();

        let mut exec = Executor::new(
            "None",
            prog,
            Context::new(Memory::new(mem)),
            DebugInfo::default(),
        );

        exec.exec::<crate::parse::DefaultSet>();

        assert_eq!(exec.ctx.acc, 3);
    }
}
