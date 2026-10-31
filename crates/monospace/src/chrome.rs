//! The chrome's own state: the bar, the keyboard, and what a press lands on.
//!
//! **The bar and the keyboard are one thing** (rule 1 of #189): the drop-down is the bar's, so a
//! menu that stays open implies the bar holds the keyboard, and a meaning carried out closes the
//! one and gives the other back together. [`Chrome`] owns the pair so that no module can do half
//! of it.
//!
//! **The leaf is not a meaning** (D1 of #189): a press, `F10`, `Alt` plus a label's character and
//! the `Esc` that closes nothing move the keyboard where they are read, and none of them becomes
//! an action. [`Target`] is the one answer to where a press landed, and [`Keyboard`] is the only
//! thing that moves the leaf.

use monospace_core::Offset;
use rat_widget::menu::MenubarState;

/// Which leaf of the chrome holds the keyboard.
///
/// **The bar and the region are the two leaves** (rule 1 of #189): the status line is not one, and
/// a control inside a leaf — a label, an entry, the dialog's button — never holds the keyboard on
/// its own. The dialog is an overlay and runs its own focus while it is up; the application's leaf
/// stays where it was, which is what makes closing the dialog give the keyboard back unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Focus {
    /// The menu bar, and the drop-down it opens: the pop-up is the bar's.
    Bar,

    /// The diagram region, border included.
    Region,
}

/// The keyboard, and the ways it moves.
///
/// **The leaf is not a meaning** (D1 of #189): a press, `F10`, `Alt` plus a label's character and
/// the `Esc` that closes nothing move the keyboard where they are read, and none of them becomes
/// an action. **The moves are named here so that no module writes the leaf**: the bar asks for the
/// keyboard and gives it back, and a press addresses it by its target.
///
/// **A move says whether it moved**, which is what owes the frame: a key that asks for the leaf
/// already held changes nothing on the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Keyboard {
    leaf: Focus,
}

impl Keyboard {
    /// A keyboard on the region, which is where an application starts (D1 of #189).
    fn region() -> Self {
        Self {
            leaf: Focus::Region,
        }
    }

    /// The leaf that holds it.
    fn leaf(self) -> Focus {
        self.leaf
    }

    /// Whether the bar holds it.
    fn is_bar(self) -> bool {
        self.leaf == Focus::Bar
    }

    /// The bar asks for the keyboard, saying whether it moved.
    fn move_to_bar(&mut self) -> bool {
        let moved = self.leaf != Focus::Bar;
        self.leaf = Focus::Bar;
        moved
    }

    /// The bar gives the keyboard back, saying whether it moved.
    fn move_to_region(&mut self) -> bool {
        let moved = self.leaf != Focus::Region;
        self.leaf = Focus::Region;
        moved
    }

    /// A press addresses the keyboard, saying whether it moved.
    ///
    /// **A target in neither leaf leaves it alone**: the status line and empty space are not
    /// destinations for the keyboard.
    fn at(&mut self, target: Target) -> bool {
        match target {
            Target::Bar => self.move_to_bar(),
            Target::Border | Target::Interior(_) => self.move_to_region(),
            Target::Nothing => false,
        }
    }
}

/// The chrome's own state: the bar and the keyboard.
///
/// **The pair is one thing** (rule 1 of #189): the drop-down is the bar's, so a menu that stays
/// open implies the bar holds the keyboard, and a meaning carried out closes the one and gives the
/// other back together. They travel together so that a handler cannot be handed a bar with the
/// keyboard somewhere else, and so that the closing has one home.
pub(crate) struct Chrome {
    menu: MenubarState,

    keyboard: Keyboard,
}

impl Chrome {
    /// The chrome of an application that starts on the region (D1 of #189).
    pub(crate) fn new() -> Self {
        Self {
            menu: MenubarState::default(),
            keyboard: Keyboard::region(),
        }
    }

    /// The bar's own state, for the frame that paints it and the test that reads it.
    pub(crate) fn menu(&self) -> &MenubarState {
        &self.menu
    }

    /// The bar's own state, for the widget that answers its events and the frame that draws it.
    ///
    /// **The mutable door is the widget's and the frame's**: `rat-menu` opens and closes the
    /// drop-down itself, and the frame hands it the state to paint. What the application does
    /// about the pair is [`Self::close`].
    pub(crate) fn menu_mut(&mut self) -> &mut MenubarState {
        &mut self.menu
    }

    /// Whether the drop-down is open.
    pub(crate) fn popup_active(&self) -> bool {
        self.menu.popup_active()
    }

    /// The drop-down closes while the bar keeps the keyboard.
    ///
    /// **An entry taken is this and not [`Self::close`]**: the meaning is carried out afterwards,
    /// and that is what gives the keyboard back.
    pub(crate) fn close_popup(&mut self) {
        self.menu.set_popup_active(false);
    }

    /// The bar asks for the keyboard, saying whether it moved.
    pub(crate) fn move_to_bar(&mut self) -> bool {
        self.keyboard.move_to_bar()
    }

    /// The bar gives the keyboard back, saying whether it moved.
    pub(crate) fn move_to_region(&mut self) -> bool {
        self.keyboard.move_to_region()
    }

    /// The leaf that holds the keyboard.
    pub(crate) fn leaf(&self) -> Focus {
        self.keyboard.leaf()
    }

    /// Whether the bar holds the keyboard.
    pub(crate) fn is_bar(&self) -> bool {
        self.keyboard.is_bar()
    }

    /// A press addresses the keyboard, saying whether it moved.
    pub(crate) fn at(&mut self, target: Target) -> bool {
        self.keyboard.at(target)
    }

    /// **A meaning carried out closes the drop-down and gives the keyboard back together**
    /// (rule 13 of #189): what a person took from the bar is done, and what is left to do is in
    /// the drawing. **The two are one operation** because a menu nobody holds the keyboard on is
    /// not open, and doing one without the other is the state this type exists to prevent.
    pub(crate) fn close(&mut self) {
        self.close_popup();
        self.keyboard.move_to_region();
    }
}

/// What a left press lands on.
///
/// **One answer, decided once.** The leaf the keyboard moves to and the cell of the drawing under
/// the pointer are two readings of the same press, so they are taken together: a rectangle cannot
/// belong to one reading and not to the other. The drop-down is asked about before the region
/// (rule 1 of #189), and the region's border is the keyboard's without being a cell of the
/// drawing (rule 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Target {
    /// The bar, its labels, and the drop-down it opens.
    Bar,

    /// The region's border: the keyboard's, with no cell of the drawing under it.
    Border,

    /// The diagram's interior, at this cell of its window.
    Interior(Offset),

    /// Neither leaf: the status line, or a rectangle no frame drew into.
    Nothing,
}
