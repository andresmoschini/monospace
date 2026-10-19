//! The loop's machinery and the application's configuration — the two things that are not the
//! document being edited.
//!
//! The name is `run_tui`'s, so this crate reads against the framework's documentation without a
//! translation table.

use rat_salsa::{SalsaAppContext, SalsaContext};

use crate::config::Config;
use crate::vocabulary::{AppError, AppEvent};

/// The loop's half of the pair `run_tui` takes.
pub(crate) struct AppGlobal {
    /// The frame count, the timings, the terminal, the focus and the queue. Read through
    /// `SalsaContext`, written by the loop.
    ctx: SalsaAppContext<AppEvent, AppError>,

    /// What the painter reads.
    pub cfg: Config,
}

impl AppGlobal {
    /// Builds the global from its configuration. The loop installs the context itself, before
    /// `init` is called.
    #[must_use]
    pub fn new(cfg: Config) -> Self {
        Self {
            ctx: SalsaAppContext::default(),
            cfg,
        }
    }
}

impl SalsaContext<AppEvent, AppError> for AppGlobal {
    fn set_salsa_ctx(&mut self, app_ctx: SalsaAppContext<AppEvent, AppError>) {
        self.ctx = app_ctx;
    }

    fn salsa_ctx(&self) -> &SalsaAppContext<AppEvent, AppError> {
        &self.ctx
    }
}
