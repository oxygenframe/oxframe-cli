//! A chat: messages appear one by one, yours on the right in the accent color.
//!
//! `blocks::chat_messages(view, &[(false, "Can it render on CI?"), (true, "Yes, on the CPU")]).await`

use oxframe::prelude::*;

const WIDTH: f64 = 1200.0;
const MAX_TEXT: f64 = 720.0;
const PAD: f64 = 28.0;
const STEP: f64 = 130.0;

/// Pops each bubble in, the earlier ones moving up to make room. `true`
/// marks your own messages. Returns the block's group.
pub async fn chat_messages(view: View, messages: &[(bool, &str)]) -> Group {
    let group = view.add(Group::new().name("chat_messages"));
    let feed = group.add(Group::new());
    for (i, &(mine, text)) in messages.iter().enumerate() {
        let bubble = feed.add(Group::new().name(format!("message_{i}")));
        let back = bubble.add(Rect::new().height(96.0).radius(36.0).fill(if mine {
            Accent
        } else {
            Surface
        }));
        let label = bubble.add(Txt::new().text(text).h2().max_width(MAX_TEXT).fill(if mine {
            OnAccent
        } else {
            Text
        }));
        let (w, h) = label.size();
        back.width().set(w + 2.0 * PAD);
        back.height().set(h + 2.0 * PAD * 0.8);
        let side = WIDTH / 2.0 - (w + 2.0 * PAD) / 2.0;
        bubble.x().set(if mine { side } else { -side });
        bubble.y().set(i as f64 * STEP);
        // The newest message sits a little below the center.
        all((
            feed.y().to(120.0 - i as f64 * STEP, 0.35),
            bubble.pop_in(),
        ))
        .await;
        wait_for(0.6).await;
    }
    group
}
