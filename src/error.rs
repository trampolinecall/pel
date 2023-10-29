use crate::source::Span;

pub struct ErrorReportedPromise(());

pub(crate) struct Error<'file> {
    main_message: String,
    span: Option<Span<'file>>,
}
impl Error<'_> {
    pub(crate) fn new(span: Option<Span<'_>>, main_message: String) -> Error<'_> {
        Error { main_message, span }
    }
}

mod private {
    pub trait Sealed {}
}
pub trait Report: private::Sealed {
    fn report(self) -> ErrorReportedPromise;
}

impl<'file, T: Into<Error<'file>>> private::Sealed for T {}
impl<'file, T: Into<Error<'file>>> Report for T {
    fn report(self) -> ErrorReportedPromise {
        report(self.into())
    }
}

fn report(error: Error) -> ErrorReportedPromise {
    if let Some(span) = error.span {
        eprintln!("error at {span}: {}", error.main_message);
    } else {
        eprintln!("error: {}", error.main_message);
    }

    // TODO: do this better

    ErrorReportedPromise(())
}
