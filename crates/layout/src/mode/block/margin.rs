use crate::context::BoxModel;

#[derive(Debug, Clone, Copy, Default)]
pub struct MarginCollapsing {
    pub max_positive: f64,
    pub max_negative: f64,
}

impl MarginCollapsing {
    pub fn add(&mut self, margin: f64) {
        if margin >= 0.0 {
            self.max_positive = self.max_positive.max(margin);
        } else {
            self.max_negative = self.max_negative.min(margin);
        }
    }

    pub fn add_collapsed(&mut self, other: &MarginCollapsing) {
        self.add(other.max_positive);
        self.add(other.max_negative);
    }

    pub fn flush(&mut self) -> f64 {
        let collapsed = self.max_positive + self.max_negative;
        self.max_positive = 0.0;
        self.max_negative = 0.0;
        collapsed
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MarginCollapseState {
    pub top_collapsed: bool,
    pub bottom_collapsed: bool,
    pub collapsed_margin: MarginCollapsing,
}

pub fn calculate_top_margin(
    has_block_children: bool,
    has_top_fence: bool,
    current_block: &mut MarginCollapseState,
    box_model: &BoxModel,
) -> f64 {
    current_block
        .collapsed_margin
        .add(box_model.margin.top.to_px());

    if has_top_fence || !has_block_children {
        current_block.top_collapsed = true;
        return current_block.collapsed_margin.flush();
    }

    0.0
}

/// Applies bottom margin changes
///
/// * The bottom margin of an in-flow block-level element always collapses with the top margin of its next in-flow
///   block-level sibling, unless that sibling has clearance.
///
/// * The bottom margin of an in-flow block box with a 'height' of 'auto' and a 'min-height' of zero collapses
///   with its last in-flow block-level child's bottom margin if the box has no bottom padding and no bottom
///   border and the child's bottom margin does not collapse with a top margin that has clearance.
///
/// * A box's own margins collapse if the 'min-height' property is zero, and it has neither top or bottom borders
///   nor top or bottom padding, and it has a 'height' of either 0 or 'auto', and it does not contain a line box,
///   and all of its in-flow children's margins (if any) collapse.
pub fn calculate_bottom_margin(
    is_bfc: bool,
    has_bottom_fence: bool,
    child_block: &mut MarginCollapseState,
    current_block: &mut MarginCollapseState,
    box_model: &BoxModel,
) -> f64 {
    let collapses_with_child = !is_bfc && !has_bottom_fence;

    if collapses_with_child {
        child_block
            .collapsed_margin
            .add(box_model.margin.bottom.to_px());
        current_block
            .collapsed_margin
            .add_collapsed(&child_block.collapsed_margin);
    } else {
        current_block
            .collapsed_margin
            .add(box_model.margin.bottom.to_px());
    }

    if has_bottom_fence || is_bfc {
        current_block.bottom_collapsed = true;
        return child_block.collapsed_margin.flush();
    }

    0.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Margin;

    fn box_model_with_margins(top: f64, bottom: f64) -> BoxModel {
        BoxModel {
            margin: Margin {
                top: top.into(),
                bottom: bottom.into(),
                ..Margin::zero()
            },
            ..Default::default()
        }
    }

    #[test]
    fn margin_collapsing_uses_largest_positive_margin() {
        let mut collapsing = MarginCollapsing::default();
        collapsing.add(10.0);
        collapsing.add(40.0);
        collapsing.add(25.0);

        assert_eq!(collapsing.flush(), 40.0);
    }

    #[test]
    fn margin_collapsing_uses_most_negative_margin() {
        let mut collapsing = MarginCollapsing::default();
        collapsing.add(-10.0);
        collapsing.add(-40.0);
        collapsing.add(-25.0);

        assert_eq!(collapsing.flush(), -40.0);
    }

    #[test]
    fn margin_collapsing_combines_positive_and_negative_extremes() {
        let mut collapsing = MarginCollapsing::default();
        collapsing.add(30.0);
        collapsing.add(10.0);
        collapsing.add(-5.0);
        collapsing.add(-20.0);

        assert_eq!(collapsing.flush(), 10.0);
    }

    #[test]
    fn margin_collapsing_add_collapsed_merges_extremes_from_another_set() {
        let mut first = MarginCollapsing::default();
        first.add(10.0);
        first.add(-5.0);

        let mut second = MarginCollapsing::default();
        second.add(40.0);
        second.add(-30.0);

        first.add_collapsed(&second);

        assert_eq!(first.flush(), 10.0);
    }

    #[test]
    fn margin_collapsing_flush_resets_state() {
        let mut collapsing = MarginCollapsing::default();
        collapsing.add(12.0);
        collapsing.add(-7.0);

        assert_eq!(collapsing.flush(), 5.0);
        assert_eq!(collapsing.flush(), 0.0);
    }

    #[test]
    fn top_margin_flushes_when_parent_has_no_block_children() {
        let box_model = box_model_with_margins(24.0, 0.0);
        let mut margin_state = MarginCollapseState::default();

        let collapsed_top = calculate_top_margin(false, false, &mut margin_state, &box_model);

        assert_eq!(collapsed_top, 24.0);
        assert!(margin_state.top_collapsed);
        assert_eq!(margin_state.collapsed_margin.flush(), 0.0);
    }

    #[test]
    fn top_margin_flushes_when_parent_has_top_fence() {
        let box_model = box_model_with_margins(24.0, 0.0);
        let mut margin_state = MarginCollapseState::default();

        let collapsed_top = calculate_top_margin(true, true, &mut margin_state, &box_model);

        assert_eq!(collapsed_top, 24.0);
        assert!(margin_state.top_collapsed);
        assert_eq!(margin_state.collapsed_margin.flush(), 0.0);
    }

    #[test]
    fn top_margin_stays_pending_when_parent_can_collapse_with_first_child() {
        let box_model = box_model_with_margins(24.0, 0.0);
        let mut margin_state = MarginCollapseState::default();

        let collapsed_top = calculate_top_margin(true, false, &mut margin_state, &box_model);

        assert_eq!(collapsed_top, 0.0);
        assert!(!margin_state.top_collapsed);
        assert_eq!(margin_state.collapsed_margin.flush(), 24.0);
    }

    #[test]
    fn fixed_height_parent_top_margin_still_collapses_with_first_child() {
        let box_model = box_model_with_margins(-100.0, 0.0);
        let mut margin_state = MarginCollapseState::default();

        let collapsed_top = calculate_top_margin(true, false, &mut margin_state, &box_model);

        assert_eq!(collapsed_top, 0.0);
        assert!(!margin_state.top_collapsed);

        margin_state.collapsed_margin.add(100.0);

        assert_eq!(margin_state.collapsed_margin.flush(), 0.0);
    }

    #[test]
    fn bottom_margin_stays_pending_when_parent_can_collapse_with_last_child() {
        let box_model = box_model_with_margins(0.0, 10.0);
        let mut child_state = MarginCollapseState::default();
        child_state.collapsed_margin.add(20.0);
        let mut current_state = MarginCollapseState::default();

        let collapsed_bottom = calculate_bottom_margin(false, false, &mut child_state, &mut current_state, &box_model);

        assert_eq!(collapsed_bottom, 0.0);
        assert!(!current_state.bottom_collapsed);
        assert_eq!(current_state.collapsed_margin.flush(), 20.0);
        assert_eq!(child_state.collapsed_margin.flush(), 20.0);
    }

    #[test]
    fn bottom_margin_collapsing_combines_positive_and_negative_values() {
        let box_model = box_model_with_margins(0.0, -30.0);
        let mut child_state = MarginCollapseState::default();
        child_state.collapsed_margin.add(20.0);
        let mut current_state = MarginCollapseState::default();

        let collapsed_bottom = calculate_bottom_margin(false, false, &mut child_state, &mut current_state, &box_model);

        assert_eq!(collapsed_bottom, 0.0);
        assert_eq!(current_state.collapsed_margin.flush(), -10.0);
        assert_eq!(child_state.collapsed_margin.flush(), -10.0);
    }

    #[test]
    fn bottom_margin_flushes_child_when_parent_has_bottom_fence() {
        let box_model = box_model_with_margins(0.0, 10.0);
        let mut child_state = MarginCollapseState::default();
        child_state.collapsed_margin.add(20.0);
        let mut current_state = MarginCollapseState::default();

        let collapsed_bottom = calculate_bottom_margin(false, true, &mut child_state, &mut current_state, &box_model);

        assert_eq!(collapsed_bottom, 20.0);
        assert!(current_state.bottom_collapsed);
        assert_eq!(current_state.collapsed_margin.flush(), 10.0);
        assert_eq!(child_state.collapsed_margin.flush(), 0.0);
    }

    #[test]
    fn bottom_margin_flushes_child_when_parent_establishes_bfc() {
        let box_model = box_model_with_margins(0.0, 10.0);
        let mut child_state = MarginCollapseState::default();
        child_state.collapsed_margin.add(20.0);
        let mut current_state = MarginCollapseState::default();

        let collapsed_bottom = calculate_bottom_margin(true, false, &mut child_state, &mut current_state, &box_model);

        assert_eq!(collapsed_bottom, 20.0);
        assert!(current_state.bottom_collapsed);
        assert_eq!(current_state.collapsed_margin.flush(), 10.0);
        assert_eq!(child_state.collapsed_margin.flush(), 0.0);
    }
}
