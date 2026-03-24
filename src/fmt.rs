use crate::mach::{Machine, MachineMode};

use core::cell::Cell;
use std::ops::Deref;

pub struct PrettyPrinter<'a, T>(
    pub(crate) &'a T,
    pub(crate) &'a dyn Machine,
    pub(crate) MachineMode,
);

impl<'a, T> Deref for PrettyPrinter<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

pub trait PrettyPrint {
    fn fmt(
        &self,
        f: &mut core::fmt::Formatter,
        mach: &dyn Machine,
        mode: MachineMode,
    ) -> core::fmt::Result;
}

impl<T> PrettyPrint for T
where
    for<'a> PrettyPrinter<'a, T>: core::fmt::Display,
{
    fn fmt(
        &self,
        f: &mut core::fmt::Formatter,
        mach: &dyn Machine,
        mode: MachineMode,
    ) -> core::fmt::Result {
        core::fmt::Display::fmt(&PrettyPrinter(self, mach, mode), f)
    }
}

impl<'a, T: PrettyPrint> PrettyPrint for &'a T {
    fn fmt(
        &self,
        f: &mut core::fmt::Formatter,
        mach: &dyn Machine,
        mode: MachineMode,
    ) -> core::fmt::Result {
        <T as PrettyPrint>::fmt(self, f, mach, mode)
    }
}

pub struct FormatList<I>(Cell<Option<I>>, &'static str);

impl<I: Iterator> core::fmt::Display for FormatList<I>
where
    I::Item: core::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sep_str = self.1;
        let mut sep = "";

        let iter = self.0.take().unwrap();

        for elem in iter {
            f.write_str(sep)?;

            elem.fmt(f)?;
            sep = sep_str;
        }

        Ok(())
    }
}

pub struct PrettyPrintList<'a, I>(Cell<Option<I>>, &'static str, &'a dyn Machine, MachineMode);

impl<'a, I: Iterator> core::fmt::Display for PrettyPrintList<'a, I>
where
    I::Item: PrettyPrint,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sep_str = self.1;
        let mut sep = "";

        let iter = self.0.take().unwrap();

        for elem in iter {
            f.write_str(sep)?;

            elem.fmt(f, self.2, self.3)?;
            sep = sep_str;
        }

        Ok(())
    }
}

pub fn format_list<I: IntoIterator>(list: I, sep: &'static str) -> FormatList<I::IntoIter>
where
    I::Item: core::fmt::Display,
{
    FormatList(Cell::new(Some(list.into_iter())), sep)
}

pub fn pretty_print_list<'a, I: IntoIterator>(
    list: I,
    sep: &'static str,
    mach: &'a dyn Machine,
    mode: MachineMode,
) -> PrettyPrintList<'a, I::IntoIter>
where
    I::Item: PrettyPrint,
{
    PrettyPrintList(Cell::new(Some(list.into_iter())), sep, mach, mode)
}
