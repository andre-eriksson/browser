use crate::Rect;

#[derive(Debug, Clone, Copy)]
pub struct Cursor {
    pub x: f64,
    pub y: f64,
}

/// Context passed down during layout computation
#[derive(Debug)]
pub struct LayoutContext {
    cursor: Cursor,
    containing_block: Rect,
    positioned_containing_block: Rect,
    deferred: bool,
}

impl LayoutContext {
    /// Creates a new `LayoutContext` with the given containing block
    pub(crate) fn new(containing_block: Rect) -> Self {
        Self {
            cursor: Cursor { x: 0.0, y: 0.0 },
            containing_block,
            positioned_containing_block: containing_block,
            deferred: false,
        }
    }

    /// Creates a new `LayoutContext` for deferred layout, which will be used for elements that are
    /// laid out in a second pass after the initial layout has completed.
    pub(crate) fn deferred(cursor: Cursor, containing_block: Rect, positioned_containing_block: Rect) -> Self {
        Self {
            cursor,
            containing_block,
            positioned_containing_block,
            deferred: true,
        }
    }

    /// Creates a child context with the specified containing block, inheriting
    /// the image and position contexts.
    pub(crate) fn child_context(&mut self, containing_block: Rect, deferred: bool) -> Self {
        if deferred {
            Self::deferred(Cursor { x: 0.0, y: 0.0 }, containing_block, self.positioned_containing_block)
        } else {
            Self {
                cursor: Cursor { x: 0.0, y: 0.0 },
                containing_block,
                positioned_containing_block: self.positioned_containing_block,
                deferred: false,
            }
        }
    }

    pub const fn cursor_ref(&self) -> &Cursor {
        &self.cursor
    }

    pub const fn cursor(&mut self) -> &mut Cursor {
        &mut self.cursor
    }

    /// Returns the containing block rect
    pub const fn containing_block(&self) -> Rect {
        self.containing_block
    }

    /// Returns the nearest positioned ancestor containing block used by absolute positioning.
    pub const fn positioned_containing_block(&self) -> Rect {
        self.positioned_containing_block
    }

    pub const fn is_deferred(&self) -> bool {
        self.deferred
    }

    /// Sets the nearest positioned ancestor containing block used by absolute positioning.
    pub const fn set_positioned_containing_block(&mut self, rect: Rect) {
        self.positioned_containing_block = rect;
    }
}
