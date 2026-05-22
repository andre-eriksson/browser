use crate::{
    ImageContext, Rect,
    context::{FloatContext, PositionContext},
};

/// Context passed down during layout computation
#[derive(Debug)]
pub struct LayoutContext<'layout, 'nodes> {
    containing_block: Rect,
    positioned_containing_block: Rect,
    deferred: bool,
    position_ctx: &'layout mut PositionContext<'nodes>,
    float_ctx: FloatContext,
    image_ctx: &'layout ImageContext,
}

impl<'layout, 'nodes> LayoutContext<'layout, 'nodes> {
    /// Creates a new `LayoutContext` with the given containing block
    pub(crate) fn new(
        containing_block: Rect,
        image_ctx: &'layout ImageContext,
        position_ctx: &'layout mut PositionContext<'nodes>,
    ) -> Self {
        Self {
            containing_block,
            positioned_containing_block: containing_block,
            deferred: false,
            position_ctx,
            float_ctx: FloatContext::new(),
            image_ctx,
        }
    }

    /// Creates a new `LayoutContext` for deferred layout, which will be used for elements that are
    /// laid out in a second pass after the initial layout has completed.
    pub(crate) fn deferred(
        containing_block: Rect,
        positioned_containing_block: Rect,
        image_ctx: &'layout ImageContext,
        position_ctx: &'layout mut PositionContext<'nodes>,
    ) -> Self {
        Self {
            containing_block,
            positioned_containing_block,
            deferred: true,
            position_ctx,
            float_ctx: FloatContext::new(),
            image_ctx,
        }
    }

    /// Creates a child context with the specified containing block, inheriting
    /// the image and position contexts.
    pub(crate) fn child_context(&'nodes mut self, containing_block: Rect, deferred: bool) -> LayoutContext<'_, '_> {
        if deferred {
            LayoutContext::deferred(
                containing_block,
                self.positioned_containing_block,
                self.image_ctx,
                self.position_ctx,
            )
        } else {
            LayoutContext::new(containing_block, self.image_ctx, &mut *self.position_ctx)
        }
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

    pub fn position_ctx(&'nodes mut self) -> &mut PositionContext {
        self.position_ctx
    }

    pub fn float_ctx(&mut self) -> &mut FloatContext {
        &mut self.float_ctx
    }

    pub fn float_ctx_ref(&self) -> &FloatContext {
        &self.float_ctx
    }

    pub fn image_ctx(&self) -> &ImageContext {
        self.image_ctx
    }

    /// Sets the nearest positioned ancestor containing block used by absolute positioning.
    pub const fn set_positioned_containing_block(&mut self, rect: Rect) {
        self.positioned_containing_block = rect;
    }
}
