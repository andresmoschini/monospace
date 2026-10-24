//! The loop's machinery, the configuration, and the failure that was reported.
//!
//! The name is the framework's, so this crate reads against the framework's documentation without
//! a translation table.

use rat_salsa::{SalsaAppContext, SalsaContext};

use crate::config::Config;
use crate::event::AppEvent;
use crate::vocabulary::AppError;

/// The loop's half of the pair `run_tui` takes.
pub(crate) struct AppGlobal {
    /// The frame count, the timings, the terminal and the queue. Read through `SalsaContext`,
    /// written by the loop.
    ctx: SalsaAppContext<AppEvent, AppError>,

    /// What the application is configured with. Read once, at startup, and not after.
    pub cfg: Config,

    /// The failure the error handler took, kept to be written out with the terminal back.
    failure: Option<AppError>,
}

impl AppGlobal {
    /// Builds the global from its configuration. The loop installs the context itself, before
    /// `init` is called.
    #[must_use]
    pub(crate) fn new(cfg: Config) -> Self {
        Self {
            ctx: SalsaAppContext::default(),
            cfg,
            failure: None,
        }
    }

    /// Keeps the failure the error handler was given.
    pub(crate) fn set_failure(&mut self, failure: AppError) {
        self.failure = Some(failure);
    }

    /// Takes the failure, if there is one.
    pub(crate) fn take_failure(&mut self) -> Option<AppError> {
        self.failure.take()
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
