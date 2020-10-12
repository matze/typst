//! Layouting of documents.

mod document;
mod fixed;
mod node;
mod pad;
mod par;
mod spacing;
mod stack;
mod text;

use crate::font::SharedFontLoader;
use crate::geom::*;
use crate::shaping::Shaped;

pub use document::*;
pub use fixed::*;
pub use node::*;
pub use pad::*;
pub use par::*;
pub use spacing::*;
pub use stack::*;
pub use text::*;

/// Layout a document and return the produced layouts.
pub fn layout(document: &Document, loader: SharedFontLoader) -> Vec<BoxLayout> {
    let mut ctx = LayoutContext { loader };
    document.layout(&mut ctx)
}

/// The context for layouting.
#[derive(Debug, Clone)]
pub struct LayoutContext {
    /// The font loader to query fonts from when typesetting text.
    pub loader: SharedFontLoader,
}

/// Layout a node.
pub trait Layout {
    /// Layout the node into the given areas.
    fn layout(&self, ctx: &mut LayoutContext, areas: &Areas) -> Layouted;
}

/// A sequence of areas to layout into.
#[derive(Debug, Clone, PartialEq)]
pub struct Areas {
    /// The current area.
    pub current: Area,
    /// The backlog of followup areas.
    ///
    /// _Note_: This works stack-like and not queue-like!
    pub backlog: Vec<Size>,
    /// The last area that is repeated when the backlog is empty.
    pub last: Option<Size>,
}

impl Areas {
    /// Create a new length-1 sequence of areas with just one `area`.
    pub fn once(size: Size) -> Self {
        Self {
            current: Area::new(size),
            backlog: vec![],
            last: None,
        }
    }

    /// Create a new sequence of areas that repeats `area` indefinitely.
    pub fn repeat(size: Size) -> Self {
        Self {
            current: Area::new(size),
            backlog: vec![],
            last: Some(size),
        }
    }

    /// Advance to the next area if there is any.
    pub fn next(&mut self) {
        if let Some(size) = self.backlog.pop().or(self.last) {
            self.current = Area::new(size);
        }
    }

    /// Whether `current` is a fully sized (untouched) copy of the last area.
    ///
    /// If this is false calling `next()` will have no effect.
    pub fn in_full_last(&self) -> bool {
        self.backlog.is_empty() && self.last.map_or(true, |size| self.current.rem == size)
    }
}

/// The area into which content can be laid out.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Area {
    /// The remaining size of this area.
    pub rem: Size,
    /// The full size this area once had (used for relative sizing).
    pub full: Size,
}

impl Area {
    /// Create a new area.
    pub fn new(size: Size) -> Self {
        Self { rem: size, full: size }
    }
}

/// How to determine a container's size along an axis.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Expansion {
    /// Fit the content.
    Fit,
    /// Fill the available space.
    Fill,
}

impl Expansion {
    /// Returns `Fill` if the condition is true and `Fit` otherwise.
    pub fn fill_if(condition: bool) -> Self {
        if condition { Self::Fill } else { Self::Fit }
    }
}

/// The result of [layouting] a node.
///
/// [layouting]: trait.Layout.html#method.layout
#[derive(Debug, Clone, PartialEq)]
pub enum Layouted {
    /// Spacing that should be added to the parent.
    Spacing(Length),
    /// A layout that should be added to and aligned in the parent.
    Layout(BoxLayout, Gen<Align>),
    /// Multiple layouts.
    Layouts(Vec<BoxLayout>, Gen<Align>),
}

impl Layouted {
    /// Return all layouts contained in this variant (zero, one or arbitrarily
    /// many).
    pub fn into_layouts(self) -> Vec<BoxLayout> {
        match self {
            Self::Spacing(_) => vec![],
            Self::Layout(layout, _) => vec![layout],
            Self::Layouts(layouts, _) => layouts,
        }
    }
}

/// A finished box with content at fixed positions.
#[derive(Debug, Clone, PartialEq)]
pub struct BoxLayout {
    /// The size of the box.
    pub size: Size,
    /// The elements composing this layout.
    pub elements: Vec<(Point, LayoutElement)>,
}

impl BoxLayout {
    /// Create a new empty collection.
    pub fn new(size: Size) -> Self {
        Self { size, elements: vec![] }
    }

    /// Add an element at a position.
    pub fn push(&mut self, pos: Point, element: LayoutElement) {
        self.elements.push((pos, element));
    }

    /// Add all elements of another collection, placing them relative to the
    /// given position.
    pub fn push_layout(&mut self, pos: Point, more: Self) {
        for (subpos, element) in more.elements {
            self.push(pos + subpos, element);
        }
    }
}

/// A layout element, the basic building block layouts are composed of.
#[derive(Debug, Clone, PartialEq)]
pub enum LayoutElement {
    /// Shaped text.
    Text(Shaped),
}
