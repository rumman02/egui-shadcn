//! shadcn `message.tsx` — the message layout scaffolding around chat
//! bubbles.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/message.tsx`: a [`Message`]
//! row (`w-full gap-2 text-sm`, reversed for `align="end"`) holding a
//! bottom-aligned [`MessageAvatar`] slot (min-w-8, hosting an `Avatar`) and
//! a full-width [`MessageContent`] column (`gap-2.5`) of bubbles,
//! [`MessageHeader`]s and [`MessageFooter`]s (`text-xs text-muted-foreground
//! px-3`), grouped by [`MessageGroup`] (`flex-col gap-2`).
//!
//! This source version exports no `MessageActions`; actions rows are part of
//! a different message component revision, so none are ported here.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! MessageGroup::new().show(ui, |ui| {
//!     Message::new().align(MessageAlign::End).show(ui, |ui| {
//!         MessageAvatar::new().show(ui, |ui| {
//!             Avatar::new().fallback("ME").show(ui);
//!         });
//!         MessageContent::new().show(ui, |ui| {
//!             Bubble::new().show(ui, |ui| {
//!                 BubbleContent::new("Deploying to prod real quick.").show(ui);
//!             });
//!             MessageFooter::new("Delivered").show(ui);
//!         });
//!     });
//! });
//! ```

use egui::{Align, Id, Layout, Ui, Vec2};

/// Horizontal alignment of the message row, mirroring the source `align`
/// prop (`start` = avatar first on the left; `end` = reversed row, avatar on
/// the right, content right-aligned).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MessageAlign {
    /// Assistant/other-party side (the source default).
    #[default]
    Start,
    /// Current-user side (`flex-row-reverse` + right-aligned content).
    End,
}

/// Layout context pushed by [`Message`] for its slot children.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ScopeState {
    pub align: MessageAlign,
}

fn scope_stack_id() -> Id {
    Id::unique("shadcn_message_scope_stack")
}

pub(crate) mod scope {
    use super::{scope_stack_id, ScopeState};
    use egui::Ui;

    pub(crate) fn push(ui: &Ui, state: ScopeState) {
        let mut stack: Vec<ScopeState> = ui
            .ctx()
            .data_mut(|d| d.get_temp(scope_stack_id()))
            .unwrap_or_default();
        stack.push(state);
        ui.ctx()
            .data_mut(|d| d.insert_temp(scope_stack_id(), stack));
    }

    pub(crate) fn pop(ui: &Ui) {
        let mut stack: Vec<ScopeState> = ui
            .ctx()
            .data_mut(|d| d.get_temp(scope_stack_id()))
            .unwrap_or_default();
        stack.pop();
        ui.ctx()
            .data_mut(|d| d.insert_temp(scope_stack_id(), stack));
    }

    pub(crate) fn current(ui: &Ui) -> Option<ScopeState> {
        ui.ctx()
            .data_mut(|d| d.get_temp::<Vec<ScopeState>>(scope_stack_id()))
            .and_then(|s| s.last().copied())
    }
}

/// `MessageGroup` — a `flex-col gap-2` stack of [`Message`]s.
#[derive(Clone, Debug, Default)]
pub struct MessageGroup {
    id_salt: Option<String>,
}

impl MessageGroup {
    /// Creates a message group.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt (MessageGroup keeps no state).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the group around its messages.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        ui.with_layout(Layout::top_down(Align::Min), |ui| {
            ui.spacing_mut().item_spacing.y = 8.0; // gap-2
            content(ui)
        })
    }
}

/// `Message` — the `w-full gap-2 text-sm` row. Children are conventionally a
/// [`MessageAvatar`] followed by a [`MessageContent`]; `align="end"` renders
/// the row right-to-left so the avatar lands on the right.
#[derive(Clone, Debug, Default)]
pub struct Message {
    align: MessageAlign,
    id_salt: Option<String>,
}

impl Message {
    /// Creates a start-aligned message row.
    pub fn new() -> Self {
        Self {
            align: MessageAlign::default(),
            id_salt: None,
        }
    }

    /// Sets the row alignment.
    pub fn align(mut self, align: MessageAlign) -> Self {
        self.align = align;
        self
    }

    /// Sets a stable id salt (Message keeps no state).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the message row around its slots.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        scope::push(ui, ScopeState { align: self.align });
        // `data-[align=end]:flex-row-reverse` — flip the row direction and
        // bottom-align the slots (`MessageAvatar` is `self-end`).
        let layout = match self.align {
            MessageAlign::Start => Layout::left_to_right(Align::Max),
            MessageAlign::End => Layout::right_to_left(Align::Max),
        };
        let inner = ui.with_layout(layout, |ui| {
            ui.spacing_mut().item_spacing.x = 8.0; // gap-2
            content(ui)
        });
        scope::pop(ui);
        inner
    }
}

/// `MessageAvatar` — the bottom-aligned `min-w-8` avatar slot. Hosts an
/// [`Avatar`](crate::components::avatar::Avatar) (or any 32px-ish widget).
#[derive(Clone, Debug, Default)]
pub struct MessageAvatar {
    size: f32,
    id_salt: Option<String>,
}

impl MessageAvatar {
    /// Creates a 32px (`min-w-8`) avatar slot.
    pub fn new() -> Self {
        Self {
            size: crate::theme::tokens::spacing::CONTROL_SM,
            id_salt: None,
        }
    }

    /// Overrides the slot side length.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Sets a stable id salt (MessageAvatar keeps no state).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the avatar slot around its contents.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        // `shrink-0 self-end justify-center overflow-hidden rounded-full
        // bg-muted` — the child (usually an Avatar) fills the square slot.
        ui.allocate_ui_with_layout(
            Vec2::splat(self.size),
            Layout::left_to_right(Align::Center),
            |ui| {
                ui.set_min_size(Vec2::splat(self.size));
                ui.set_max_size(Vec2::splat(self.size));
                content(ui)
            },
        )
    }
}

/// `MessageContent` — the full-width `flex-col gap-2.5` column of bubbles
/// and header/footer rows; children right-align when the message is
/// `align="end"`.
#[derive(Clone, Debug, Default)]
pub struct MessageContent {
    id_salt: Option<String>,
}

impl MessageContent {
    /// Creates a content column.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt (MessageContent keeps no state).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the content column around its contents.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        let align = scope::current(ui).map(|s| s.align).unwrap_or_default();
        let align_flag = match align {
            MessageAlign::Start => Align::Min,
            MessageAlign::End => Align::Max,
        };
        let in_message = scope::current(ui).is_some();
        ui.with_layout(Layout::top_down(align_flag), |ui| {
            // `w-full` — span the row's remaining width so `align="end"`
            // pushes bubbles to the right edge.
            if in_message {
                ui.set_min_width(ui.available_width());
            }
            ui.spacing_mut().item_spacing.y = 10.0; // gap-2.5
            content(ui)
        })
    }
}

/// `MessageHeader` — the `text-xs font-medium text-muted-foreground px-3`
/// row above the bubbles. The source strips the horizontal padding when the
/// message contains a ghost bubble; use [`MessageHeader::flush`] to opt out
/// (immediate-mode layout can't retroactively detect sibling variants).
#[derive(Clone, Debug)]
pub struct MessageHeader {
    text: String,
    flush: bool,
    id_salt: Option<String>,
}

impl MessageHeader {
    /// Creates a header row.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            flush: false,
            id_salt: None,
        }
    }

    /// Drops the `px-3` padding (`group-has-ghost:px-0`).
    pub fn flush(mut self, flush: bool) -> Self {
        self.flush = flush;
        self
    }

    /// Sets a stable id salt (MessageHeader keeps no state).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the header row.
    pub fn show(self, ui: &mut Ui) -> egui::Response {
        show_message_row_text(ui, &self.text, self.flush)
    }
}

/// `MessageFooter` — the `text-xs font-medium text-muted-foreground px-3`
/// row below the bubbles (`justify-end` when the message is `align="end"`).
#[derive(Clone, Debug)]
pub struct MessageFooter {
    text: String,
    flush: bool,
    id_salt: Option<String>,
}

impl MessageFooter {
    /// Creates a footer row.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            flush: false,
            id_salt: None,
        }
    }

    /// Drops the `px-3` padding (`group-has-ghost:px-0`).
    pub fn flush(mut self, flush: bool) -> Self {
        self.flush = flush;
        self
    }

    /// Sets a stable id salt (MessageFooter keeps no state).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the footer row.
    pub fn show(self, ui: &mut Ui) -> egui::Response {
        show_message_row_text(ui, &self.text, self.flush)
    }
}

/// Shared painter for [`MessageHeader`] / [`MessageFooter`].
fn show_message_row_text(ui: &mut Ui, text: &str, flush: bool) -> egui::Response {
    let theme = crate::theme::ShadcnTheme::of_ui(ui);
    let p = theme.palette;
    let align = scope::current(ui).map(|s| s.align).unwrap_or_default();
    // `justify-end` for end-aligned messages (footer), left otherwise.
    let layout = match align {
        MessageAlign::Start => Layout::left_to_right(Align::Min),
        MessageAlign::End => Layout::right_to_left(Align::Min),
    };
    ui.with_layout(layout, |ui| {
        ui.add_space(if flush { 0.0 } else { 12.0 }); // px-3
        ui.label(
            egui::RichText::new(text)
                .size(crate::theme::tokens::type_scale::XS)
                .color(p.muted_foreground),
        );
    })
    .response
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{avatar::Avatar, bubble::Bubble, bubble::BubbleContent};
    use crate::theme::{Mode, Preset};
    use egui::{Pos2, Rect};

    #[test]
    fn default_align_is_start() {
        assert_eq!(MessageAlign::default(), MessageAlign::Start);
        assert_eq!(Message::new().align, MessageAlign::Start);
        assert_eq!(
            Message::new().align(MessageAlign::End).align,
            MessageAlign::End
        );
    }

    #[test]
    fn avatar_slot_is_32px() {
        assert_eq!(MessageAvatar::new().size, 32.0);
        assert_eq!(MessageAvatar::new().size(40.0).size, 40.0);
    }

    #[test]
    fn renders_in_light_and_dark() {
        for mode in [Mode::Light, Mode::Dark] {
            let ctx = egui::Context::default();
            Preset::default().build(mode).install(&ctx);
            let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    Id::unique("message_render_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 700.0))),
                );
                MessageGroup::new().show(ui, |ui| {
                    Message::new().show(ui, |ui| {
                        MessageAvatar::new().show(ui, |ui| {
                            Avatar::new().fallback("RB").show(ui);
                        });
                        MessageContent::new().show(ui, |ui| {
                            MessageHeader::new("Rabbit").show(ui);
                            Bubble::new().show(ui, |ui| {
                                BubbleContent::new("It's 4:55 PM. On a Friday.").show(ui);
                            });
                            MessageFooter::new("9:55 PM").show(ui);
                        });
                    });
                    Message::new().align(MessageAlign::End).show(ui, |ui| {
                        MessageAvatar::new().show(ui, |ui| {
                            Avatar::new().fallback("ME").show(ui);
                        });
                        MessageContent::new().show(ui, |ui| {
                            Bubble::new().show(ui, |ui| {
                                BubbleContent::new("Deploying to prod real quick.").show(ui);
                            });
                            MessageFooter::new("Delivered").show(ui);
                        });
                    });
                });
                // Standalone footer flush variant.
                MessageFooter::new("flush").flush(true).show(ui);
            });
            out.textures_delta.clear();
        }
    }
}
