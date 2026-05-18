use anyhow::Result;
use headless_chrome::{Browser, LaunchOptions};
use url::Url;

// Chrome renders at 96 dpi by default. PrintToPdfOptions expects paper
// dimensions in inches, so we divide pixel measurements by 96.
const CHROME_DPI: f64 = 96.0;

pub async fn html_to_pdf(input: &str, output: &str, scale: &f64, one_page: bool, manual_breaks: bool) -> Result<()> {

    let path = std::fs::canonicalize(input)?;
    let url = Url::from_file_path(&path)
        .map_err(|_| anyhow::anyhow!("Invalid file path"))?;

    let browser = Browser::new(LaunchOptions::default())?;
    let tab = browser.new_tab()?;

    tab.navigate_to(&url.as_str())?;
    tab.wait_until_navigated()?;

    // When --one-page is set:
    // 1. Read the @page margin from the loaded template stylesheet.
    // 2. Measure the natural scrollWidth/scrollHeight of the screen layout.
    // 3. Add the margins to the measured dimensions so the paper is sized to
    //    fit the content plus its intended breathing room.
    // 4. Inject a <style> that overrides @page size (keeping the original margin)
    //    and suppresses all page breaks.
    // 5. Pass the computed dimensions as paper_width/paper_height with
    //    prefer_css_page_size: false so Chrome uses our values.
    //
    // Measuring before injecting matters: we want the screen-layout content
    // height, not the height of a single fixed print page.
    let (paper_width, paper_height, margin_top_in, margin_bottom_in, margin_left_in, margin_right_in) = if one_page {

        // Walk document.styleSheets to find the first @page rule and extract
        // its margin values in pixels. Falls back to 0 for any side not found.
        let margin_result = tab.evaluate(r#"
            (function() {
                var top = 0, bottom = 0, left = 0, right = 0;
                var PX_PER_IN = 96.0;
                function parseToPx(val) {
                    if (!val || val === '') return 0;
                    val = val.trim();
                    if (val.endsWith('px'))  return parseFloat(val);
                    if (val.endsWith('cm'))  return parseFloat(val) / 2.54 * PX_PER_IN;
                    if (val.endsWith('mm'))  return parseFloat(val) / 25.4 * PX_PER_IN;
                    if (val.endsWith('in'))  return parseFloat(val) * PX_PER_IN;
                    if (val.endsWith('pt'))  return parseFloat(val) * PX_PER_IN / 72.0;
                    return 0;
                }
                for (var i = 0; i < document.styleSheets.length; i++) {
                    var rules;
                    try { rules = document.styleSheets[i].cssRules; } catch(e) { continue; }
                    for (var j = 0; j < rules.length; j++) {
                        var r = rules[j];
                        if (r.type === CSSRule.PAGE_RULE) {
                            var s = r.style;
                            var mt = s.getPropertyValue('margin-top')    || s.getPropertyValue('margin') || '0';
                            var mb = s.getPropertyValue('margin-bottom') || s.getPropertyValue('margin') || '0';
                            var ml = s.getPropertyValue('margin-left')   || s.getPropertyValue('margin') || '0';
                            var mr = s.getPropertyValue('margin-right')  || s.getPropertyValue('margin') || '0';
                            top    = parseToPx(mt);
                            bottom = parseToPx(mb);
                            left   = parseToPx(ml);
                            right  = parseToPx(mr);
                            break;
                        }
                    }
                    if (top || bottom || left || right) break;
                }
                return JSON.stringify({ top: top, bottom: bottom, left: left, right: right });
            })()
        "#, false)?;

        let margin_raw = margin_result.value
            .as_ref()
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| r#"{"top":0,"bottom":0,"left":0,"right":0}"#.to_string());

        let margins: serde_json::Value = serde_json::from_str(&margin_raw)
            .unwrap_or(serde_json::json!({"top":0,"bottom":0,"left":0,"right":0}));

        let m_top    = margins["top"].as_f64().unwrap_or(0.0);
        let m_bottom = margins["bottom"].as_f64().unwrap_or(0.0);
        let m_left   = margins["left"].as_f64().unwrap_or(0.0);
        let m_right  = margins["right"].as_f64().unwrap_or(0.0);

        let w_result = tab.evaluate("document.body.scrollWidth", false)?;
        let h_result = tab.evaluate("document.body.scrollHeight", false)?;

        let px_w = w_result.value
            .as_ref()
            .and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow::anyhow!("Failed to read scrollWidth from Chrome"))?;

        let px_h = h_result.value
            .as_ref()
            .and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow::anyhow!("Failed to read scrollHeight from Chrome"))?;

        // Paper = content + margins on each side + rounding buffer.
        let px_w_total = px_w + m_left + m_right;
        let px_h_total = px_h + m_top  + m_bottom + 48.0;

        // Inject a @page override with our computed size, keeping the original
        // margin intact so the template's spacing is preserved.
        let inject_style = format!(
            r#"(function() {{
                var s = document.createElement('style');
                s.textContent = `
                    @page {{
                        size: {px_w_total}px {px_h_total}px !important;
                    }}
                    * {{
                        break-inside: avoid !important;
                        page-break-inside: avoid !important;
                    }}
                `;
                document.head.appendChild(s);
            }})();"#
        );
        tab.evaluate(&inject_style, false)?;

        // Convert margins from px to inches for PrintToPdfOptions.
        (
            Some(px_w_total / CHROME_DPI),
            Some(px_h_total / CHROME_DPI),
            m_top    / CHROME_DPI,
            m_bottom / CHROME_DPI,
            m_left   / CHROME_DPI,
            m_right  / CHROME_DPI,
        )
    } else {
        (None, None, 0.0, 0.0, 0.0, 0.0)
    };

    // When --manual-breaks is set:
    //
    // We want each section between break divs to produce a PDF page sized to
    // fit that section's content exactly. The approach:
    //
    // 1. Read @page margin and width from the template stylesheet.
    // 2. Find all <div style="break-after: page"> elements (section boundaries).
    // 3. Wrap the content between each pair of boundaries into a <div> and
    //    remove the original break divs.
    // 4. Measure each section's scrollHeight.
    // 5. Inject a named @page rule per section (mdpdf-page-0, mdpdf-page-1, ...)
    //    with width = template width and height = measured content + margins + buffer.
    // 6. Assign each section div page: mdpdf-page-N and break-after: page.
    //
    // prefer_css_page_size: true lets Chrome use the named @page sizes.
    // No paper_width/paper_height is passed — Chrome reads them from CSS.
    if manual_breaks {
        tab.evaluate(r#"
            (function() {
                // --- Step 1: read @page margin and width from template stylesheet ---
                var pageMarginTop = 0, pageMarginBottom = 0;
                var pageMarginLeft = 0, pageMarginRight = 0;
                var pageWidth = 'auto';
                var PX = 96.0;
                function toPx(val) {
                    if (!val || val === '') return 0;
                    val = val.trim();
                    if (val.endsWith('px')) return parseFloat(val);
                    if (val.endsWith('cm')) return parseFloat(val) / 2.54 * PX;
                    if (val.endsWith('mm')) return parseFloat(val) / 25.4 * PX;
                    if (val.endsWith('in')) return parseFloat(val) * PX;
                    if (val.endsWith('pt')) return parseFloat(val) * PX / 72.0;
                    return 0;
                }
                for (var i = 0; i < document.styleSheets.length; i++) {
                    var rules;
                    try { rules = document.styleSheets[i].cssRules; } catch(e) { continue; }
                    for (var j = 0; j < rules.length; j++) {
                        var r = rules[j];
                        if (r.type === CSSRule.PAGE_RULE) {
                            var s = r.style;
                            pageMarginTop    = toPx(s.getPropertyValue('margin-top')    || s.getPropertyValue('margin') || '0');
                            pageMarginBottom = toPx(s.getPropertyValue('margin-bottom') || s.getPropertyValue('margin') || '0');
                            pageMarginLeft   = toPx(s.getPropertyValue('margin-left')   || s.getPropertyValue('margin') || '0');
                            pageMarginRight  = toPx(s.getPropertyValue('margin-right')  || s.getPropertyValue('margin') || '0');
                            var sz = s.getPropertyValue('size');
                            if (sz) pageWidth = sz.trim().split(/\s+/)[0];
                            break;
                        }
                    }
                    if (pageWidth !== 'auto') break;
                }

                // --- Step 2: collect body children and locate break divs ---
                var body = document.body;
                var children = Array.from(body.childNodes);

                // Identify break div indices
                function isBreakDiv(node) {
                    return node.nodeType === 1 &&
                           node.tagName === 'DIV' &&
                           (node.style.breakAfter === 'page' ||
                            node.style.pageBreakAfter === 'always' ||
                            (node.getAttribute('style') || '').indexOf('break-after') !== -1);
                }

                // Split children into sections at each break div
                var sections = [];
                var current = [];
                for (var k = 0; k < children.length; k++) {
                    if (isBreakDiv(children[k])) {
                        sections.push(current);
                        current = [];
                    } else {
                        current.push(children[k]);
                    }
                }
                sections.push(current); // last section (no trailing break div needed)

                // --- Step 3: wrap each section in a div and replace body contents ---
                body.innerHTML = '';
                var wrappers = [];
                for (var s = 0; s < sections.length; s++) {
                    var wrapper = document.createElement('div');
                    wrapper.className = 'mdpdf-section';
                    for (var n = 0; n < sections[s].length; n++) {
                        wrapper.appendChild(sections[s][n]);
                    }
                    body.appendChild(wrapper);
                    wrappers.push(wrapper);
                }

                // --- Step 4 & 5: measure each section and build named @page rules ---
                var cssRules = '';
                for (var w = 0; w < wrappers.length; w++) {
                    var h = wrappers[w].scrollHeight + pageMarginTop + pageMarginBottom + 48;
                    var pageName = 'mdpdf-page-' + w;
                    cssRules += '@page ' + pageName + ' { size: ' + pageWidth + ' ' + h + 'px; ' +
                        'margin-top: ' + pageMarginTop + 'px; ' +
                        'margin-bottom: ' + pageMarginBottom + 'px; ' +
                        'margin-left: ' + pageMarginLeft + 'px; ' +
                        'margin-right: ' + pageMarginRight + 'px; }\n';
                    wrappers[w].style.page = pageName;
                }

                // --- Step 6: inject styles ---
                var styleEl = document.createElement('style');
                styleEl.textContent = cssRules +
                    'body { margin: 0 !important; padding: 0 !important; }\n' +
                    '.mdpdf-section { break-after: page; page-break-after: always; }\n' +
                    '.mdpdf-section:last-child { break-after: avoid; page-break-after: avoid; }\n';
                document.head.appendChild(styleEl);
            })();
        "#, false)?;
    }

    let options = headless_chrome::types::PrintToPdfOptions {
        margin_top:    Some(if one_page { margin_top_in }    else { 0.0 }),
        margin_bottom: Some(if one_page { margin_bottom_in } else { 0.0 }),
        margin_left:   Some(if one_page { margin_left_in }   else { 0.0 }),
        margin_right:  Some(if one_page { margin_right_in }  else { 0.0 }),
        generate_document_outline: Some(true),
        // When one_page is active we supply exact paper dimensions measured
        // from the rendered content. prefer_css_page_size must be false in
        // that case, otherwise Chrome ignores paper_width/paper_height and
        // uses the @page { size: ... } rule from the template CSS instead.
        prefer_css_page_size: Some(!one_page),
        display_header_footer: Some(false),
        print_background: Some(true),
        scale: Some(*scale),
        paper_width,
        paper_height,
        ..Default::default()
    };

    let pdf = tab.print_to_pdf(Option::Some(options))?;

    std::fs::write(output, pdf)?;

    Ok(())
}