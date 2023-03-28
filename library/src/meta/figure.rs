use std::str::FromStr;

use super::{Counter, LocalName, Numbering, NumberingPattern, RefAnchor, Supplement};
use crate::layout::{BlockElem, VElem};
use crate::prelude::*;
use crate::text::TextElem;

/// A anchor to be referenced.
///
/// Display: Anchor
/// Category: meta
#[element(Locatable, Synthesize, Show)]
pub struct AnchorElem {
    #[required]
    pub counter: Counter,

    #[default(NonZeroUsize::ONE)]
    pub level: NonZeroUsize,

    #[required]
    pub supplement: Option<Content>,

    #[required]
    pub numbering: Option<Numbering>,
}

impl Synthesize for AnchorElem {
    fn synthesize(&mut self, _styles: StyleChain) {
        self.push_numbering(self.numbering())
    }
}

impl Show for AnchorElem {
    fn show(&self, _vt: &mut Vt, styles: StyleChain) -> SourceResult<Content> {
        let mut content = Content::empty();

        if let Some(supplement) = self.supplement() {
            content += supplement + TextElem::packed('\u{a0}')
        }

        content += self
            .numbering()
            .map(|numbering| {
                self.counter().update(super::CounterUpdate::Step(self.level(styles)))
                    + self.counter().display(Some(numbering), false)
            })
            .unwrap_or_default();

        Ok(content)
    }
}

/// A figure with an optional caption.
///
/// ## Example
/// ```example
/// = Pipeline
/// @lab shows the central step of
/// our molecular testing pipeline.
///
/// #figure(
///   image("molecular.jpg", width: 80%),
///   caption: [
///     The molecular testing pipeline.
///   ],
/// ) <lab>
/// ```
///
/// Display: Figure
/// Category: meta
#[element(Locatable, Synthesize, Show, LocalName, RefAnchor)]
pub struct FigureElem {
    /// The content of the figure. Often, an [image]($func/image).
    #[required]
    pub body: Content,

    /// Supplement prefix text in the caption.
    pub supplement: Smart<Option<Supplement>>,

    /// Counter used in this figure for numbering.
    #[default(Counter::of(Self::func()))]
    pub counter: Counter,

    /// How to number the figure. Accepts a
    /// [numbering pattern or function]($func/numbering).
    #[default(Some(NumberingPattern::from_str("1").unwrap().into()))]
    pub numbering: Option<Numbering>,

    /// The figure's caption.
    #[default(Some(TextElem::packed(": ")))]
    pub sep: Option<Content>,

    /// Caption of this figure.
    pub caption: Option<Content>,

    /// The vertical gap between the body and caption.
    #[default(Em::new(0.65).into())]
    pub gap: Length,
}

impl Synthesize for FigureElem {
    fn synthesize(&mut self, styles: StyleChain) {
        self.push_numbering(self.numbering(styles));
    }
}

impl Show for FigureElem {
    fn show(&self, vt: &mut Vt, styles: StyleChain) -> SourceResult<Content> {
        let mut realized = self.body();

        let mut cap = Content::empty();

        if self.numbering(styles).is_some() {
            cap += self.anchor(vt, styles)?.show(vt, styles)?;
        }

        if let Some(caption) = self.caption(styles) {
            if !cap.is_empty() {
                cap += self.sep(styles).unwrap_or_default();
            }
            cap += caption
        }

        if !cap.is_empty() {
            realized += VElem::weak(self.gap(styles).into()).pack();
            realized += cap;
        }

        Ok(BlockElem::new()
            .with_body(Some(realized))
            .with_breakable(false)
            .pack()
            .aligned(Axes::with_x(Some(Align::Center.into()))))
    }
}

impl LocalName for FigureElem {
    fn local_name(&self, lang: Lang) -> &'static str {
        match lang {
            Lang::GERMAN => "Abbildung",
            Lang::ENGLISH | _ => "Figure",
        }
    }
}

impl RefAnchor for FigureElem {
    fn anchor(&self, vt: &mut Vt, styles: StyleChain) -> SourceResult<AnchorElem> {
        let supplement = Supplement::resolve(self.supplement(styles), vt, self, styles)?;
        Ok(AnchorElem::new(self.counter(styles), supplement, self.numbering(styles)))
    }
}
