use css_values::cursor::Cursor as CssCursor;
use html_dom::{DocumentRoot, HtmlTag, Tag};
use iced::{
    Rectangle,
    keyboard::{Key, key::Named},
    mouse::{self, Cursor, Interaction},
    widget::{Action, shader::Program},
};
use layout::LayoutTree;
use renderer::{ImageRenderInfo, RenderRect, RenderTri, TextBlockInfo};

use crate::{
    core::{ScrollOffset, WindowType},
    events::{Event, browser::BrowserEvent, devtools::DevtoolEvent, kernel::EngineRequest},
    renderer::primitives::HtmlPrimitive,
};

/// State for the shader widget
#[derive(Debug, Default)]
pub struct HtmlState {
    pub holding_shift: bool,
}

/// HTML/CSS renderer using wgpu
#[derive(Debug, Clone)]
pub struct HtmlRenderer<'a> {
    /// Rectangles to render (populated by layout engine)
    pub rects: Vec<RenderRect>,

    /// Triangles to render (populated by layout engine)
    pub tris: Vec<RenderTri>,

    /// Text blocks to render (populated by layout engine)
    pub text_blocks: Vec<TextBlockInfo>,

    /// Images to render (populated by layout engine + image cache)
    pub images: Vec<ImageRenderInfo>,

    /// The DOM tree being rendered
    dom_tree: &'a DocumentRoot,

    /// The layout tree being rendered
    layout_tree: &'a LayoutTree,

    /// Current scroll offset for rendering and hit testing
    scroll_offset: ScrollOffset,

    /// Where wheel scroll events should be routed.
    window_type: WindowType,
}

impl<'html> HtmlRenderer<'html> {
    pub fn new(
        dom_tree: &'html DocumentRoot,
        layout_tree: &'html LayoutTree,
        scroll_offset: ScrollOffset,
        window_type: WindowType,
    ) -> Self {
        Self {
            rects: Vec::with_capacity(1000),
            tris: Vec::with_capacity(1000),
            text_blocks: Vec::with_capacity(1000),
            images: Vec::with_capacity(100),
            dom_tree,
            layout_tree,
            scroll_offset,
            window_type,
        }
    }

    /// Determine if the cursor is hovering over a link and return its href if so.
    fn get_hovered_href(&self, cursor: iced::advanced::mouse::Cursor, bounds: Rectangle) -> Option<String> {
        let position = cursor.position()?;
        let x = position.x + self.scroll_offset.x - bounds.x;
        let y = position.y + self.scroll_offset.y - bounds.y;

        let nodes = self.layout_tree.resolve(x, y);

        for node in nodes {
            let dom_node = if let Some(dom_node) = self.dom_tree.get_node(&node.node_id) {
                dom_node
            } else {
                continue;
            };

            if let Some(n) = dom_node.data.as_element()
                && n.tag == Tag::Html(HtmlTag::A)
            {
                return Some(n.attributes.get("href").cloned().unwrap_or_default());
            }

            for ancestor in self.dom_tree.ancestors(&node.node_id) {
                if let Some(n) = ancestor.data.as_element()
                    && n.tag == Tag::Html(HtmlTag::A)
                    && let Some(href) = n.attributes.get("href")
                {
                    return Some(href.clone());
                }
            }
        }
        None
    }

    /// Determine the mouse cursor interaction based on the layout nodes under the cursor position.
    fn hovered_cursor(&self, cursor: iced::advanced::mouse::Cursor, bounds: Rectangle) -> Option<Interaction> {
        let position = cursor.position()?;
        let x = position.x + self.scroll_offset.x - bounds.x;
        let y = position.y + self.scroll_offset.y - bounds.y;

        let nodes = self.layout_tree.resolve(x, y);

        for node in nodes {
            match node.cursor {
                CssCursor::Alias => {
                    return Some(Interaction::Alias);
                }
                CssCursor::AllScroll => {
                    return Some(Interaction::AllScroll);
                }
                CssCursor::Auto => continue,
                CssCursor::Cell => {
                    return Some(Interaction::Cell);
                }
                CssCursor::ColResize => {
                    return Some(Interaction::ResizingColumn);
                }
                CssCursor::ContextMenu => {
                    return Some(Interaction::ContextMenu);
                }
                CssCursor::Copy => {
                    return Some(Interaction::Copy);
                }
                CssCursor::Crosshair => {
                    return Some(Interaction::Crosshair);
                }
                CssCursor::Default => {
                    return Some(Interaction::None);
                }
                CssCursor::EResize => {
                    return Some(Interaction::ResizingHorizontally);
                }
                CssCursor::EwResize => {
                    return Some(Interaction::ResizingHorizontally);
                }
                CssCursor::Grab => {
                    return Some(Interaction::Grab);
                }
                CssCursor::Grabbing => {
                    return Some(Interaction::Grabbing);
                }
                CssCursor::Help => {
                    return Some(Interaction::Help);
                }
                CssCursor::Move => {
                    return Some(Interaction::Move);
                }
                CssCursor::NResize => {
                    return Some(Interaction::ResizingVertically);
                }
                CssCursor::NeResize => {
                    return Some(Interaction::ResizingDiagonallyUp);
                }
                CssCursor::NeswResize => {
                    return Some(Interaction::ResizingDiagonallyDown);
                }
                CssCursor::NoDrop => {
                    return Some(Interaction::NoDrop);
                }
                CssCursor::None => {
                    return Some(Interaction::None);
                }
                CssCursor::NotAllowed => {
                    return Some(Interaction::NotAllowed);
                }
                CssCursor::NsResize => {
                    return Some(Interaction::ResizingVertically);
                }
                CssCursor::NwResize => {
                    return Some(Interaction::ResizingDiagonallyDown);
                }
                CssCursor::NwseResize => {
                    return Some(Interaction::ResizingDiagonallyUp);
                }
                CssCursor::Pointer => {
                    return Some(Interaction::Pointer);
                }
                CssCursor::Progress => {
                    return Some(Interaction::Progress);
                }
                CssCursor::RowResize => {
                    return Some(Interaction::ResizingRow);
                }
                CssCursor::SResize => {
                    return Some(Interaction::ResizingVertically);
                }
                CssCursor::SeResize => {
                    return Some(Interaction::ResizingDiagonallyUp);
                }
                CssCursor::SwResize => {
                    return Some(Interaction::ResizingDiagonallyDown);
                }
                CssCursor::Text => {
                    return Some(Interaction::Text);
                }
                CssCursor::VerticalText => {
                    return Some(Interaction::Text);
                }
                CssCursor::WResize => {
                    return Some(Interaction::ResizingHorizontally);
                }
                CssCursor::Wait => {
                    return Some(Interaction::Wait);
                }
                CssCursor::ZoomIn => {
                    return Some(Interaction::ZoomIn);
                }
                CssCursor::ZoomOut => {
                    return Some(Interaction::ZoomOut);
                }
            }
        }

        None
    }
}

impl<'renderer> Program<Event> for HtmlRenderer<'renderer> {
    type Primitive = HtmlPrimitive;
    type State = HtmlState;

    fn draw(&self, _state: &Self::State, _cursor: Cursor, _bounds: Rectangle) -> Self::Primitive {
        let mut primitive = HtmlPrimitive::new(self.scroll_offset);

        for tri in &self.tris {
            primitive.push_triangle(tri.p0, tri.p1, tri.p2, tri.color);
        }

        for render_rect in &self.rects {
            primitive.push_rect(render_rect.rect, render_rect.background);
        }

        for text_block in &self.text_blocks {
            primitive.push_text_block(text_block.clone());
        }

        for image in &self.images {
            primitive.push_image(image.clone());
        }

        primitive
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: &iced::Event,
        bounds: Rectangle,
        cursor: iced::advanced::mouse::Cursor,
    ) -> Option<iced::widget::Action<Event>> {
        if let iced::Event::Keyboard(ke) = event {
            if let iced::keyboard::Event::KeyPressed { key, .. } = ke
                && key == &Key::Named(Named::Shift)
            {
                state.holding_shift = true;
            } else if let iced::keyboard::Event::KeyReleased { key, .. } = ke
                && key == &Key::Named(Named::Shift)
            {
                state.holding_shift = false;
            }
        }

        if let iced::Event::Mouse(e) = event
            && let mouse::Event::WheelScrolled { delta } = e
        {
            if state.holding_shift {
                let delta = match *delta {
                    mouse::ScrollDelta::Lines { y, .. } => -iced::Vector::new(y, 0.0) * 60.0,
                    mouse::ScrollDelta::Pixels { y, .. } => -iced::Vector::new(y, 0.0),
                };

                let max_scroll_x = (self.layout_tree.content_width - bounds.width).max(0.0);
                let new_x = (self.scroll_offset.x + delta.x).clamp(0.0, max_scroll_x);

                if (new_x - self.scroll_offset.x).abs() > f32::EPSILON {
                    let event = match self.window_type {
                        WindowType::Browser => Event::Browser(BrowserEvent::Scroll(new_x, self.scroll_offset.y)),
                        WindowType::Devtools => Event::Devtools(DevtoolEvent::Scroll(new_x, self.scroll_offset.y)),
                    };

                    return Some(Action::publish(event));
                }
            } else {
                let delta = match *delta {
                    mouse::ScrollDelta::Lines { y, .. } => -iced::Vector::new(0.0, y) * 60.0,
                    mouse::ScrollDelta::Pixels { y, .. } => -iced::Vector::new(0.0, y),
                };

                let max_scroll_y = (self.layout_tree.content_height - bounds.height).max(0.0);
                let new_y = (self.scroll_offset.y + delta.y).clamp(0.0, max_scroll_y);

                if (new_y - self.scroll_offset.y).abs() > f32::EPSILON {
                    let event = match self.window_type {
                        WindowType::Browser => Event::Browser(BrowserEvent::Scroll(self.scroll_offset.x, new_y)),
                        WindowType::Devtools => Event::Devtools(DevtoolEvent::Scroll(self.scroll_offset.x, new_y)),
                    };

                    return Some(Action::publish(event));
                }
            }
        }

        if matches!(self.window_type, WindowType::Browser)
            && let Some(href) = self.get_hovered_href(cursor, bounds)
            && let iced::Event::Mouse(e) = event
            && let mouse::Event::ButtonReleased(mouse::Button::Left) = e
        {
            return Some(Action::publish(Event::EngineRequest(EngineRequest::NavigateTo(href))));
        }

        None
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        bounds: Rectangle,
        cursor: iced::advanced::mouse::Cursor,
    ) -> iced::advanced::mouse::Interaction {
        if !matches!(self.window_type, WindowType::Browser) {
            return Interaction::default();
        }
        self.hovered_cursor(cursor, bounds).unwrap_or_default()
    }
}
