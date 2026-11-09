# Japanese Typography Specification & Native Font Support

This document details the architectural specification and implementation of native Japanese typography and font handling in PhotoCraft, establishing full parity with professional desktop image editing standards.

| Capability | Technical Implementation |
|---|---|
| Localized Family Names | Resolves localized native font family names (e.g. `HG行書体`, `ＭＳ ゴシック`, `游明朝`, `メイリオ`, `HG正楷書体-PRO`) via OpenType `name` records (Platform 3 / Language ID 0x0411) |
| Dynamic UI Locale Coupling | When UI locale is Japanese (`ja`), font menus automatically prioritize localized native family records in `sync_font_names` |
| Preference Defaults | `TypePrefs.show_font_names_in_english` defaults to `false`, providing immediate localized font listing out of the box |
| Native Sample Glyph Previews | Fonts containing Hiragana glyphs (`['あ', 'い', 'う']`) dynamically switch preview rendering from Latin `AaBbCc` to Japanese `あいうえお` |
| High-Resolution Banner | Font picker top preview banner renders `あいうえお` at 40 pt (`Huge`), faithfully reflecting brush weight, termination, and calligraphic nuances |
| Visual Classification: Serif | Mapped via OpenType `sFamilyClass`, PANOSE, and name patterns (`mincho`, `明朝`, `sung`, `song`, `宋体`, `宋體`) |
| Visual Classification: Sans Serif | Mapped via name patterns (`gothic`, `ゴシック`, `ｺﾞｼｯｸ`, `meiryo`, `メイリオ`, `hei`, `maru`, `丸ゴシック`) |
| Visual Classification: Script | Mapped via name patterns (`gyosho`, `行書`, `kaisho`, `楷書`, `sousho`, `草書`, `reisho`, `隷書`, `kanteiryu`, `勘亭流`) including `HG行書体` and `HG正楷書体-PRO` |
| Visual Classification: Handwritten | Mapped via name patterns (`kyokasho`, `教科書`, `pop`, `ポップ`, `ﾎﾟｯﾌﾟ`) including `UD デジタル 教科書体` and `HG創英角ﾎﾟｯﾌﾟ体` |
| System CJK Fallback Matrix | Complete coverage of Windows standard Japanese fonts (`Yu Gothic`, `Yu Mincho`, `Meiryo`, `MS Gothic`, `MS Mincho`, `UD Digi Kyokasho N-R`) |

---

## 1. Technical Background & Requirements

### 1.1 Localized Family Resolution vs. ASCII Fallback
In digital typography on Windows, Japanese OpenType/TrueType collections (`.ttc` and `.ttf`) store multiple string records in their OpenType `name` table:
- Record ID 1 (Font Family Name) and Record ID 16 (Typographic / Preferred Family Name).
- Platform ID 3 (Windows), Encoding ID 1 (Unicode BMP), Language ID 0x0409 (`en-US`): e.g. `HGGyoshotai`, `MS Gothic`, `Yu Gothic`, `Meiryo`, `HGSeikaishotaiPRO`.
- Platform ID 3 (Windows), Encoding ID 1 (Unicode BMP), Language ID 0x0411 (`ja-JP`): e.g. `HG行書体`, `ＭＳ ゴシック`, `游ゴシック`, `メイリオ`, `HG正楷書体-PRO`.

Prior to this implementation, the engine fell back to ASCII family names whenever `show_font_names_in_english` was enabled. In professional image editors operating in Japanese locales, native Japanese speakers expect font pickers and character panels to display fonts using their native Japanese names, indexed and sorted accordingly.

### 1.2 Hiragana Glyph Preview vs. Latin Fallback (`AaBbCc`)
Standard desktop font menus display a visual sample of each font. Because standard Japanese fonts on Windows contain basic ASCII glyphs for half-width alphanumeric input, naive preview systems checked for ASCII coverage and rendered `AaBbCc`.

This was fundamentally insufficient:
- Latin letters in Japanese typefaces are secondary auxiliary glyphs that fail to exhibit the typeface's true aesthetic characteristics.
- Calligraphic brush scripts (such as `HG行書体` and `HG正楷書体-PRO`) or traditional text faces (`ＭＳ 明朝`, `游明朝`) cannot be meaningfully evaluated by looking at `AaBbCc`.
- Industry-standard typography engines detect whether a font contains core Japanese phonetic glyphs and render the canonical Japanese preview string `"あいうえお"`.

### 1.3 Typographic Classification of Japanese Faces
Professional font pickers categorize typefaces into visual filters:
- **Serif**: Mincho typefaces characterized by triangular stroke terminals (uroko) and horizontal/vertical stroke contrast.
- **Sans Serif**: Gothic typefaces characterized by uniform stroke thickness and square or rounded terminals.
- **Script**: Traditional calligraphic brush scripts, including Gyosho (`HG行書体`, `HGP行書体`, `HGS行書体`) and Kaisho (`HG正楷書体-PRO`).
- **Handwritten**: Educational textbook styles (`UD デジタル 教科書体`) and casual display fonts (`HG創英角ﾎﾟｯﾌﾟ体`).

Without explicit classification patterns, Japanese fonts were either dumped into unclassified buckets or improperly grouped.

---

## 2. Architecture & Pipeline

```
+---------------------------------------------------------------------------------+
|                                 PhotoCraft UI                                   |
|                                                                                 |
|   +-------------------------------------------------------------------------+   |
|   | Font Picker / Options Bar                                               |   |
|   |                                                                         |   |
|   |  +--------------------+  +--------------------------------------------+ |   |
|   |  | Filter Column      |  | Font List & Dynamic Previews               | |   |
|   |  |                    |  |                                            | |   |
|   |  | - All Classes      |  | [TT] HG行書体           あいうえお         | |   |
|   |  | - Serif            |  | [TT] ＭＳ ゴシック      あいうえお         | |   |
|   |  | - Sans Serif       |  | [TT] 游明朝             あいうえお         | |   |
|   |  | - Script           |  | [TT] メイリオ           あいうえお         | |   |
|   |  | - Handwritten      |  | [ O] Inter              AaBbCc             | |   |
|   |  +--------------------+  +--------------------------------------------+ |   |
|   +-------------------------------------------------------------------------+   |
+---------------------------------------------------------------------------------+
                                      |
         +----------------------------+----------------------------+
         |                                                         |
         v                                                         v
+-----------------------------------+     +---------------------------------------+
|  crates/ui-egui                   |     |  crates/text                          |
|                                   |     |                                       |
|  * sync_font_names                |     |  * OpenType name table parser         |
|    - checks i18n::current()       |     |    - StringId::TYPOGRAPHIC_FAMILY     |
|    - unblocks localized_families  |     |    - StringId::FAMILY_NAME            |
|                                   |     |    - Platform 3 / LangID 0x0411 (ja)  |
|  * font_preview                   |     |    - Non-ASCII heuristic fallback     |
|    - JAPANESE_CORE ['あ','い','う']|     |                                       |
|    - JAPANESE_SAMPLE "あいうえお" |     |  * font_class::classify               |
|    - offscreen 18pt / 40pt render |     |    - Mincho/Gothic/Gyosho/Kaisho      |
+-----------------------------------+     +---------------------------------------+
```

---

## 3. Implementation Details

### 3.1 Localized OpenType Name Resolution (`crates/text/src/fonts.rs`)
The font parser inspects OpenType name table records with language precedence:
1. Queries `StringId::TYPOGRAPHIC_FAMILY` (16) and `StringId::FAMILY_NAME` (1).
2. Searches for Windows Platform (3), Unicode encoding (1), Language ID `0x0411` (Japanese).
3. Evaluates Macintosh Platform (1), Roman encoding (0), Language ID `11` (Japanese).
4. If no language tag strictly matches but a name record contains non-ASCII multi-byte characters (`!b.is_ascii()`), it is extracted as the localized representation.
5. Populates `FaceInfo.localized_family` alongside the standard ASCII `family`.

### 3.2 Dynamic Preview Generation (`crates/ui-egui/src/text/font_preview.rs`)
In `font_preview.rs`, sample text resolution inspects the font's character mapping:
```rust
const JAPANESE_CORE: [char; 3] = ['あ', 'い', 'う'];
const JAPANESE_SAMPLE: &str = "あいうえお";
```
When `JAPANESE_CORE.iter().copied().all(|c| face.has_char(c))` evaluates to `true`:
- The primary sample string switches to `"あいうえお"`.
- Offscreen rasterization renders the sample at medium menu size (18 pt) and banner size (40 pt).
- Provides immediate visual discrimination between Mincho, Gothic, Gyosho, and Kaisho typefaces.

### 3.3 Visual Class Classification (`crates/text/src/font_class.rs`)
The classification engine inspects `sFamilyClass`, PANOSE, and family name strings:
- **Serif**: Matches `mincho`, `明朝`, `sung`, `song`, `宋体`, `宋體`.
- **Sans Serif**: Matches `gothic`, `ゴシック`, `ｺﾞｼｯｸ`, `meiryo`, `メイリオ`, `hei`, `maru`, `丸ゴシック`.
- **Script**: Matches `script`, `gyosho`, `行書`, `kaisho`, `楷書`, `sousho`, `草書`, `reisho`, `隷書`, `kanteiryu`, `勘亭流`. Covers `HG行書体`, `HGP行書体`, `HGS行書体`, and `HG正楷書体-PRO`.
- **Handwritten**: Matches `hand`, `marker`, `brush`, `chalk`, `kyokasho`, `教科書`, `pop`, `ポップ`, `ﾎﾟｯﾌﾟ`. Covers `UD デジタル 教科書体` and `HG創英角ﾎﾟｯﾌﾟ体`.

### 3.4 UI Preferences & Font Synchronization (`crates/ui-egui/src/tools/type_tool/fonts.rs`)
- `TypePrefs.show_font_names_in_english` defaults to `false`.
- When UI language is set to `ja` (`craft_i18n::current().starts_with("ja")`), localized names are enabled regardless of legacy configurations:
```rust
let show_english = app.session.prefs.type_tool.show_font_names_in_english
    && !craft_i18n::current().starts_with("ja");
```
- Populates `font_names` with `localized_families` while preserving ASCII indexing for persistent document interchange.

### 3.5 System CJK Fallback Integration (`crates/text/src/cjk.rs`)
Windows system fallback chains provide seamless rendering across:
- `Yu Gothic` (`YuGothR.ttc`, `YuGothM.ttc`)
- `Yu Mincho` (`yumin.ttf`)
- `Meiryo` (`meiryo.ttc`)
- `MS Gothic` (`msgothic.ttc`)
- `MS Mincho` (`msmincho.ttc`)
- `UD Digi Kyokasho N-R` (`UDDigiKyokashoN-R.ttc`)

---

## 4. Verification & Validation

1. **Unit Testing (`photocraft-text`)**:
   - `font_class::tests::fixed_pitch_and_names_decide_the_rest`: Verifies correct classification of `MS Mincho` (Serif), `游明朝` (Serif), `MS Gothic` (SansSerif), `游ゴシック` (SansSerif), `Meiryo` (SansSerif), `メイリオ` (SansSerif), `HGGyoshotai` (Script), `HG行書体` (Script), `HG正楷書体-PRO` (Script), and `UD デジタル 教科書体 N` (Handwritten).
   - `cjk::tests::type_tool_fallback_follows_locale`: Validates locale-driven candidate ordering.
   - `tests::works_without_craft_fonts`: Validates fallback stability when optional font bundles are omitted.
2. **Binary Verification**:
   - Built optimized production release binary `photocraft.exe` (115 MB).
   - Validated font picker display with Windows native Japanese fonts.
