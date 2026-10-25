//! A keyboard shortcut: the keys light up one after another, then what it does.
//!
//! `blocks::key_combo(view, &["Ctrl", "Shift", "P"], "Open the command palette").await`

use oxframe::prelude::*;

const KEY_H: f64 = 120.0;
const GAP: f64 = 70.0;

/// Shows the keys, presses them in order (each turns the accent color) and
/// fades in `action` under them. Returns the block's group.
pub async fn key_combo(view: View, keys: &[&str], action: &str) -> Group {
    let group = view.add(Group::new().name("key_combo"));
    let widths: Vec<f64> = keys
        .iter()
        .map(|k| (k.chars().count() as f64 * 34.0 + 70.0).max(KEY_H))
        .collect();
    let total = widths.iter().sum::<f64>() + GAP * keys.len().saturating_sub(1) as f64;
    let mut x = -total / 2.0;
    let mut caps = Vec::new();
    for (i, (key, w)) in keys.iter().zip(&widths).enumerate() {
        let center = x + w / 2.0;
        let cap = group.add(Group::new().x(center).y(-40.0).opacity(0.0).name(format!("key_{i}")));
        let back = cap.add(Rect::new().width(*w).height(KEY_H).radius(20.0).fill(Surface));
        let label = cap.add(Txt::new().text(*key).h2().weight(700));
        if i + 1 < keys.len() {
            let _ = group.add(
                Txt::new()
                    .text("+")
                    .h2()
                    .fill(Muted)
                    .x(x + w + GAP / 2.0)
                    .y(-40.0),
            );
        }
        caps.push((cap, back, label));
        x += w + GAP;
    }
    let shown: Vec<_> = caps.iter().map(|c| c.0.clone()).collect();
    stagger(0.12, &shown, |c| c.slide_in(Side::Bottom)).await;
    wait_for(0.3).await;
    for (_, back, label) in &caps {
        all((back.fill().to(Accent, 0.15), label.fill().to(OnAccent, 0.15))).await;
    }
    let text = group.add(
        Txt::new()
            .text(action)
            .h2()
            .fill(Muted)
            .max_width(1400.0)
            .y(110.0)
            .opacity(0.0)
            .name("key_combo_action"),
    );
    text.fade_in().await;
    group
}
