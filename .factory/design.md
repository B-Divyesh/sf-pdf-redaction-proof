# Redaction Proof — visual thesis

## Direction: generative geometry / forensic aperture

Redaction is a spatial security problem: visible planes can conceal recoverable
planes. The interface uses a field of precisely offset rectangles, clipped
scan-lines, and a single circular “aperture” motif to make those layers
legible. Geometry is explanatory rather than decorative: shifted layers mean
recoverable content; aligned, sealed layers mean a verified output. The result
should feel like a quiet forensic instrument, not a broad PDF editor or a
generic security dashboard.

## Palette

- **Ink** `#171714` — warm near-black, like toner; primary text.
- **Paper** `#F4F1E8` — the global light canvas, warmer and less clinical than
  stock white.
- **Raised paper** `#FFFEF8` — work surfaces.
- **Muted graphite** `#69675F` — explanatory text (tested above 4.5:1 on paper).
- **Signal vermilion** `#C43B22` — the sole primary action/accent, taken from
  physical redaction markup; white is not used as small text on it.
- **Proof blue** `#135E75` — verified/neutral technical marks.
- **Pass green** `#246B4B`, **warning ochre** `#866100`, **danger red**
  `#A93425` — each paired with iconography and labels, never color alone.
- Dark treatment: `#141714` canvas, `#20241F` surface, `#F3EFE3` text,
  `#B8B7AE` muted, with the same vermilion and a brighter `#6AB2C7` proof blue.

The app follows the operating-system color preference. Contrast targets are
4.5:1 for text and 3:1 for focus/UI boundaries in both modes.

## Type

No webfont is loaded. The display voice is **Georgia** (editorial, document
native); the working voice is the local **system UI stack**, and evidence
values use the local monospace stack. This avoids a network request, makes the
utility feel native, and keeps the initial payload small. Scale: 48/38, 32,
24, 20, 17, 14 px. Body text is at least 17 px in the app.

## Space and shape

An 8 px base rhythm with 4 px optical adjustments. Main measures are 1120 px
on the site and 1040 px in the app. Corners are restrained (4, 8, 16 px):
documents are rectilinear; the aperture/status stamps are circular. Fine
1 px rules use ink at low opacity. Touch targets are at least 44 px.

## Interaction grammar

The primary action is always vermilion on paper. Drop/pick zones behave like a
physical inspection tray. Findings unfold directly beneath the file summary;
there are no modal dead ends. Status always combines a geometric glyph, a
plain-language word, and supporting text. Keyboard focus is a 3 px proof-blue
ring with 3 px offset. The phone layout drops decorative coordinate labels and
stacks every action at full width; the audit itself remains complete.

The landing page uses the same inspection tray for its isolated sample. In demo
mode, the generated aperture image yields to a populated result at the same
size, so the transition explains the product instead of decorating it. A
proof-blue banner keeps the temporary state visible.

## Motion

UI transitions are 180–240 ms and limited to opacity and transform. During a
scan, three finite scan bars cross once, then the progress state remains
static. Result groups enter from the tray that produced them. Under
`prefers-reduced-motion: reduce`, transforms and scans are removed and state
changes use instant/opacity-only updates. Nothing loops indefinitely.

## Original asset plan and provenance

The landing hero uses one generated raster illustration: an exploded stack of
paper planes under a circular forensic aperture, with a black redaction bar
whose concealed cyan geometry remains visible only in the aperture. It
clarifies the difference between visual covering and structural removal.
Interface icons and the wordmark are hand-authored SVG/CSS geometry.

Prompt sheet:

> Use case: stylized-concept. Asset type: wide landing-page hero illustration.
> Scene: an abstract exploded stack of ivory paper sheets and translucent
> technical layers suspended in a deep warm-black field. Subject: one matte
> black redaction rectangle crosses the upper paper plane; inside a large
> circular forensic aperture, a hidden cyan typographic-like grid is visibly
> exposed below it, while outside the aperture the sheet appears sealed.
> Style: high-end editorial 3D papercraft, strict generative geometry, subtle
> grain, no photoreal people. Composition: wide 3:2, object concentrated
> center-right with calm negative space left, slight isometric lens. Light:
> raking warm studio light, precise shadows. Palette: warm paper, toner black,
> vermilion edge marks, proof cyan. Materials: uncoated paper, frosted acrylic,
> matte ink. No text, no letters, no watermark, no logos, no interface, no
> brands, no gradients, no neon glow.

- Generator: Azure AI Foundry factory image deployment via
  `/opt/fleet/lib/gen-image.sh`.
- Generation date: 2026-08-28.
- License/provenance: original generated asset created for this repository;
  no people, brands, or copyrighted characters.
- Source candidates and exact prompt sidecars live in `assets/src/`.
  Responsive WebP files and a JPEG fallback ship in `site/public/assets/`.
- `social-card.webp` is a 1200×630 crop of the same original source. The app
  icon supplies the local apple-touch icon; the favicon is hand-authored SVG.
