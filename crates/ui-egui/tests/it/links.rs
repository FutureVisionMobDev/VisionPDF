//! The app UI does not expose repository hosting links.

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use visionpdf_ui_egui::{Dialog, VisionPDFApp};

fn harness(setup: impl FnOnce(&mut VisionPDFApp) + 'static) -> Harness<'static, VisionPDFApp> {
    let mut h = Harness::builder().with_size(egui::vec2(1400.0, 900.0)).build_eframe(move |_cc| {
        let mut app = VisionPDFApp::new();
        app.set_option("language", "en").unwrap();
        setup(&mut app);
        app
    });
    h.run_steps(4);
    h
}

#[test]
fn project_link_is_not_in_the_top_bar_or_help_menu() {
    let h = harness(|_| {});
    assert!(h.query_by_label("GitHub").is_none());
    assert!(visionpdf_engine::commands::command("help.github").is_none());
    assert!(visionpdf_engine::commands::command("help.discord").is_none());
    assert!(visionpdf_engine::commands::command("help.app_page").is_none());
}

#[test]
fn home_has_no_source_promotion_card() {
    let h = harness(|_| {});
    assert!(h.query_by_label("VisionPDF is open source").is_none());
    assert!(h.query_by_label("Get VisionPDF support").is_none());
    assert!(h.query_by_label("VisionPDF downloads").is_none());
}

#[test]
fn about_dialog_shows_visionpdf_branding_without_repository_links() {
    let pdf = b"%PDF-1.7\n1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj\n2 0 obj << /Type /Pages /Kids [3 0 R] /Count 1 >> endobj\n3 0 obj << /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] >> endobj\ntrailer << /Root 1 0 R >>\n%%EOF";
    // With a document open, so the home screen's own links are not on screen.
    let mut h = harness(move |app| {
        app.open_bytes("one.pdf", None, pdf.to_vec()).unwrap();
        app.dialog = Some(Dialog::About);
    });
    assert!(h.query_all_by_label("VisionPDF").count() >= 2, "the mark and the wordmark (alt text)");
    assert!(h.query_by_label("Get VisionPDF support").is_none());
    assert!(h.query_by_label("VisionPDF downloads").is_none());
}

/// Runs frames until the node labelled `label` exists and has stopped moving, then returns it.
///
/// The About dialog is a modal centred on its own size, and switching tabs changes that size, so
/// for a few frames afterwards the modal grows and recentres and every control in it moves. A
/// fixed number of frames is a guess at how long that takes: it was too few after the 0.5.0
/// contributors refresh (78 names instead of 5) on some machines, the click on Table landed where
/// the button had been, and the table never opened (#671).
fn settled<'a>(h: &'a mut Harness<'static, VisionPDFApp>, label: &'a str) -> egui_kittest::Node<'a> {
    let mut last = None;
    for _ in 0..60 {
        h.run_steps(1);
        let now = h.query_by_label(label).map(|n| n.rect());
        if now.is_some() && now == last {
            return h.get_by_label(label);
        }
        last = now;
    }
    panic!("{label:?} never appeared or never stopped moving");
}

#[test]
fn about_dialog_has_contributors_and_models_tabs() {
    let mut h = harness(|app| app.dialog = Some(Dialog::About));
    settled(&mut h, "Contributors").click();
    // The owner is always in the compiled-in credits (contributors/contributors.json), shown by username.
    settled(&mut h, "@echelon");
    settled(&mut h, "Table").click();
    settled(&mut h, "PRs");
    settled(&mut h, "Display name").click();
    settled(&mut h, "Brandon Thomas");
    settled(&mut h, "Models").click();
    h.run_steps(4);
    assert!(h.query_all_by_label("Anthropic").count() >= 1);
}
