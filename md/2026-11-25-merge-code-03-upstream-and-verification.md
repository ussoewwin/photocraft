# Upstream inventory: every commit the two merges brought

Generated with `git log --oneline --no-merges <old>..<new>`: change commits only. The merge commits
carry the pull-request titles and are not reproduced here; list them with the same command without
`--no-merges`.

- craft-libs `upstream/next` `a4e08cdaf..9f2961402`: **1279 commits**, of which **1085 are listed here**
- photocraft `upstream/main` `dbad7de99a..e56522780`: **629 commits**, of which **354 are listed here**

## craft-libs `a4e08cdaf..9f2961402` (non-merge commits)

```
06675d0d5 CI: replay audit in a parallel shard from one build; test builds use cl's profile overrides
9054b83c2 CI: next keeps its own Linux workspace-crates cache entry; CRW samples are saved once
ceded0663 CI: pull requests gate on the Linux Workspace legs; macOS and Windows run after the merge
44260259b ui-contract: access state through runtime::egui::memory; mark the host-key trial's TextEdit read
ea3b99e89 ui-patterns: drop a dock-gaps trial file the merge picked up from the worktree
42598245f egui: storytold/egui next a87d00caa (an open ComboBox menu grows back after its list shrinks, egui#5225)
86ac6ac5f ui-panels trial: the claims-after-focus stop is a canvas control
859d80118 A float opened from a listed layer stacks above it
53e91a75d font-manager: the FontPicker stop-build counter lives in a runtime memory session, not the egui adapter
3ad64a26e craft-keymap: DeleteKeys says whether Delete and Backspace are one key
77ce50254 fonts::load records the facts of the definitions it installs
ef7239a97 A host canvas holding egui focus keeps Escape and its clipboard in egui
4549b4f01 automation: the client trials' server writes each reply line in one write
7b8bbd254 font-manager: count FontPicker stop builds through the runtime's service memory
a783828e5 menu: a translated label with no letter of its own shows its English letter only in CJK languages; elsewhere the letter stays as a hidden mnemonic
5b190db66 Lasso cursors: an arrow on the hotspot with the tool icon beside it (photocraft#16671)
2014d31f3 theme, dock grips: GripMark::Bar (shape.grip.mark "bar"), one level bar length long and width thick; a grip's length may reach 32; GripMarks paints it; VectorCraft's themes pin production's Tools grip, an 18 by 3 point bar. Trial; dumps updated
c7ce7e546 ui-widgets: ReferencePointLook::Squares fills the pick in the normal text ink and draws no sunken well or border, as production's locator; compact squares story
747dad4c1 ui-contract: CaptionInk, an env style value for the ink of a side label's name: Muted (default) or Normal, VectorCraft's field labels as production drew them; Field story field_caption_ink
33774554c ui-panels: a titled Panel draws the rule under its header only for a weight that stands with one, as Section does for its rule above; an Inspector's Quiet kind line has none
c26752ab9 WIP: lasso cursors: an arrow on the hotspot with the tool icon beside it (photocraft#16671)
a81bc6fe7 dock group: beside PanelTabLook::Pill tabs the panel menu button stands in the strip's middle, level with the pills, as production's ≡ is centred in its cell; plain tabs keep it at the strip's top. Story and trial
32dcf265b Move egui to storytold/egui next 7520646e5c: AccessKit for every viewport window (storytold/egui#58)
b3b2eaac4 tabs: DocumentTabLook::Semibold shows VectorCraft's document tabs, whole titles as Whole with the title and its * in the label's semibold face (Span::strong), measured as painted; a Semibold story
5a2896d3e dock: the host's focus request brings the floating window that holds the panel to the front
60a5de5ca dock: a host moves key focus to the group that shows a panel, as a click on its tab does
d1f00545e ui-contract: an IME owner that is not shown loses the IME in that pass, so its interrupt is not a frame late
5bf2d9cb4 font-store: the preview service's client moves here from font-manager as `preview` (no view code: subset keys, the disk cache, the fetch pool); the pool reads the store through `PreviewStore`, which font-manager's `Installer` builds on; Instance, Subset, PreviewError and Preview are non-exhaustive
f4d3f6c55 design-lint on next: the plain artboard edge's black as a data colour, the task bar grip story's drag in drag thresholds, dock column cards take their outer inset as a CardInset step, the link label ink story reads the link's click from its output
36106c2b9 tabs: a strip with wheel_selects adds the wheel scroll short of a notch to the next pass, so small trackpad deltas switch tabs
16c2e06ce ui-panels: a floating window's resize grips carry the accessible name Resize panel window
dba4b84b2 keys: the app's own widget, focused or the window's canvas, keeps its keys in egui again
b961098d0 A press on an app's own egui widget in a container body is not a press on the container's surface
badd2a882 ui-contract: a host's window-input policy reaches every OS window the runtime hosts
ba2c36b30 SplitArea keeps the handle paths it declared for its dividers; trial for DockAreaOutput::sources
393282d78 DockArea reports the gesture sources it drew: tabs, group bars, window grips and resize grips, dividers, each with its rect, OS window and node id
b43aa1259 ui-canvas: a full-window dialog's own window is at least OWN_WINDOW_MIN_SIZE (640 x 480), so a placement saved while it shrank opens usable
b0a8dcb38 automation: the unasked-line test writes its line in one write, so the client's close can't reset the newline (BrokenPipe on Linux CI)
43c89a0b9 Docs links from main's sync: material-graph names Graph by its module path; font-store re-exports SERVICE_BACKOFF, which its public docs name
500a9708c craft-aaf: document the linked-audio descriptor variants (next denies missing docs)
ac416949b ui-canvas: a task bar whose anchor fills its area keeps its members (photocraft#16657)
9b897a57a Full-window dialogs keep their title clear of the window buttons; Liquify labels its View Options choices (photocraft#16658, photocraft#16659)
a9ef8480a WIP: full-window dialog title clears the window buttons; Liquify dropdown labels (photocraft#16658, photocraft#16659)
481946905 WIP: a task bar whose anchor fills its area keeps its members (photocraft#16657)
34e96f932 ui-panels footer: DialogButtonLook::Filled shows Cancel, Other, Alternate and Alternative as the filled neutral button (buttons::FILLED, now public: surface.control with no edge, state.hover under the pointer) beside the accented answer. Footer story and trial under Filled
7cb5edfda ui-widgets segmented: a picked segment follows ActiveLook; under ActiveLook::Pressed it takes state.pressed with text.normal at rest, under the pointer and pressed, as an icon button that is on does; ActiveLook::Accent, the default, keeps state.selected with text.on_selected. Segmented story under Pressed
68cde01dc ui-widgets choice: ChoiceLook, a root style value; under ChoiceLook::Strong a check box, radio button or switch track that is on fills with accent.strong (pressed accent.strong_pressed, disabled unchanged) under a text.on_strong mark; ChoiceLook::Accent, the default, keeps accent.solid with text.on_accent. Check box and radio stories under Strong
09650e1d3 ui-contract style: TrackFill, a root style value; under TrackFill::Accent a slider's and a range slider's track fill the value's part in accent.solid (disabled text.disabled) on a line.outline groove; TrackFill::Muted, the default, keeps text.muted on line.control_edge. Slider story under Accent
6ea1ea73b ui-contract, sections: SectionRhythm, an env value an app sets once at its root. Under Spaced a header ruled above (a Section) stands its rule space.gap.section off what precedes it in its Stack, in place of the stack's gap (no room at its flow's start), and its header space.gap.row below the rule; Tight, the default, keeps the stack's gap and the header right under the rule. The Stack's gap reaches its direct children through the env. Story section_rhythm shows both.
6798a83eb theme, tabs, canvas panel: size.bar.tab_large, the height of large document tabs (today's size.bar.title in every size set), so large tabs no longer take the title bar's height; CanvasPanel gives its tab pane an extent, size.bar.tab or size.bar.tab_large with the new large_tabs prop, so a strip taller than its first measured height shows its tabs whole. Story canvas_panel_with_large_tabs.
66ab239f5 ui-contract, text: TextRun::data_color fills the run's colour from data, painted as given, as IconSpec::data_color does for an icon: text over document content in the content's own colour
327fcaa90 canvas, pasteboard: Pasteboard::artboard_edge chooses ArtboardEdge::Plain, which edges the active artboard in black at stroke.thin and the others in grey 120 at a hairline, data colours that read against the paper; Accent stays the default
dc8b91f5f list, menu: CommandHighlight (an env style value, Accent by default) chooses how a menu's active row, a popup list's command row, the palette's active row and an open menu bar title show it; under Hover they paint state.hover with their own ink, as a row with an open submenu
b4d4b64dd color-ui: pass PaintChip's style to layout without a needless borrow, so next's Workspace lint passes on clippy 1.100
6996c6479 font-manager: skip building FontPicker stops on idle closed frames
9a41bd4b8 ui-contract, field labels: LinkLabelInk, an env value an app sets once at its root, draws a field's label shown as a link in text.normal (Neutral) instead of the link look's accent (Accent, the default); the underline stays. A free Link keeps the accent and takes Tone::Neutral for the same ink. Story field_link_label_ink shows both.
6af5a5eaf theme: VectorCraft Modern and Modern Medium pin state.text_selection to the accent at 55% alpha (dark #f06a528c, light #d9472f8c), as production's field selection
06e94d732 tabs, dock: PanelTabLook::Pill shows a panel strip's shown tab on a filled rounded pill (surface.control, shape.radius.control, size.control.sm high in the middle of the strip; a hovered tab shows it too); TabStrip::panel_look and DockConfig::tab_look choose it, Plain by default
2eb195b88 ui-panels, dock area: DockConfig::column_cards (was rail_cards) shows every docked column as a card, expanded or collapsed to icons: surface.panel with shape.radius.surface corners on the dock's background, inset by space.gap.stack on top, bottom and at the window edge, and space.gap.tight between neighbouring cards; each column wider by its inset so its content keeps its width; group dividers stay inside the card; container surfaces keep to their layer's frame
a4b6d3b6c ui-contract: MOVE_BAND is re-exported in every build, not only the suite's, since BodyCx::moving reads it; an app building without the suite feature compiles again
faf5a954e ui-panels, dock area: DockConfig::rail_cards shows a docked tool rail's column as a card: inset by space.gap.stack on its outer, top and bottom sides over the dock's background, surface.panel with shape.radius.surface corners and no frame, the column wider by the inset so its steps stay
6692f1b83 A memoized view whose content fills the axis its body shares is one of the children that fill that body
02dc636a6 ui-contract, title bar: one way for an end group whose ranked members leave first: TitleBar::show_fitted turns on its end region's fit, as Bar::trailing does, and layout::end_group goes; the fit's env fact carries the body's shortfall, which the region's first row gives up off its ideal width
50a9ed02a Right-click on guides, rulers and the transform box is theirs again; percent fields keep the caller's decimals; a mixed font family shows an empty field
41a067929 ui-widgets tests: the selectable-text trial names its canvas with Id::unique, not the deprecated Id::new
02d26c8cc bars: a bar's trailing group joins the body's fit; its ranked members leave, lowest rank first, before a body member narrows or leaves
bbef5516c ui-widgets: the selectable text trial names its canvas with Id::unique, as egui deprecates Id::new
225de8b83 automation: the actions file keeps every set's order
cb367bccf menu bar: MenuTitlePad, the padding role of the titles an app sets at its root; Control sets them closer, as an app bar whose menus share the line with many controls
161c99c63 ui-contract, title bar: layout::end_group places a row's end group at its end edge, its members ranked units of the row's fit; TitleBar::show_fitted shows the trailing content as the body's end group, so its ranked members leave before the menus fold
49d439d63 ui-widgets: a tile grid without card_columns asks for one card in a measuring run again (ebf925452, lost in the merge b138b533f), so a dialog pane sized from it holds its rows and the row under them: presses on the tiles, the button and the field beside it land on them
d3b03c1b4 ui-contract: a command a key, a native menu pick or the palette runs no longer closes a layer whose Dismiss::commands is off; a dock panel's flyout turns it off, so the command no longer closes and reopens it and a focused list in it keeps the focus
065efff0c Outline buttons: a line.outline token for the neutral outline's edge
0aeb4251c Filled push buttons follow PushButtonLook only when they ask for no variant
17d8a3424 ui-widgets: the inline number trials share one field rig
d0e1ca86f docking: DockConfig::panel_menu_icon picks the panel menu button's icon, more by default
6080b3950 sections: HeaderStrength::Quiet, the caption role in the muted ink with no rule, for an inspector's kind line
b808cac9f labels: GlyphColon::Form gives a glyph caption (GlyphLabel::caption) a form's colon; draw's X/Y/W/H are captions
1a17a4963 ui-canvas: TaskBar::hang_gap(TaskBarHang::ClearOfHandles) hangs the bar the handle ring and a section gap from its anchor
0d8d3f9c9 ui-canvas: TaskBar::member_gap(Gap) spaces the bar's members, through a new Bar::gap
898219168 title bar: document the surface role it takes and show it on the background surface in a story
45a8457c7 ui-widgets: DocumentTab::name_len, the label's name part, which a modified tab's mark follows: "Untitled-1* @ 93.5 % (RGB/Preview)"
bb06692b1 ui-widgets, ui-panels: ActiveLook::Pressed shows an icon button that is on in the pressed fill with no accent edge; ToolRail::active_look gives it to the rail's buttons
95c145682 ui-panels: a dense tool rail step: RailDensity::Dense packs the rail's buttons a control high, and DockGrid::density sizes the rail's column with them
4ae91846f title bar: take show_fitted out again; with caption buttons its trailing content stops short of the end and a ranked member narrows instead of leaving
ddd2e3c4e Filled push buttons: an Emphasis::Filled look on a new surface.control token, and PushButtonLook for an app's default
0a31be98c ui-widgets: NumberPrecision::Measured shows a length to a thousandth of a point and a ten-thousandth of the larger units (357.114 pt)
361012ff9 ui-canvas: TaskBar::grip(TaskBarGrip::Bar), one rounded bar lit under the pointer and while the bar moves (BodyCx::moving)
269ce9d69 ui-widgets: NumberPrecision::Measured lets an app show whole percentages (60%) where the standard precision writes one decimal
827141f83 color-ui: PaintChip::chip_size(PaintChipSize): Regular, Compact (a small well in a control-high slot) and Swatch (a third of a rail button, no popover)
b21bc0e35 select: SelectLook::Quiet, a quiet box as wide as the entry it shows; WorkspaceSwitcher::look passes it on
fe138f934 docking: DockConfig::panel_menu_always gives every group its panel menu button
8da5ec92a ui-widgets: NumberLook::Inline sets a number field's value and unit as one run at its start (720 pt, 0°, 100%)
19370dcaf font-manager: document the preview client's public items
988a5c82a color-ui: PaintChip::popup_mark(PopupMark::Menu) draws the dropdown arrow after the well, inside the chip
9450c7fa7 title bar: show_fitted puts the trailing content after the body on its line, so ranked members leave before the leading content narrows
aa973d98f ui-contract: a press the focused component holds leaves egui's input again unless the app's own widget reads it; the merge b138b533f passed whether anything had focus for whether the focus was the app's widget
51eaf9fd2 ui-contract: a page key the keymap binds to a command that runs now runs it ahead of the document canvas's page scroll, focused or not; an unbound one still pages
47691f10c Inspector: title_strength gives the title another weight than a section heading's
c71c3025c sections: HeaderStrength::Subheading, label text in the normal ink with no rule
48d19d281 ui-contract: a command fired from a button runs as a command; only a menu's or the palette's fire stands for the key a component claims
45c8a4a3f draw: Shift while dragging a Bezier handle turns it to the nearest 45 degrees about its anchor
1d281f38c atomic: a write-only file's unreadable attributes are dropped, as documented and tested (review)
f29372d28 atomic: an unreadable file saves only when it has no attributes to lose (review)
52503acd7 Fix the macOS test failures next picked up from the PdfCraft framework sync (#3232)
0bf0a39d7 text: the Character panel's fields fill their columns
48936610c arrange: Tile All tiles docked cells in the document area
8b4e49d62 drag: the window a drag moves paints the drop mark over itself
cfca45a97 docking: a tab drag names its panel inside the drop region, and a tab join marks the group as shown
c7dbf52f5 ui-contract: a tooltip keeps its text width in an inspector cell, a press hides it until the pointer leaves, and a sampled press does not nudge its modal
eaeb0fa02 platform: owner_only's docs name its Windows-only functions as code, not links
87ed3c0be ui-contract: one modal stack across viewports, ordered by the frame each top modal opened in
9750fe6d7 i18n: the region test module builds only where its one test runs
c13c6eeed docs: regenerate the components index
8ef406cd5 ui-widgets: a tile grid that takes its width from its content asks for at least one card again
3b200f43d text: a one-line field pastes each line ending as a space again; the editor's step loop is its own function
22205fffd ui-contract: an OS file drop lands where it was dropped, and files hovering in the pass that takes it start their own drag
ae722739a ui-contract: presses the focused component holds leave egui's input again, and Tab stays navigation with nothing focused
9c9e7ae80 ui-contract: a focused stop in a layer that plays its exit loses the focus in the first pass of the exit
a6c6eda35 theme: VectorCraft Modern's selected, pressed and disabled colours take production's values
12c5d012e ui-contract: a focused text field keeps the Escape it does not use, and every other focused control lends it to the window
cd87dc61f theme: DeckCraft and SoundCraft accents in the suite theme's app pins
3a10c49a2 Quit is a framework command, and the update dialogs show the icon the app gives its host
b6b290812 theme: regenerate the VectorCraft token dumps with the six tokens next added
68a41a9e8 text: Format Text's font row names the font_picker prop without a link the font-picker feature gates
5d9f4b62c Hosted egui apps on next: host code runs once per pass, and the app's own modal blocks its other windows
9aa180d03 ui-contract: a press egui gives to a selection part's text takes focus through the press plan
afbe61c54 automation: the control server ends a connection after its last reply by closing its write side and reading off what the client still sends, so the close no longer resets the connection and loses the reply or the end of the stream
f92a0be4e atomic: drop the empty line between KEPT_ATTRIBUTES and its doc comment
7002b2ec8 ui-contract: drag::register takes a target's reach as one TargetReach; collections report an item's output through one helper
daed17160 ui-patterns: a hosted dialog's automation set takes the host's row labels and shown states, every tab's rows and the read-only refusal again, and reports the rows it changed, which the merge of next dropped from the batched set
969bfa99f ui-patterns: a document dialog's fixed section puts its rows on a card again and heads them with its name, which the merge of next dropped from the section rewrite
c41413e8b platform: name the page save callback type; web-host: implement Files::pending_pick
1b7222027 behavior: a grid's least probe keeps its Fill columns Fill, unbounded, so a field that fills its width there measures its own width instead of none; the fill column's least is then the field's, so a grid beside a menu no longer squeezes the field under it, stacks its label in rows measured unstacked, and overlaps the next row
5d094e34a ui-widgets: declare the Arabic test font read from ui-contract
22e82c930 Keymap: warn about shortcut overrides that name no command; framework doc names the pane and menu-bar command ids
48e798653 HostStyle moves to primitives: the dock pattern reaches the host style through the contract
1b0ffe1ad Hosted egui apps on next: dock tabs keep key focus, floats stack with app windows, host bodies keep the app's background
07a223c0f ui-contract: a full-window container's own window keeps its size
80854fad0 draw: Slice Options takes the host's answers only in a real run
8311df040 ui-patterns: hosted New Document takes the host's answers and closes on an accepted held answer only in a real run
3eddf17ee ui-patterns: hosted New Guide Layout and New Guide take the host's answers only in a real run
b17875224 ui-patterns: hosted Save for Web takes the host's answers, the Optimize answers and the automation inputs only in a real run, so a palette colour answered in a measured frame still becomes a shift
8f60b79b3 ui-patterns: hosted Export As takes the host's answers only in a real run, so a path answered in a frame whose first run measures the dialog still finishes it
7e5184034 WIP: own-window full-window dialogs keep their size (photocraft#16595)
55fef13aa Hosted egui apps on next: canvas claims, Ctrl+wheel pinch switch, host style in dock bodies
362844f70 ui-contract: sync egui font color transfer function with resolved theme mode
e952ad33c workspace: unreached-rs allows vendor/**, whose crates are on the workspace exclude list and so have no module tree the check can follow
3a90546e2 ui-contract, ui-test-kit: a press egui gives a component's text widget (a focused field's) is no press on its container's own surface, so a second click into a focused field in a dock group no longer hands the focus to the panel's tab, whose Escape and Enter then never reached the window; test kit Windows::root_env sends an app's root values
b78d580c7 ui-contract: a window-wide drop target (a tab strip's tear-off) yields to a component's target under the pointer in another window, also one that window's running pass registered: a tab dragged out of a document's OS window and released over the main window's strip tore off instead of docking there
5353053e5 ui-contract: RootEnv::shortcuts_repeat: with it on, a held key's repeats run its chord's command again as FirePhase::Repeat, matched on their own (no double press, no hold wait settled), except a key with a hold binding or one held over a modal layer's opening or closing; off by default: a shortcut bound to a held key ran only on its press, where production repeated it
0a6e42b80 fw sync: a Fill picture without a set aspect takes the SVG's own or a square (next 126ec28dc meets the port's optional aspect)
dab7d1a11 ui-contract: a shell without Caps::SYSTEM_CURSOR keeps the system pointer under a painted cursor and paints none, nor hands the window an image: a host on Wayland with no cursor theme, where a hidden pointer never comes back, could not keep it
efebc0cda color-ui: ColorDisc::stored_as and HarmonyWheel::stored_as take the host's round trip through its own colour model, so a drag whose colour comes back moved by it (CMYK through a profile) is still the drag's; without it the disc keeps its 8-bit echo check: a host storing colours in another model had its own drags cancelled as outside changes
0726c0f87 ui-contract, ui-canvas: KeyUse::command_keys and Canvas::editor_command_keys: a canvas editor that has the keys leaves chords with the platform's command modifier and function keys to the app's commands under any typing rule, its text editing chords kept; off by default: under TypingRule::AllWait a type tool's editor ran none of the app's Cmd chords or function keys
83b922e63 ui-contract: an Alt chord's text is withheld only when a binding of it in a scope around the focus can run, as the key stage matches the press: a type editor's Alt chord in a scope no focused component is in kept the character a layout types for it (a Blob Brush's [) from reaching anything
85ba69b14 ui-patterns: PaletteOutput::refused names the disabled command Enter was pressed on, from the new CollectionEvent::Refused of a disabled lead's Enter, so an app can close the palette and say why: the palette stayed open and told the app nothing
662ff9264 ui-contract: with RootEnv::escape_leaves_fields, Escape in a field of a dialog that Escape closes stays the dialog's with nothing typed too: the field took it to give up the focus and the dialog stayed open
4a6ea8a98 ui-contract: a text part asks where the keyboard goes back to only once it held the focus or keeps a selection for a menu, so a part that never held the focus reads no focus in its run: the replay audit found a memoized view whose field read the focus only when another stop had it
126ec28dc ui-widgets: a painter picture can fill its slot (wordcraft#7317 C3-1) (#3131)
0dfc9d47e fw sync: the flat note Reply's edge takes the note's colour
346f5ac9f fw sync: PageRangeChoices::Line, ParamDialog::caption_column, Param::short_slider (Slider/SliderField::short)
e93d468b7 fw sync: floating window bodies keep to the window's rounded corners (frame_surfaces, FloatingGroup::rounded_body, DockConfig::floating_rounded_body); CaptionColumn env value for inline forms
249a41bd9 craft-develop: make the sensor clip mask public for callers that demosaic themselves (#3178)
e36d3d003 ui-contract: RootEnv::escape_leaves_fields lets an app's Escape in a focused text field also give up the focus, with or without typing to revert, as egui's own fields did; off by default: a framework field kept the keys after Escape, so the next Enter stayed the field's
3b17e2b36 behavior: NumberCodec::share_of_start reads a lone percentage (Reader::share) as that share of the value the edit started from, on every pass and on commit only: a size field's 50% could only be read against the value of the moment, which the numbers typed before the % had already applied
a3fca1707 Share typed AAF linked audio descriptors (#3180)
dd5da2141 theme: the port theme's token dumps take the values the merge of next changed
928e47822 dialogs: Save As onto the document's own file writes the file type the panel picked, also after the changed-on-disk question's Replace: the own-file path dropped the pick
fa1fac17d ui-contract: a text part keeps its selection while a menu bar or a menu the keyboard goes back to it from has the focus, as it did for its own edit menu: the window's Edit menu showed Copy disabled for a field's selected text
a3b7f774f color-ui: trial of a linked harmony wheel drag whose host moves every colour from the drag's start in 8-bit colours: it runs to its release and the other colour turns
b991e3f9c ui-panels: a footer's armed buttons wait again for a host's next dialog shown in the same place with the same ids (overlay::running_dialog): a question put in place of another between two frames took the click the one before had armed
1c093cc7e color-ui: a click on any swatch reports SwatchesPanelOutput::used, with its colour only where it holds one: gradient, pattern and None swatches reported nothing, where production applied every swatch
fe2d1d09d ui-contract: DropTarget::under_modal and WindowDropTarget::under_modal let a target take OS file drops while a modal layer holds input, through the modal's scrim but not its body: a modal refused every drop below it and on the window, where VectorCraft placed files dropped under its alert; off by default
46b9fe002 ui-contract: Shift or Alt with Escape that the keymap binds to an enabled command runs it: the key stage read every Escape without Cmd or Ctrl as the scope's escape, so a command bound to Shift+Escape never ran; a plain Escape keeps its meaning
d0a0cf671 fw sync: a compact TaskBar takes a popup's smaller corners and shadow
9b12c1960 fw sync: Param::stacked_options and Param::under_option (production's Split and Background option rows); a document-dialog group under a heading of its name shows the name once
02e29c4e5 fw sync: NoteReplyLook::Flat opt-in: the note pop-up's Reply as production's small flat rectangle
c3155aad9 Share the material graph runtime and WGSL pixel compiler (#3109)
db33516bf Tests: restore control-connection and X11 clipboard guards lost in WordCraft's port (wordcraft#7317) (#3115)
1c5ad1b85 Update: a rate-limited check is a typed reason apps can translate (#3090) (#3098)
ee01a2b42 theme: dark-mode accent buttons show a visible hover (wordcraft#7317 C7-14) (#3124)
eb7fe4ff0 ui-contract: a bare Tab where it would toggle the panels runs the enabled command the keymap binds to it, when that is not a chrome command: an app command that took Tab from Toggle Panels never ran, as the Tab gate only took chrome commands
9850b031c fw sync: FormatText paragraph_spacing_presets and paragraph_spacing_range (default stays the Paragraph dialog's 0-1296 pt); trial paragraph_spacing_takes_the_hosts_presets_and_range_on_the_line_spacing_row
be5e97ea4 fw sync: F-R1: Param::select_on_focus for text rows (Rename opens on the name selected); trial a_text_row_that_selects_on_focus_is_replaced_by_typing
874efb457 fw sync: F-P4: PrintPanel/PrintDialog/HostedPrint printer_search opt-in (Search printers field, No printers match., Save as PDF kept, 430 pt list); Select search_label, no_matches, search_kept, list_width; trials printer_picker_filters_a_large_fleet_without_changing_the_selection, printer_picker_keeps_save_as_pdf_available_when_the_filter_matches_nothing, printer_picker_scrolls_long_names_without_losing_the_search_or_pdf_choice
10c2dc7ce fw sync: F-P3: TextEditor Shift-clicks after a double press extend from the whole word both ways, a key move or Select All ends the word anchor; trials a_double_press_on_a_one_letter_word_selects_only_that_word, shift_clicks_after_a_word_selection_extend_from_the_whole_word, a_key_move_ends_the_word_anchor (production 439401188)
fb99ed907 fw sync: F-P2 / L-315: macOS title bar follows the traffic lights at the right (craft-window reads windowTitlebarLayoutDirection; MacInset::lights_right; the lights' room moves to their side; AppShell::title_follows_lights mirrors the title slot's flow, tabs and overflow included; Overrides::flow); trials a_title_that_follows_the_lights_mirrors_when_they_are_at_the_right, the_lights_room_follows_the_lights_without_mirroring_the_bar (production 4448ee940)
05f6b87d4 fw sync: TaskBar::compact opt-in (small members in a tight frame, Pad::Tight, Bar::compact) for PdfCraft's text box Done/Discard bar
c170a4138 fw sync: clippy clean: the eased wheel zoom's payment is its own function (settle's cognitive complexity)
157714c25 ui-patterns: in the pass a tab click picks a panel of the focused group, DockAreaOutput::focused names that panel: it named the panel the group drew in that pass, the one before the click
090f7c0c2 fw sync: CommentComposer::fixed_rows keeps the card one size
05260a58d fw sync: PageView::fill_screen has no scroll bars
fb3a6a636 fw sync: opt-in platform paste for a fired Paste (CommandInput::paste_reads_clipboard)
4854e2cff fw sync: a search field's Clear shows the empty text in the pass of its click
efe404092 fw sync: opt-in eased wheel zoom on the canvas (NavigationOptions::eased_wheel_zoom)
15e8ac805 fw sync: a dialog can keep a field's typing on Escape (FocusPolicy::escape_keeps_typing, Dialog::escape_keeps_typing, PreferencesWindow::escape_keeps_typing)
238d922e1 fw sync: L-316: copied files are a Files primary in the clipboard offer plan (arboard's file list); trial a_file_list_is_written_as_files
51ad138dd ui-contract: a drag of OS files runs in the window under them: the main window's pass, which the OS sends none of a child window's files, ended the session, so files dropped on a native panel window never reached DockAreaOutput::window_files; the test kit's Windows now sends files to one window (FilesOver), as the OS does
ac276c4e9 font-manager: the picker's script trial reads Sarabun from font-subset's test data, so the crate declares that read
f8714b237 ui-canvas: ToolHost::navigates: a press the host says navigates (a finger once a pen has been seen) runs the momentary Hand, latched for its gesture
95e4ec879 behavior, ui-canvas: RulerUnits::with_labels(TickLabels::Signed) writes ruler ticks signed with only the decimals they need (-2.5, 0.3, 1), VectorCraft's form; TickLabels::Distance stays the default
24e30ffac ui-shell: a document window's raise brings its fallback layer to the front in that pass (overlay::raise), as a press on it does: it only asked an OS window for focus
bbd5eb35d ui-contract: a focused text part takes as its edits only the bindings of Cut, Copy, Paste and the rest that are the platform's text-editing chords: an app's alternate key (F4 for Paste) went to the field too, where production left the text alone
cfd3adbe6 fw sync: L-321: MiddleAutoscrollFeel::Reader opt-in (production's speed, short-hold latch, ends on canvas blur)
f2e83cd92 fw sync: a focus request for a part waits for that part instead of landing on a sibling
d132f0101 fw sync: TextEditor::returns_focus (on by default): an editor its caller opens from a part can leave the opener alone
6e619265e fw sync: cargo fmt (review note_popup trial from fbd0bd096)
0c633a83c ui-canvas: a rulers corner drag lands at its release only while its edit is open: after the host cancelled it, the release proposed and committed again and moved the zero point
60e39bc32 cl doctor: dev-dependency features across the egui crates
52d8e207d feature-root: ask epaint_default_fonts for the monochrome emoji faces
4304f51ff cl doctor: rules for the split UI test kit
e433ef83e ui-widgets: trial: a grid row holds a field whose label stacks above its control for want of room
3ce58d7ca ui-patterns: a form dialog's body takes the host's answers, the opening pass and OK only in a real run: the dialog's own measuring run of the body (a re-layout under a child dialog) used them up, so Save for Web's real run reported no palette shift
2e4bf56a8 print: the hosted print dialog's automation preset field takes a row's key or its name in any case and applies that row's key, as production found presets by name ignoring case
b14f3a313 ui-contract: a component's focus request in a press's pass holds: the press's own focus move at the pass end does not take the focus back, as a layer that appeared in the pass already kept it
2f068b43a ui-contract: an injected step whose events the frame it entered held back (behind a real press, H0) stays in flight into the pass that delivers them, which reports injected_pass
77b2d98f6 ui-contract: an open anchored layer keeps the side it took only while its host asks for the same side; asked for another, it takes that side's placement
a7f40c447 draw: a running TransformFields gesture reads its values against the start box it began with, so a host that moves the start box with the gesture's own values does not cancel it as an outside change
deb9ad9f0 ui-contract: Tab toggles the panels from read-only text a press focused (a selection part, a press-only stop), as from any pressed control; only a text field keeps Tab for the next field
d824d42d2 ui-patterns: a new palette query's active row is its first match, enabled or not, so Enter on a disabled first match runs nothing and the footer says why, rather than Enter running the next match
508faa1eb ui-contract: an Alt press an input method types through is typing: the focused text stop that has the IME holds it, so its bound command does not run beside the commit
1fc46b573 ui-contract: a one-character text with no press of its own is a tap of the key it stands for: a claim takes it, the shortcuts read it with the held modifiers (Shift+~ arrives as ~ alone) and a focused field types it; a text Alt typed with a press no claim took is its key's for a claim (Option+8 types [)
cbbc1c26e ui-contract: a clipboard event with Cmd and Shift or Alt held is that chord (Cmd+Shift+X), the app's, unless the focused editor lists it for the edit: only the edit's own chord is the editor's Cut, Copy or Paste
fd80adccc ui-contract: a press of a key that is down already is a repeat, as egui reads it: egui-winit sends every press as no repeat and leaves egui to tell, so the key stage's repeat rules (shortcuts, Escape, Tab) never saw a held key's repeats; the trials that pressed one key twice release it between
34d7846cd ui-panels: a docking flyout closes on a press outside it only with Auto-Collapse Iconic Panels on
b72fc4200 ui-contract: Fixture::by_label passes over a node that labels another, so a field's control comes before its caption link
80e46cf00 ui-widgets: IconButton::choice reads as a combo box named by its label, valued by the picked row, expanded while the container hung from it is open (contract::overlay::opened_from)
879b8f82a ui-canvas: Fit Artboard on Screen leaves 30 pt around the board on every side
08ada730d ui-panels: an edited tool rail takes the drops off its buttons: beside the nearest button, and on a blank toolbar as its last slot
fbd0bd096 fw sync: an edit comment composer opens with the caret after the text (TextField::keep_caret_on_open)
47c6d53a1 font-catalog: FaceTraits goes, its class and PANOSE are FaceInfo's; the picker reads Japanese from the OS/2 code pages and ranges and Thai from the cmap
5abaa7360 fw sync: clippy clean over the merged sync (all targets, all features)
042dccd07 android-res: bound retained values and report UTF-8 recovery omissions
12d67b1cd ui-contract: a reveal that moves a collection's rows after it declared its parts asks for another pass, so a memoized view around it runs rather than replay a stale scroll bar
ecd156585 ui-contract: on the focused menu bar a plain letter is a title's access key: it opens the title it marks, and one no title marks runs no shortcut
5c5e0d52c Font samples: one baseline in a fixed band, shrunk only where their ink would leave it (#3144)
7089e2de8 color-ui: ColorPickerDialog::working_rgb puts the host's working RGB in place of the built-in RGB in the Full and tabbed layouts
3ccf14ed9 color-ui: the Color panel rounds only what its channel fields show; an edit of one channel leaves the others unrounded
173d63d91 fw sync: cargo fmt --all (including drift that predates the sync)
08e5ee8eb fw sync: L-324/L-314: PageInsets::Theme (the port's inset spacing, opt-in; the theme side-margin trial opts in), PageView::go_to_place(GoToPlace::GapAbove) for production's Go To, PageView::fill_screen for full screen edge to edge
85659945c fw sync: L-323: StandardPrompt::alternate_role (production's order), DialogButtonLook::Quiet (ghost Cancel and Don't Save), StartScreen::heading_sections and pinned_first, ToolbarDrawer::grouped_tools_drag opt-in; tab marks resolve outside the memoized tab row and join its key; a grip floating window keeps no title-rule band
e9d0bb026 fw sync: L-322: the filled dropdown triangle (DropdownArrow::Triangle) and PdfCraft's document tabs (DocumentTabLook::Shrinking: shrink, inline gap, marker dot for modified, 22-digit name cap) become opt-in env values; upstream's chevron, whole touching tabs and '*' stay the default
bfe51c762 fw sync: L-320: parts_group takes the nested case: a part group registered under the body group keeps its own group, for the header's hosted cells as for the parts
cd7273940 fw sync: a key whose bindings are all disabled is still declined to the app
5d812cfef fw sync: command-search story chord off Cmd+K, which upstream's menu-row trial binds
dd35b1e88 fw sync: resting-pointer popup trial names its list (upstream's PopupList::list)
33d7e8d65 fw sync: unsaved question long-text trial finds the question as an alert dialog
a0035430e fw sync: L-72 hint-alone tooltip becomes opt-in (CommandState::hint_alone)
24f40451a fw sync: L-318: Escape over a draft in a modeless window reverts the field first
ea9e4e3be fw sync: L-321: the merged middle autoscroll ends on the window's blur event
6a7488ad7 fw sync: task bar writes its place from the drop's Commit (upstream wins)
df81ec186 fw sync: L-324 Creative page spacing back to upstream's default (section gap, roomy margin, rail side); the port's inset spacing is left for the opt-in variant (fwg-optin), whose trial a_page_fitted_to_the_width_keeps_the_theme_side_margin must opt in
9c2d4522b fw sync: L-324 comment rows keep upstream's one editor-focus mechanism (EditShown); each card menu opens once per pass (duplicate stage ids)
321dbe667 fw sync: L-324 Creative page spacing back to upstream's default (section gap, roomy margin, rail side); the port's inset spacing is left for the opt-in variant (fwg-optin), whose trial a_page_fitted_to_the_width_keeps_the_theme_side_margin must opt in
4db8b3782 fw sync: docs: components index regenerated by its generator; api-map shape (new setters name old = new, password_strength as a call rewrite)
76be9d70a fw sync: L-324 comment rows keep upstream's one editor-focus mechanism (EditShown); each card menu opens once per pass (duplicate stage ids)
a2760e0c0 fw sync: test frames stamp held modifiers on generated pointer buttons (F-144 press modifiers vs upstream's held-modifier presses; imaging pins/lasso/vanishing pass)
89ce8fcbf ui-widgets: a hexagon swatch picture's mesh takes the textures' UNTINTED tint, not a raw white (design-lint raw-colour/const)
49646c794 fw sync: the comments panel's lead card names its message menus once a pass (a row built twice opened two menus on one id)
6a8983598 fw sync: design lint clean (107 findings to 0): non_exhaustive or closed list, #[prop] setters, layer imports moved down (begin hook, line-break chord, scroll move targets), session memory, story consts, marker exceptions with reasons
aa46fe4bb ui-widgets: rustfmt the merged swatch face
9afb1ebf5 ui-panels: the grip trial reads GroupGripOutput::drag_started as the bool next's grip reports
46b14822a font-catalog: app start helper, font store watch and changed-family list (#2781)
a29bfe866 ui-canvas: the rulers' corner drag proposes where it lands at its start and its release, so a drag of one move sets the zero point
f53608910 color-ui: a disc drag takes the host's colour as its echo when it lies where the drag put the marker, so a host that keeps the unshown channel from the press or stores its own colour model is not cancelled as an outside change
eeb5b2619 font-manager: previews from the font preview service; font-store downloads through it (#2782)
295c967c6 ui-shell: a document window's fallback layer moves by its title row and reports where a move or resize took it, so a host that places the window keeps it there
cbf17f41b ui-panels: the rail's locate trial is reached by a module, against the rail's Tracked slots and its reveal and mark props
b84703ac8 ui-contract: a container level keeps its run's core, hooks, plans and body Ui on the heap, so sixteen nested containers fit a 512 KiB stack in a Windows debug build
7e2b61dba color-ui: SwatchPair activates a well only when focus moves onto it, so a well keeping the focus leaves the host's choice of the other well
2bad4aebd ui-contract: a field label without text names no control, so a custom row with an empty label leaves its control the name its own label gives it
f712b3306 ui-contract: a plain press on a row or tile outside the selection picks it when it ends as a click, so a drag carries the row alone and leaves the selection, as the old apps picked on the click's release; a selection change restates the rows the pass measured with the old one
ac24169b1 ui-contract: a press that moves past slop leaves the click run, so the press after a drag is a single press, as only clicks counted toward a double click in the old apps
ebf925452 ui-widgets: a tile grid that takes its width from its content asks for at least one card, so a measuring run lays it out at the width it shows in and a stack sized from it holds every row and what follows
0f14d9049 ui-widgets: an equal link the host turned off again before the pass that sees it turn on sets no field to the first one's value
71a5b3b0b print: in the sections layout an app's row with a printer setting's key (copies, duplex, grayscale) takes the value automation sets and is the one the listing reports, as the printer panel shows only the printer there
3b629776d Testing runtime runs background work inline: InstallSpec::jobs and Env::jobs_of
f9d38c866 fw sync: L-319: moves a region passes its fallback collection stay its keys (page and end included); the rows' own scroll moves page
7d61a755c docs: regenerate contract-api for Reader::split_unit and contract::response_of
99ea0489a fw sync: workspace check --all-features clean (review panel state literals, unused results and imports)
438ae3da7 fw sync: review/pages/draw/text compile: comment selection and status fields, note popup scope, story binds, memoized-view preview trial, trials registered
a297595c4 fw sync: automation dock_method takes PanelSpecs; platform BrowserFiles writes to an embedding page (page_saves) before downloading, web-host's page_save feeds it (L-317); compile fixes in canvas/patterns trials
8a620b3d6 fw sync: ui-canvas and ui-patterns libs compile: WheelScheme::Document arm, one scroll-bar same_value (both tolerances), region drop mark for a split, memo props on env values, story binds, unused imports
dfc8ff521 test(platform): restore rtf_target_prefers_text_rtf test (wordcraft#7317)
975ccce6c fw sync: ui-panels compiles: one Dialog::least_width (the container prop), story commands on CommandStates, tracked rail slots in a trial
ccbd4283b fw sync: ui-widgets compiles: FaceSpec fields, story binds, registry-visible story modules, text_x, deprecations
4335d1ef1 ui-widgets: a document tab dropped on a strip with no tabs, or past its tabs, is reported: DocumentTabsEvent::DroppedInto's over is an Option
47d6a3e09 ui-panels: a tool rail inside an arena (a dock) reports its Edit Toolbar button: the grid's run sends the button's response, and a replay reads it back through contract::response_of, which also knows arena members
bf91cb7e1 behavior: the number Reader reads picas and points written together (2p6, Reader::split_unit) and a percentage after * or / as a share (3cm*50%); length readers register p
1385f0ff3 ui-contract: a held Escape cancels once: its repeats raise no Cancel, focused or not
7f53dfd8b fw sync: ui-contract compiles: one AccessKit tree store (upstream's settled tree, the port's view over it), journal calls on upstream's signature, Span strikethrough, filtered Source borrows; the settled-region rename trial on a memoized view
308478fe0 fw sync: restore platform's appearance and browser features; resolve Cargo.lock
a48cd842f fw sync: apply the areas' owed lines (platform/web-host files, clipboard cfg, tooltip, canvas, stories gating, balloons selection, recent list, specimen, preferences re-export)
daae7fcd5 fw-patterns: resolve the origin/next sync for ui-patterns dock area, form dialogs, preferences, preset browser and their trials
a1be794fe android-res: mark the table partial when library records are omitted
fd94fad30 fw-pages-review: merge origin/next into the port's pages, review and text editor
58f35e8ec fw-panels-shell: resolve the sync of origin/next into the port's ui-panels and ui-shell files
c6250e81b fw sync, fw-widgets: merge origin/next into the port's ui-widgets
0228bb256 fw-canvas: resolve the origin/next sync conflicts (canvas autoscroll, wheel schemes, rulers, editor part, crop scope, task bar kinds, transform box handles; trials and stories)
80ce862b7 fw sync fw-contract-core: resolve ui-contract contract/primitives conflicts with origin/next
84dd65f48 fw-sync fw-menu-misc: resolve menu, behavior, automation, font picker, theme, color-ui, docs, vendor, Cargo.lock
a9126c6a7 fw-sync fw-contract-runtime: resolve ui-contract runtime, egui glue, fonts, style, testing kit, manifest and runtime trials
98d01dfc3 fw-services: resolve the sync of origin/next into pdf-port/gaps
f6b339e02 android-res: mark the table partial when a string pool or value is unreadable
443a05c86 docs: regenerate contract-api for SubNode::run_link's removal, EditorStep, ItemLayout::set_fit, Fired's place and FrameOutput::dimmed
090fd7058 font-manager: the install-queue trial counts only install runners
897b45faa ui-patterns: the hosted New Document keeps the details rows' memory and lends it to the dialog (NewDocumentDialog::memory), so a field automation sets is noted where the rows read it, and its preset field applies the saved preset's card
8e05942ef ui-patterns: the hosted Save PDF's automation OK in the preset editor files nothing for an empty name or one the app's check refuses, as the footer's OK is off then
1bd0567df ui-patterns: the hosted Save PDF keeps both passwords through an automation write to another setting, which reads the settings back from JSON that never carries them
2c718b583 ui-panels: trial: a drag from a collapsible section's title moves it in a section stack, through a header menu and a drop slot, and a click there still folds it
b2d2a1e94 develop: expose Camera Raw Detail's resolved parameters (#3097) (#3101)
f812d2f59 android-res: report skipped entries when the entry limit stops a type
b012adb0f android-res: mark the table partial when ARSC chunks or entries are skipped
ea77c1676 Quiet check box: white mark, small corners
8c4e115ea docs/contract-api.md: regenerated (document_face's job and the face bytes; also ColumnSpec::at_least and data_fill_tiled, which next's copy lacked)
1fadb0345 Document faces: one queue per context, lost loads fail, replays mark faces drawn, one byte budget
6a1d13c76 ui-panels: the grip trials compare within a named slack (design-lint test/literal-size)
d6d8e4c85 ui-widgets: a link in a sentence is one Link node: the paragraph is its own node, and the link's words on each row are their part's Link node below the row's run, which takes focus and the click; SubNode::run_link is gone
83b424571 Param::beside_previous: rows that share a line in a parameter form (trial)
e117e783c TransformBox::round_handles: circular corner and side handles (trial)
fb939c5c7 text: the text editor takes its inputs and commands in arrival order with editor_steps, so typing after a commit's chord is not the session's
f6fd9623b ui-contract: EditorStep is non_exhaustive
42d6487d9 ui-panels: a prompt with no default answer opens with no button focused, so Enter answers nothing
d6b9b77de ui-widgets: HelpEntry::with_tie_order orders entries that match a query equally, before the label's length, as production's result kind did
28a5d985f ui-contract: a key press whose chord binds only commands disabled where it lands is reported in FrameOutput::dimmed, so the app can say why nothing ran
4b7937c27 Tile: quiet check box (Checkbox::quiet) and an outlined selected band tile (trials)
de01d25a5 font-catalog: name the per-file slot type (clippy type_complexity)
ef11aa243 MenuBarUi::compact / MenuItem::compact: rows a text line tall (trial)
67409c465 Font picker: sample faces load off the UI thread and stay while drawn lately
5ddae9f82 Hue/Saturation: AdjustedFn borrows for the editor's lifetime; swatch trials drop an unused import
cee746e97 ui-panels: rustfmt the grip's drag-start condition
96f769a6e imaging: the magnetic lasso fastens points only when the pointer moves, so a hover rerun by a key under a still pointer leaves the outline as it was
b634808ce Dock grips on next pass design-lint; the facade table lists main's new crate features
8242da0a9 ui-canvas: Space moves the Zoom tool's rectangle while held, once let go since the press, and the release zooms to the moved rectangle
25c3f6490 Hue/Saturation: name the adjusted callback type AdjustedFn (clippy type_complexity)
cfb37fb71 TileGrid: insert gaps are muted at rest, accent under the pointer (trial)
e55137b72 Select and Mask previews again after a document edit
52b338169 PaintBar: show the Mixer Brush load as a colour, image or checkerboard
a309c143a Hue/Saturation: pick the range with seven round swatches
7ada3dc77 Swatch: round wells, split, spectrum, image and empty fills
8f82948bc Ruler: Shift snaps to 45 degrees keeping the shorter leg
a8a3a594d Curve graph: one grab radius for every point, past the plot edge too
0a33cd921 ui-contract: a command a key fires carries its event's place, and Cx::editor_steps gives an editor its inputs and its scope's commands in one stream in arrival order
276161b92 Shortcuts reference opens at a moderate fixed height; FindBar options icon is the adjustments glyph (trial)
c7e1e4126 Document tabs: a long name is cut to a compact tab, and a tab drag shows no badge (CollectionProps::drag_badge; trials)
baaa53c80 Comments: an edit's button reads Save, a new comment's Post (trials)
2469c6b61 MenuBarUi::icons / MenuItem::no_icon: text-only menus without icon column (trial)
b5e0109bf Table::plain_header_menu: the header menu lists Reset Columns without column commands (trial)
d02b07100 FindBar: a plain integrated field (no clear button) and a sliders options icon; TextField::clear_button (trial)
6468047ea ui-contract: a key the key stage takes from egui (a claimed chord, the canvas's Space) stirs the pass, so a held pointer drags and hovers again when it goes down or up
8fd8840fc craft-develop: doc links to private or feature-gated items as code (next denies rustdoc warnings)
c6452d95e ci: allow excluded vendor trees in the unreached-rs check
d69bdaf0b style: drop rustfmt churn on the rest of context.rs
9fcd7ea22 hayro-interpret: a form that draws itself is not content-truncated
4a0cbec2c ToolRail::edit_icon: the Edit Toolbar button takes the app's icon (trial)
cafa35b0b list: a row sized to its content is as wide as it measures in a span layout, and a thumbnails-only row is
c23779278 ui-contract: a wrapping row sets each child on the middle of its line, which is as tall as its tallest child
39bb3a06f craft-aaf: document the property record fields and errors (next denies missing docs)
a69b2ca3e DocumentTabs: the trailing control ends at the visible edge of its row when the bar leaves less than the row's width (fixes Open under Discord; trial)
8ea9184ca Cargo.lock: facade members for craft-android-res, craft-cdr and craft-png
9f42cd9b8 FloatingGroup::grip / DockConfig::floating_grip: a quick-action strip's title row is a bare grip (trial)
1d93a4708 dialog::set_close_button: an app takes the close x off every dialog's title row (trial)
eaa7808df ui-patterns: the preset browser trial toggles the lead with Ctrl+Space, the chord that toggles on macOS too
fa918b109 DocumentTabs: the trailing control follows the last tab when every tab shows (strip takes the room less the control's last width; trial)
763fbc909 font-catalog: adding fonts neither starts nor waits for the scan of the installed fonts
402dc5769 StartScreen::pinned_title: an app names the pinned folders section (trial)
8945a9b57 ui-contract: drop_spot reads the row count from its outline
dbcf3998a craft-pdf: the cos suite uses the crate's testkit instead of loading the file a second time
e0bbb110d HeaderStrength::Overline: a small capitals group name; the print dialog's section names use it (trial; print test premise updated to the capitals)
935607fff ui-contract: a drop on a grid's group header lands first in its group, now that grid places count the header rows
134abcc8d Build the tests next lost in the sync into the crates' test executables
e1c00aac1 docs: regenerate contract-api for the drag payloads, CollectionEvent::Picked and CollectionKind::Picked
d5b799341 Note pop-up: the title band runs under Close (trial the_title_band_runs_under_the_close_control)
1fa27301b color-ui: builds and tests clean with default features: the stop readers only the editor uses are ui-only, and the palette tests have serde_json
e7e922903 ui-contract: CollectionProps::drag_payload takes self by value without mut
f84629432 ui-contract: a scope's initial focus on a roving group arrives at the group's arrival member, as Tab does; SwatchPair makes its active well the arrival, so opening a panel on the pair activates no other well
a2dbb5795 text: CharacterControls names its axes button Variable Font, the word made for it
871d18483 ui-patterns: the row menu shows the host's entries as the entry popup does, so a plain row is a menu item and only a checked one a check box
b008cd58f Tabbed document dialog (Document Properties): aligned caption column, controls fill the row (Inspector scheme; trial)
b1218a5fb ui-patterns: a grid reorder or drop counts the group header rows, so a drop in the left half of a tile under a header lands before it
bde3742fd StartScreen: offers_open(false) leaves Open out, section headings are page headings with no rule, Pin Folder is ghost, a pinned folder's Unpin is text (trial)
abb4f1e9f ui-contract: a click on the sole selected item reports CollectionEvent::Picked, and PresetBrowser reports it as Select
bd5798e9f Calendar: Sunday's weekday head is no longer blank (table asked for 2026-10-32), two-letter heads (NameLength::Short), Today centred and Clear only with a value (trials)
3c9bd2e48 ui-contract: a drag carries any number of extra payloads, one per type; PresetBrowser and SwatchesPanel take a host drag payload beside their own
3815fa4da Note pop-up: the pale tint is painted behind the body over the window surface, with the surface's bottom corners
d143c8013 craft-develop: retain same-colour clipping dependencies
a7ff91cd1 docs: regenerate contract-api for PointerFrame::pick_pos
aec1c3589 ui-contract: a modal layer keeps the document canvas's claimed chords from it while nothing is focused, as it keeps the arrows
9f852f6dd Note pop-up: pale note-colour body read in the light palette, small end-aligned Reply/Cancel/Post (trial the_reply_button_sits_at_the_end_edge_of_the_window)
f671a2f62 ui-contract: Cut in a focused selection part copies its selected text, as a label's selection does, instead of running the app's Cut
77807fd1b craft-metafile: keep the outer file's decode budget for a nested PICT
51cc84a7c color-ui: a disc drag's echo takes the unshown channel from the host's colour before, so a brightness disc over 8-bit colours is not cancelled as an outside change
20e9257bd ui-contract: an Escape over a captured press cancels it whatever the modifiers, so Cmd+Escape ends a borrowed tool's drag
56a174652 ui-contract: a drop lands where the button went up (PointerFrame::pick_pos), not where the pointer ends the frame
7b5af94f1 Dialog footer: Cancel is a ghost button beside the accent answer, as production (trial a_footer_cancel_is_a_ghost_button_beside_the_accent_answer)
9a1040272 ui-contract: with nothing focused, the document canvas's claimed chords hold, so a tool takes the Delete and Backspace it claims instead of the app's Edit > Clear
22e56d81c NumberField mixed_word: no unit beside the word, so 'Mixed' keeps its room
35a14126a FormatText: the size field carries no glyph label, as production, so a narrow pane keeps room for its value
9a128c770 dock: a group grip reports its drag's press point, which a tool rail's grip puts on the drag instead of reading egui's pointer; the grip trials assert relations to the grip shape
d21395f48 theme: regenerate the vectorcraft-modern token dumps for line.grip and the grip shape
09d4ef706 NumberField mixed_word: a mixed field can read 'Mixed'; FormatText size reads it; colour well stays content-sized so it takes no share (trials a_mixed_number_field_can_read_mixed, ..._reads_mixed_in_a_272_point_pane)
f61e73e0d ui-contract: the lead trial reads the last lead move from the end
9f99a64cf docs: regenerate contract-api
7668530ec FormatText: the size, colour, list and alignment row uses the tight gap so the size field reads its value in a 272-point pane (trials: ..._reads_its_value_in_a_272_point_pane, ..._reads_mixed_...; fail with 38.5 pt on the old gap)
6acd584c3 ui-contract: a lead whose row leaves the source takes the host's current item without a report, so PageList reports no Go the user never made
7436a99e0 color-ui: a named ColorWheel marker reports its colour's hex as its value
4a8fff263 ui-canvas: a tool drag held in the band inside the canvas edge scrolls the view, under the one edge rule hovers already use
333ba0f68 table: a grid drag starts where the pointer already is, so a release right after the drag start commits the move; a cancel names the value the grab started from
5bfcb2249 ui-contract: a press of any button runs on the click run, as the old apps counted clicks; only a primary press carries the count
0d980bf74 ui-contract: a link field label reports the Link role, not Label
89c28d42b ui-patterns: the hosted PDF dialog takes the open and permissions passwords as automation fields (never listed), under the rows' standard rule
7a3c1b4b6 craft-print: trial that the sections layout of a hosted print dialog shows and switches its sections
e1632f95f ui-patterns: HostedParamDialog::can_reset compares with the reset target (opening values with reset_to over them), as reset does
116029a89 ui-patterns: trials that drops on a preset browser reach a tile and the empty notice in the grid view as in the list
43752a37c ui-widgets: the toggle-state trial also covers a toggle given a shared (Mixed) value
d8dd9d94e ui-contract: an accessibility click on an item of a collection that activates on a click activates it, as a pointer click does
5c8ac4b47 ui-widgets: a link in a row's cell is as wide as its text, so a press past it selects or drags the row; Hosted::role
932e8df74 Preferences: one scrolling page of cards, segmented short choices, a search with its match count (L-311)
9b89c55e5 ui-canvas: trial that two clicks in one frame reach the tool, the second as a double click
f3119648e ui-contract: a multi-finger tap allows its own finger drift (MULTI_TAP_SLOP, 16 pt), not a press's slop
33be35625 develop: wrap negative Camera Raw hue turns (#3089)
9688cdf37 develop: keep Camera Raw quantile indices in bounds (#3088)
d2a776ee5 metafile: expose frame and source-unit facts (#3071)
1c137fea6 image-codecs: share bounded PNG sRGB metadata writing (#2941)
12d0cec13 svg: identify the first element in SVG sniffing (#2909)
b20e66f65 avi: share framed MPEG and AC-3 audio scanning (#2928)
4499f2f82 exr: share bounded PXR24 scanline reconstruction (#2935)
d5493fe6e exr: share bounded RLE and byte prediction (#2921)
0da914f4a video: share seekable raw APV access-unit ranges (#2925)
fb04a21e9 video: allow a lower Motion JPEG track allocation limit (#2916)
1b8d0150d Normalize PostScript literal string line endings (#2844)
66ed98f1f usd: accept scalar external arcs with prim targets (#2707)
4255a185d Resolve nested DrawingML compatibility choices (#2851)
8adecb1cb Preserve short SVG elliptical arc sweeps (#2835)
33e36c8cb Preserve finite SVG smooth control reflections (#2832)
684a0a90f develop: bound shared Camera Raw sharpening detail (#3091)
49f38d0e3 tiff: expose plain RGBA16 without the raster facade (#3082)
b3970d12a PageRangeOptions: document dialogs stack the radios and end the range in 'of N'; SelectedFromTo arrangement; subset_select for a row of its own (trial ...stack_under_their_heading_and_count_the_pages, does not compile on the old code)
1dbdd824a PagePreview: a framed dialog with the file's name, the page's place and a close x, not the full window (trial the_preview_is_a_framed_dialog_with_a_header_and_a_close_x; does not compile against the old code: no title)
54af1b5ab font-manager: drop a leftover DIAG line from the picker preview trial
1789a0541 cargo fmt on the merged font-manager files
0f77a2239 ui-contract: the empty-commit IME trial matches EditorInput::Newline by its shift field
6f69df0e4 docs: regenerate contract-api.md
6d02a9d79 craft-metafile: format pict.rs and lib.rs
cbe886617 keymap: CommandRef equality follows its binding key, so no parameters and an empty object are the same command
8a7f01b82 ui-widgets: a Select whose value names no entry reads as the placeholder its face shows
a37e40277 ui-contract: a hidden field label, and one drawn inside its control, makes no Label node of its own: it names its control by its text alone (FieldLabel::shown)
73701aca9 ui-contract: a run too large to show (past MAX_EXTENT) declares no parts and is not parted, so its parts read idle, as a canvas on a very wide window reads its background
69751723d ui-contract: a clipboard command reaches the focused component in its event's place among the frame's keys and texts, so an editor reads a paste in order with the typing around it
39f86f764 behavior, ui-contract: Enter reaches an editor as EditorInput::Newline { shift }, each routed intent keeping the Shift of the press it came from, so an editor tells Shift+Enter from Enter in the order they came
a2ca28d81 automation: ControlClient::connect reads a refusal the server wrote at accept as the sign-in's answer and returns its reason, without sending the sign-in
67b68a638 ui-contract: a button over a two-state value (a shown bool, a toggle command's checked state) reports pressed off as well as on, so an off ToggleButton says it is not pressed
e695aadc6 ui-contract, ui-widgets: a number field's value reads the unit it draws after its number ("160 pt") through Access::unit; trials that read a field's text read its text runs (AccessTree::run_texts_of)
0fc19936f android-res: report partial XML at event and nesting limits
e16f2f2be android-res: bound ARSC framing to declared headers
22a80f296 develop: extract PhotoCraft Camera Raw Filter into craft_develop::camera_raw (#587) (#3081)
408c6f95d Titled header: the close button names its panel (Close <title>)
ccd093d23 craft-metafile: retain pattern fills, prioritize emf/wmf in parse, report dropped drawings
4f2d0f5a4 sqlite: cover a WAL frame checksum mismatch
32300bf1b FormatText: narrow-panel size field regression trial (passes in isolation; the app's clip is not reproduced here)
72211adc8 android-res: split locale formatting from configuration qualifiers
5837a568d Titled group header: title and close button centred on the header line
b3d3c18ba ui-contract, ui-canvas: a nudge carries the keys held when its arrow went down, in the order of the frame's events (Intents::nudge_held, ToolKey::Nudge::held)
9c58208bb ui-contract: an Alt+letter the keymap binds to an enabled command runs it instead of opening the menu with that mnemonic
f9cbb0a72 ui-contract: a control given selected outside a collection reports it as pressed, so the rail's current tool and the other tools say whether they are the tool
0bf4cb99d craft-metafile: address CodeRabbit feedback on bare v1 routing and decoding
9e8287c5f DockGroup::titled / DockConfig::titled_headers: a group header shows the shown panel's title, count and a close button in place of the tab strip (trial a_titled_group_shows_the_panel_title_and_a_close_button_instead_of_tabs, seen failing with the flag unwired)
bfdaa2b71 android-res: share AXML and ARSC wire readers
7e88389c8 Document legacy JPEG zero-marker handling
9f18c4e96 tga: share footerless top-left sequence writing
5239b47ad Share the legacy EMF and WMF EOF diagnostic
b06acb3d0 Use fixed-width AAF property index chunks
57999e775 sqlite: use fixed-size chunks in WAL fixture
db9b6ac70 craft-metafile: address review feedback on PICT reader and formatting
7e5b60317 ui-contract: trial that files hovering in the pass right after a drop drag and drop on their own
b42062884 Scroll body: a reveal of a rect already in view queues no jump, so no repeated pass every frame
42f8006cc font-catalog: Catalog::versions lists a face from every cataloged file that has it, the index still listing it once
75279bac6 craft-metafile: add QuickDraw PICT v1/v2 reader
0720c236e docs: regenerate components-index.md from the registrations
03df09a1c hayro-interpret: a soft mask's group draws within the forms where the mask was set, so a mask drawing the form it masks is not a form drawing itself
363bc05ed font-catalog: a language the typographic family records leave out keeps its legacy family name in localized_families
7039de09f plotter: share classic HP-GL command serialization
983a11678 tiff: share the plain RGBA16 writer
e084f8c1f jpeg: share strict EXIF orientation framing
796a26552 Share borrowed AAF property stream framing
28ee9742f sqlite: share committed WAL checksum scanning
b2bbfa6f5 develop: share legacy Gaussian curves and LUT tails
52d1ec740 menu, dialogs: rustdoc -D warnings clean: no redundant link targets on MenuBar::show; the service docs name the private with_chosen_type without linking it
2dcc5763c ui-contract: NodeKey::raw is test-only, as AccessTree::none is, now that the access tree builds without the testing feature
e751a7a49 craft-cdr: refuse previews that preview cannot read whole
b9144731d api-map: ZoomLadder is a removed entry and ZoomScheme its own replaced entry, the shape the codemods read
603697c7a automation, draw, imaging, menu: clippy 1.100 clean: no must_use on builders of must_use types; tests return directly, take Copy rows uncloned, name the remembered answers type
0beefd0d7 ui-panels: clippy 1.100 clean tests: collapsed if; the scopes trial drops the id import ruling (c) left unused
d017edbad ui-canvas: clippy 1.100 clean: no must_use on builders of a must_use type; the task bar's More story drive finds its button without expect
c7059b3a3 ui-shell: clippy 1.100 clean: the start screen's app section type named; no must_use on a builder of a must_use type; test types named, slice from_ref
e85a6d3fe ui-patterns: clippy 1.100 clean: no must_use on builders of must_use types; collapsed ifs; Copy fields not cloned; the drop builder type named; the PDF export automation trial is two trials
03ef469d9 text: clippy 1.100 clean: the kerning row takes its side label built, as the leading row does; the axis value type named
7577f7a56 color-ui: clippy 1.100 clean: no must_use on builders of must_use types; pixel loops over as_chunks; harmony and pair closure types named; no needless borrow
5ad0e5288 layers: clippy 1.100 clean: the row's eye and lock cells in with_toggles; host drop place and footer types named; no needless borrow
368b3c930 layers: LayersEvent::Filter boxes its edit, as the other large edit events do, so the event enum stays small
f812df100 ui-contract: clippy 1.100 clean: a row unit reports how it leaves as one (cut_first, rank) pair; the items' place_rows takes the detail through ItemCx::take_detail and a cell_width helper; face and whole-paste types named; tests return directly and take the last visibility by reduce
122219411 behavior, platform: name the history's weigh function and the clipboard's deferred read (LaterRead) for clippy's type_complexity
8faa8a02c keymap: clippy 1.100 clean: no needless borrow in the import's tool check; the hold-binding trial is three trials (text form, off and edits, clashes)
0c33c374a platform: the browser file service answers Files::pending_pick with the oldest pick it runs, never a sheet
1507e63e6 develop: test local tone mapping across crop and tile coordinates (#3057)
430b7425f craft-cdr: read the picture a .cdr file keeps of its drawing
8300e3d44 fix(develop): apply deferred DNG linearization after OpcodeList1 (#3050)
1e48d9702 ui-contract: the suite's row press avoids pointer-only press parts of the collection, its children and ancestors (a grid's column edges)
b1af55834 ui-patterns: NewDocumentDialog print_resolution and screen_resolution are quantity props again, set over the setup through DocumentSettings::set_resolutions
bcce1c9f8 ui-patterns: DockArea::continue_move is a quantity prop taking (panel, grab)
83f452d66 ui-contract: the icon-fill trial's white tint is marked; icon-trial-fixed-tint covers 4
b950fb234 ui-patterns: PresetRow::preview with its counted anonymous key goes; callers give a keyed face through preview_face (pattern swatches by texture, contours by their points)
ccda1c03b imaging: the Filter Gallery session carries its layer-id counter (next_layer), kept across undo, instead of a process-wide static; ids are unique within a session
b2544efa2 automation: control lookup moves to access.rs (Target::rect, ControlMap); At::resolve takes a control map instead of the access tree
e3d43eaf2 framework: button presses read through contract::clicked; the command registry names layers as runtime::egui::areas::LayerKey; Pasteboard declares its overlay (its rename story covers Open); rename_field returns the TextField for callers to show; the tool host's cancel mapping no longer names the pipeline's Gesture; quantities declared on roundness and a field's named number; fixed sample and paste-picture colours from Srgba and the test kit
fe14ea0bc TitleRow::icon_ink; a dialog's title icon is drawn in the accent (trial the_title_lock_is_drawn_in_the_accent_colour, seen failing)
8e1aaea8d Print preview: the paper stands inset in its well (trial the_paper_stands_inset_in_the_preview_well, seen failing; placement trial's premise updated)
f3bbd5125 framework: trial rooms, drags and story offsets the port added are documented consts, not literal sizes; the tool-rail story's unused memory marker and its exception go; a caption trial asserts a relation
bd7e55c42 framework: FacePaint and the preset browser's host closures key through the MemoProps and PlainData derives (covered_by, every_pass) instead of hand impls; the Font Height Reference glyphs are consts, and their segment helper no longer shares egui's stroke-taking name line
f61a28ff0 Dialog::title_icon; the password dialog's lock stands before its title (trial the_lock_icon_stands_by_the_title_not_the_message, seen failing)
35535b772 ui-patterns: ParamDialog keeps its footer rows' states and pending flip in one session slot (runtime::memory::with_session), not egui temp data; the chord story narrows its room in testing::memory::scoped
9a4b9efb4 framework: setters that only store their argument are #[prop] fields the Props derive writes (TaskBar, FloatingGroup, ToolbarDrawer, CommandPalette, PresetBrowser, PresetPicker, AppShell, Radio, Swatch, Text, ListRow); ToolbarDrawer::mark and reveal take the tool, as ToolRail's do
eaa343366 framework: public enums and records the port added are #[non_exhaustive]; HoldSetting, PopupRegion, ParamPresets, CardSource::preset and WebDocument::vector take constructors for the crates that build them
0ba9cb07c PageView::pasteboard: the colour round the pages (L-309 full-screen black; trial seen failing)
ec7d40e3e Theme: page spacing back to production's 70/28/14 comfortable (L-310; trial comfortable_page_spacing_is_production_s)
d5f3da32f Revert "Theme: page spacing values keep the app's current geometry (the floating toolbar covers page text at a 70 pt side)"
ff6ba8b21 ui-contract: a link side label keeps its caption's type role and takes only the link's ink, so it takes the room its control measured for it
91012d712 ui-contract: LayerRole::InPlaceEditor marks a layer that edits its owner in place, so the owner reports no open or expanded state; the pasteboard's rename field is one
7d755a474 ui-contract: a toast card the user ends while it holds focus hands focus to the next card, else the one before
c5de7eaf6 Theme: page spacing values keep the app's current geometry (the floating toolbar covers page text at a 70 pt side)
df3f0f022 craft: regenerate the facade features table from the member manifests
55512ecf7 ui-patterns: a section list shows its sections at their natural height, so it owns no scroll in a dialog body that scrolls
b0c52a343 table: Escape ends a running resize, fill or move of the grid, which took the claimed Escape before its grab could see it
694f73388 dialogs tests: the chooser's service trial expects Picked::Saved for a Save (contract fl.F1) and clicks once the chooser has laid out its listing
706ddc4f7 craft-develop: track clipping at bilinear taps
4d15f54b2 font-manager tests: the ended-preview trial waits out the eviction grace and keeps a component showing, whose run drops the evicted memory
8d583f5af font-manager tests: the corner action trials give the tile a pass to see the hover, as a tile declares hover-revealed parts from the last pass's hover
8527eab46 font-manager: the alternates popup's grid takes its natural height, so it never shares the popover's scroll
d66deae98 PasswordDialog: bare password field (placeholder, no caption column); Open waits for a typed password
35c78eaf3 Summary card: the description wraps onto its two reserved lines
1f2850aad ui-shell tests: the reveal pin trial holds the button where the pin was taken and lets go elsewhere, so the release ends the pin
497cc7758 keymap: a reserved chord is never bound, so a default binding on one is dropped when the keymap reserves it
03c211ab6 Start screen: pinned folders above recent files; a Clear button on the Recent heading
fca492c03 Page view spacing from the theme (space.inset.page_side/page_margin/page_gap: 70/28/14 comfortable)
cee82a0c4 Theme: stroke.focus (2 pt) for the focus ring
0455a4610 draw tests: the distort pad shows its canvas at the room it allocates, as the Free Distort dialog does, so the content-sized dialog body gives it its size
92f0457a8 behavior tests: a picas reader reads 1pt+12 as thirteen points, as its bare 12 reads one pica
67fbae7db behavior: Workspace::rename is an edit under upstream's revision rule: it moves the revision and keeps the generation, and the mutator walk covers it
43b9cf1e0 Document print dialog: compact inline printer rows, caption sections, no repeated radio heading, untitled section
52d2560c1 image-codecs: share ISO-BMFF brand prefix framing (#2638)
6ca10b60f Print preview tints the page pictures grey for a grayscale job (as production)
67e3da656 ui-panels stories: the tool rail stage puts the rail's room at the reading start, so under right-to-left its flyout has room on the mirrored side
0d16d9271 ui-canvas: a closing page rename field fades out where it last showed, and a field with no place yet is never placed at a non-finite rect
301120c9e ui-canvas tests: the outline trial runs at a zoom the canvas does not pixel-align, so its fractional pan reaches the outline
2d5190511 ui-canvas tests: the grid order trial leaves out the empty shapes a paint slot leaves behind
6dfcb167f ui-canvas: a resting pass that hovers for a stir takes the hover owed after a press, so the selected tool gets no late hover once a momentary tool lets go
8206a076a ui-contract: a focused stop that is no longer shown leaves focus nowhere; only a removed child of a collection row hands it back to the row
d20e7ff88 craft-develop: bound binned CFA phase tables (#587) (#3037)
8d4065f9e craft-develop: demosaic white-balanced mosaics with a clip mask
b2df7dff3 Select and field arrows are the small filled down triangle, as production
446d51c8b Percent number fields read one decimal and the sign beside the number (L-308: 100.0%)
fb144cf56 MenuItem: an unchosen radio row shows an empty ring
d6d67f79e Dialog footer: an alternative answer (Save as...) stands between Cancel and the accepting button, as production
3a34b8b2e docs(develop): clarify sensor and local tone consumer contracts (#3038)
ac052352b Print preview: the paper is white in every theme, so the gaps between pages are paper
a303e0440 HostedPrint: the app can mark its settings invalid, which disables Print
7631ed550 craft-develop: extract sensor and local_tone stages (#587) (#3021)
3d31bbfba Document dialogs show a small page picture with its pager and notes beside it (L-304)
4da205a0c Notice: a centred notice (empty state) draws a large, quiet icon
59c9ebfa6 Report DNG linearization in decoder comparisons (#3030)
9c1bc7aff Trial: a SpaceBetween row in a dialog keeps its children apart
60c1698c3 Trial: a context menu near the window top stays inside the window
fe7701007 Dialog: Close in place of OK and Cancel while the fields are unchanged (L-307)
eeb754615 Preview pager is centred under the pane in a stacked or document dialog (L-304)
f8c3cef36 craft-develop: widen raster blur width arithmetic (#587) (#3019)
dd90a5637 Document dialogs fit a 900 px window: radio captions share their options' line, no zoom row; zoom row under the pane keeps its room before the picture (L-304)
a7f8dd621 craft-develop: extract local, nrc_joint, auto, and profiles stages (#587) (#3015)
eda95dd6b craft-develop: preserve LightCraft raster sRGB rounding (#587) (#3017)
f01b24765 UnsavedChangesDialog: the app words the Don't Save button (L-304)
11a43e9f1 review: comments look: muted avatars, thread line, text under the name, quiet selected card, compact reply field, empty state, Sort by heading, plain group heading
6314fc55b review: annotate preview outline heavy, flat grey eraser band, round handles, one callout leader
742af23a3 craft-develop: extract raster and crystal grain stages (#587) (#3009)
00957212d Document dialogs write every decimal place their number rows ask for (L-304)
18ab71d7d Unsaved changes question: document wording and order, app title, question, note and icon (L-304)
70364c83a Page range options in a document dialog: heading over one line of radio buttons, fields and subset (L-304)
5021eacb3 ParamLayout::Document: inline captions, shaded option groups, buttons across the foot; the root's DialogLayout is every dialog's default (L-304)
b7ef69b8e Label scheme Inline: captions at the row start, controls right after them, no shared column (L-304)
b67df8175 craft-develop: extract colorops and light_v3 stages (#587) (#3000)
9cea32802 Toasts keep their close button; its glyph takes the inverse card's text colour
53054939b Number rows of a parameter dialog show their unit symbol (L-304)
b86e89054 StandardPrompt and Prompt: an icon beside the message and a muted note (L-304)
eb4fe2262 Modified tabs show the marker dot, not an asterisk (L-302)
26fa96b3a Calendar, popover and date picker take the first weekday (L-301)
34129bac9 png: add write_16bit helper for 16-bit PNGs (#3001)
644e6ff8c Inverse surface role (surface.inverse, text.on_inverse) in every theme; toasts use it (L-300)
3c1773674 Dock area: top inset keeps floating windows below the bars they float over (L-303)
2be8991cb Share SQLite record header splitting (#2846)
7e7707e10 Notice cards are as wide as their text up to the card width
bce467283 Trial: a long unbroken read-only value stays inside the dialog
0a7c0c01c Dim the window behind modal dialogs
988d0c35d ui-shell: format the merged slot pane and trial imports
840e2f67f ui-widgets: a menu row measures its label line as paint finds it, at least the row's least height, so the sample column is as wide as the painted samples
85042f535 ui-contract: a wrapping Row fits a leaf wider than a whole line to the line, never below its least, as contract cv.F37 asks of a wrapping bar
e48b1bab2 menu tests: Defaults in the import review keeps the Will change count, as the production review counts ready entries whatever the start
d5a497daf menu: an import entry left out is listed with the skipped keys, its include box off, so it can be included again
dc09e50ad ui-contract tests: the edge-scroll trial's inner table ends inside its panel's view, so the pointer at the panel's end is past the table's rows
cbe4d9695 ui-contract tests: the wheel-over-popup trial's field takes focus on a press, and each of its wheels starts its own gesture
7e87adadd ui-contract: a window's builder carries the app's hidden state only until its first pass, so a palette hidden later gets one Visible command
b5a630acf ui-contract: a press held on a container's own surface counts as captured, so Escape cancels it and its release is no click
1d08ef8bf ui-contract tests: the detail-body trial's list takes its rows' heights from the source's size hints (kit List::sized)
7395c5f1e ui-patterns: a param row's help button uses the catalog's open icon, as contract wg.W23 names
3acb0d21a ui-patterns: PresetPicker::reborrowed; the named preset row shows a fresh picker in each run of its wrapping row
f1114495f ui-widgets, ui-patterns: NumberField::submits; Enter in Save for Web's Desired File Size field files the search
837c52cb3 ui-contract: where a list takes a payload onto rows too, a between drop leaves the middle of a row to the onto drop
3d8aa12c8 ui-contract: a scrolled body of a layer, or in one, starts at the top at every opening of the layer
5f3373afb ui-contract: a fixture's root window reports its inner rect, so WindowHost::facts and its focus exist in trials
7ca5158c5 ui-contract: gesture_note reports no gesture for a component that was not shown in the pass before
ba7e1355d ui-contract: a fixture's presses and releases carry the modifiers Fixture::hold keeps, so a drop reads a held Alt
011e44f88 ui-contract: a measuring run may read where commands were shown last pass; only real runs register it
89f96bcaa automation: ControlClient::is_usable tells a connection the server closed or sent an unasked line; a call checks it before sending and closes such a connection with nothing sent
3b0c8b780 color-ui: Harmony::apply_hsb gives a rule's colours as HSB, a grey base keeping its hue; Harmony::hues_differ tells rules whose colours all have their own hue
359ebf3e5 ui-widgets: DocumentTab::tooltip gives a document tab a tooltip of its own, shown also when the strip cuts its name
f6f0a87a6 Keep the margin handle visible in its story
78b03dd90 ui-patterns: HostedPdfExport is a HoldsOk dialog; take_ok hands out the answer OK filed, once
d32256ba7 ui-patterns: HostedSaveForWeb::set_original shows a newly rendered document in the Original pane
81c9ae03c color-ui: HarmonyWheel::base draws another colour large than the selected one; the output carries the disc's clicked and context
a6f11a7ec Pair notice bar text with its tone surface
8aa373261 Expose summary tile action in its accessible name
78527d6fd Use a catalog icon for the source link story
82371efbb color-ui: StopColor::rect carries the rect of what asked for the colour, the stop's marker or the Color well
7e9703652 ui-contract: a picture paste with no text is the Paste clip with no text; keys::textless_paste reads it in its pass
baa57be35 theme: token dumps of the vectorcraft-modern theme
0f75980f5 text: the ruler drag trial releases its first drag back on the ruler, so the stop the next drags need is not cleared
714cbf62b menu: the review's words are the ones its view asked for: Start with Current shortcuts and Defaults, Include, Record a key
a92ed3462 ui-panels: the drawer reveal trial finds the revealed row inside the drawer, a popover over the window
a9dd25213 ui-panels: the rail trials compare a compact scope against a comfortable root, and measure each captioned rail under an id of its own
fa3548d4c ui-panels: the dialog trials keep the dialog inside the window's rect, not the root's allocation, and report the window's focus before the armed answer's wait
70bb11ab7 ui-widgets: NumberField::STEP_UP and STEP_DOWN name the step arrows' parts publicly; the step-arrow trial and story use them
ae8bffc19 platform: a boxed ClipboardBackend is a backend, forwarding every method to the one it holds
8713c93c6 ui-contract: CommandState::is_checked reads the checked state the app set
f746bb7df text: CharacterRows::BASELINE and STYLE_TOGGLES gate the baseline shift and the style toggles; PROPERTIES is the Properties panel's Character section
832701e48 pages: PageListEvent::rename, lock and new_row build the struct events outside the crate
de4ebd397 design-lint: contract/unsafe-allow names craft-appkit and craft-wayland
81266c36d ui-contract: Fixture::click_at's press and release carry the modifiers Fixture::hold keeps
4f74e5639 ui-contract: a foreign drag moving to another row restarts the rest wait
04570159a ui-widgets: a Button reads the OS only when it shows a chord
fe8ebb274 ui-widgets: the link-in-a-sentence story fits its room at every text size
ad0ab4b9c ui-widgets: the long chord-button story gives its long text as the label, not as painted text
7d0a26c93 ui-widgets: the List rule-heads story keeps its open groups and applies their open events
0fb47e584 ui-widgets: a List rule head reports no disclosure and no expanded state
e14eb05b5 behavior: a ragged grid's page move passes over rows that cannot take the lead
91c54be4d ui-widgets: a TileGrid rule head stays a section the grid draws and only stops taking the lead
241e37e03 layers: the host-drop trial gives its two panels their own documents, so their row lists get their own ids
294e44299 layers: the holder-depth trial reads each row's tree level; a row's node spans the list's width at every depth
48534a279 ui-patterns: the multi-selection trial closes the row menu its secondary press opened before the next pick
dc534e738 ui-patterns: the removed-draft trial clears the textures of the egui pass it runs nested in
e02ecb67a ui-patterns: palette trials give the capped list room for every result and press the button under the modal by its rect; the press that closes the palette presses nothing, as production's does
f40071883 ui-patterns: the command palette's dialog face opens with an empty query and no active row, as the in-place face does
14d4948c2 ui-widgets: the multi-row ListRow stories show their rows as one set in a Stack
1f7df2dd4 ui-widgets: a click on a field's link label is the click its output reports to a story
2257be8f6 ui-widgets: the held step-arrow story stops short of the repeat, and the step-arrow trial finds the field it shows
f01a9733c color-ui: the fill/stroke spectrum trial warns on a teal outside the print gamut
39fa3de88 color-ui: the disc trials keep the drag value, so the markers sit where their colours put them
58951094f color-ui: a mixed wheel's hue slider keeps its range
db364a1d2 color-ui: a square wheel's inner press part is pointer-only, the square's own stop takes the keys
813bd9c35 Fit tab strips to their assigned viewport
64990fcbf Keep covered context menu openers from receiving clicks
cd5f8103c Open popup after focusing editor in trial
88234f530 Keep field editor drafts while using popups
069ca4503 wayland: unmap_window attaches no buffer to the window's surface, commits and flushes
c66ae09aa theme, ui-contract: RootEnv::accent derives the theme's accent colours and the text on them from the user's seed
0bb4b3cbc automation, text: the Actions and Text Styles row menus take production's rows; Redefine names its style
9e84abce8 text: the styles panel's footer acts on the body's facts of the pass, not on its own empty memory
3bd9464c4 dock: DockArea::continue_move hands the held press to the floating window holding a panel
67b79a74a form_dialogs trials follow the shipped behaviour: the report lists its notes, a narrow room is one short of the pair, typing is the field's gesture, OK confirms the select's pick; stories show Text, not a bare egui label
4da7b481d form_dialogs: a host refusal shows its reason under the rows of a dialog without a held OK
f525fa4e6 ui-contract: a reset of a binding already at its default files no edit
ef7ff0374 Keep a page's go-to offset between bounds that cross by a rounding error
e1a822305 png: share an 8-bit PNG writer
aa3ec09ed Keep trailing tab controls visible in narrow rows
cedf9e01b Size document tabs to available title bar room
3d23eae3d develop-model: map remaining Camera Raw fields to DevelopSettings (#587) (#2942)
253d76d13 color-ui: the picker dialog's Color Swatches view keeps the buttons' cell; the swatch request test names a swatch no channel shares
1ad61320a develop-model: map Camera Raw 1-to-1 scalar fields to DevelopSettings (#587) (#2938)
4cbd19c3a ui-patterns: the radio rows' closure needs no mut
3aba5a108 ColorRequest::on_swatches opens the colour picker on its Color Swatches view
249c70730 Size organize tiles for the page grid
7ed64e69c ui-patterns: HostedListManager, the list manager on the dialog host; take_choice hands over actions that keep it open
a930e2025 font-catalog: Catalog::set_confined_dirs; a link in a confined folder is followed only inside one
e2dca317e ui-patterns: WorkspaceSwitcher::commands replaces the switcher's header menu rows
3de39fd3a media-source: play indexed Y4M from bounded byte ranges (#2903)
d059b9288 ui-widgets: Radio::detail shows a muted line after the radio's label through the label step; form radio rows use it
188b5ad5d Place band tile controls outside the picture
3222847bd Keep constrained number values visible
ecb5efdeb ui-canvas: ToolInput::clicks injects a double or triple click through the window host's click injection
a7a3b72eb Length fields: LengthUnits::bare_in reads bare numbers in another unit; a length row honours Param::codec
3269303dc ui-panels: StandardPrompt readers for the title, message, paragraphs and buttons it shows under a host
2f27abb01 layers: LayersEvent::rename builds a committed rename outside the crate
aa44f62c8 ui-patterns: re-export the palette's RESULTS from patterns::panels
9da354739 automation: ActionList::set_options replaces an action's app options
f8910e877 ui-widgets: SearchIndex::for_host builds the command index without a Ui
f8176040a menu: summary::text_for writes the shortcut summary without a Ui, in the host's language
e13c89f2f host: Host::words and Host::language_code, the language a scope under the host reads, for code without a Ui
5d5d1922c Make tile insert targets discoverable
d0e5b4803 ui-patterns: a custom row's CustomContext::request_action reaches HostedParamDialog::take_actions
547deb28b ui-patterns: Param::on_set runs a custom row's set-time rule; enabled_above, enabled_when_differs and disabled_value
a48f67a91 automation: ControlClient::call_until stops waiting when cancelled; connect and replies honour CONNECT_TIMEOUT and MAX_REPLY_BYTES
024517d26 Clip transform previews in source space
037eeb7ff Keep portrait frames for Organize pages
f6f56c94d Test room for Organize page controls
706806575 Remove unfinished Organize layout trial
7487619fe Handle Organize trial output
6c31c15bb Exercise landscape pages in Organize trial
345aa013f Add trial for Organize tile proportions
31bed444b Show labels beside About link icons
0cfe8611e Preserve form edits across interactive split panes
970d4d876 Add trial for labelled About links
858806e99 Keep document tab titles readable when shrinking
b2fb0af16 Keep rail footer controls compact
6ac171676 Add gallery story for many document tabs
31ff19822 Add regression trial for scrolling document tabs
54fa31df1 Keep tile actions visible over pictures
f131044b8 Show insert targets in tile grids
ee356c24d Restore scrolling extent of dock rail icons
d0af90856 Constrain dock rail icons to available space
086cbb61a ui-contract: part_senses is not suite-only (a collection row reads it for pressing parts)
5a40cb64f Restore rail pending complete layout fix
739a4cdcc Constrain dock rail icons above footer
fc4b1c7bb Restore field edits before closing modeless dialogs
235ae0872 Restore insert gap paint pending full affordance
5d9620053 Restore rail layout pending complete fix
a8aeb9858 Add compact action card tiles
1f671adbe Docking trials: upstream's macOS click holds the primary button through FakeSystem::set_buttons_held
a9ae38864 A memoized view in a shrinking row reports no fit rank: report_shrink's rank argument added to upstream's new caller
e84a6b5de Show insertion affordances in tile grids
d6a77f18a Constrain rail icons to space above footer
6a061a30b audio: decode FLAC frames through TrackDecoder (#2868)
d14207f66 One alias for a host's name check (NameRefuses) in the PDF and print dialogs; the swatch thumb painter's boxed type named; a doubled must_use dropped
5f4c1f282 cargo fmt on the merged crates
a3cbd1070 Trials: Id::unique for upstream's deprecated Id::new; an unused import
4a76442e8 Port trials follow upstream's APIs: the guides rig without a body rect, in-place tidy and English names on the menu model, page-aware document points
5e71eb0e3 ui-canvas: upstream's task bar drag trial expects the port's offset for an unpinned drop
6647bdd48 text: upstream's Find trial reads the contract's find and replace fields
d20a82481 svg: share CSS transform, style, unit, and viewBox syntax helpers (#789) (#2887)
64aee5af2 color-ui and draw stories bind through data.bind, value_at and pair_at, as upstream's do
b6a424608 draw: slice dialogs module is pub(crate) for upstream's registry
9e6623383 Presets: one source over prebuilt rows (PresetSource::listed), NameCheck::of_host for a host's closure, and Naming public; the PDF and print dialogs keep their contract closures and check through them
20b882101 Port types in upstream's memoized models are memo props and plain data: Alternate, ThumbAspect, FacePaint (by key), LabelUnderline, DataColor, the font height glyph
ed0c7f5ff menu: the context menu's level labels take upstream's Node::Heading
eb3c79935 ui-patterns stories: the port's About stories pass upstream's with_mark; rows stories bind through data.bind
1b1f51954 ui-canvas: the pasteboard's page names read upstream's least layout
990352055 ui-shell: options_place is pub(crate) so upstream's registry reaches its stories
8c03deaab pdf_export: check_name keeps the contract's closure, boxed; the name field checks through check_with over it
0f9af5527 SelectMode, GroupHeads, PresetRowMenu and Selection are memo props, for the presets browser's memoized view
4afe53cae Preserve title space in document tab wrapper
495f8bb53 ui-panels: the memoized tool grid reruns every pass while the host passes tips, keeping the contract's ToolTips trait
425743234 Label is plain data and ArmedFor memo props, as upstream's tracked rails and env values need
d86394cd5 Do not carry keyboard highlight into accessible menu activation
b9ac6ab34 Stories bind story values through data.bind, as upstream's do
70d1d2ee1 Story modules of section_list and AxesButton follow upstream's suite registry
43ae81394 ui-patterns: the dock area derives its prune through PruneKey::new and hide::shown; a held form keeps a blocking host refusal shown until the host withdraws it
ab7554e1e Upstream's selection matches take the port's CollectionEvent::Selection struct variant
21bec17fa ui-widgets: SwatchTable forwards upstream's hover_lead; the removed MenuSamplePaint export goes
cfb9ec9c6 dialogs: the chooser's Save answers Answer::Saved with the type picked, and a browser save answers Picked::Saved, so name settling and records_file follow upstream's save path
2b7f93a91 Keep transform preview geometry aligned during edits
068e079c7 eps,ps: share AI editing-data lexer and journal save/restore dictionaries (#790) (#2885)
b2b9a7e10 ui-contract: port fields in upstream's literals and calls (span link, run link colour, fill raster, Id::unique)
43c03001a ui-contract: measure-ahead leaves a detail row's height to the shown row, so it no longer asks for a discard every frame
b828193b3 ui-contract: port types meet upstream's memo and replay bounds (ShownFact, ScrubDrags, ZoomScheme); one public keymap
c3c9440a8 ui-contract: one access-tree store: the host's access_tree reads upstream's settled trees; the port's kept store is gone
eed48a1fa Give document tab strips with controls their available width
fb3c66781 behavior: HitShape::Excluding in upstream's translated and snapped; the rename trial reads Workspace through its accessors
531981143 psd: share plane stream, RLE row-count and ZIP limit framing (#786) (#2884)
35dfd1fe9 Pass selection to grid tile actions
71896a085 Show About icon links in gallery
8bf462750 Support icon links in About dialogs
ba25ccd23 Honor initial GIF and WebP frame budgets (#2843)
bb30de6b9 Keep GIF graphic control with its rendering block (#2848)
8b40f1a4b exr: share exact-size scanline zlib inflation (#2853)
0a5b1389c stream-codecs: decode an LZW stream prefix (#2883)
44b842cad video: expose FFV1 configured alpha before decoding (#2881)
e1d62d7e1 image-codecs: share PNG physical resolution writing (#2879)
3f3b6ef03 develop-model: map PhotoCraft Camera Raw exposure to a caller baseline (#2877)
69b7468cd video: dispatch Motion JPEG through TrackDecoder (#2874)
5d25fa3d2 Share scoped media request cancellation (#2861)
810e55270 Add a full-width notice bar layout
fe7647e4d Preserve note draft focus through close press
57bc4efe0 Drop the dp.C12 markers: the Selection match sites take the new fields
99bf55aac ui-patterns: warnings of the forms set, list manager and PDF trials follow the contract
6ecb549d4 pdf_export: Sections carries the editor, art bounds, bleed link, Custom and Save Summary presses; NameCheck::refuses stays public
c60e7a200 image-codecs: expose raw PNG pHYs pixels_per_meter reader (#785) (#2865)
db0fa5522 Fix late-set calls: wheel event fields, shows_outside flag, outline tokens read in style, one warning_lines, help icon button, section list row lifetime, callout Props import
6b3c17464 type-w6: net change of 1 commits
573c38c23 canvas-w6: net change of 1 commits
0f07217ed canvas-w4: net change of 1 commits
33306b07c menus-w3: net change of 1 commits
893caadf6 forms-w1: net change of 2 commits
6766559d3 dialogs: a hook runs around the native dialog's blocking show
4cac56f5a behavior: HitShape::excluding cuts discs out of a hit shape
e62c14cd6 Selection sites: dp.C12 applied, markers removed
5adf65037 font-manager: tests follow the contract
03fae1e21 color-ui: tests follow the contract
53aff1873 ui-panels, ui-shell: tests follow the contract; serde_json for the shell's command trials
88c8e079e ui-widgets: tests follow the contract
aa4c6adbd ui-contract: tests follow the contract (held buttons, wheel share, surface, held keys)
1f255b73e ui-contract: edge step in the body's settle, fit rank forwarded, tooltips off and cursor zoom read
2a5b738e5 draw: one FieldArrangement, Select value bound, named colours per run
904f131b0 ui-patterns: PresetDrop keeps its payload type erased so a browser borrowing its rows less long takes it
b09a1e4f1 print, ui-patterns: name check is public, section list foot, story data borrow
02aeea721 layers, menu, print, font-manager, color-ui: finish calls against the contract (Button popup props, imports, style derive, palette drag map)
37a598bd5 ui-patterns: closure captures, non-exhaustive match, face paint call
1cfd70c13 ui-shell, ui-patterns: finish the writers' calls against the contract (enabled rule, warning_lines, palette ranking, event shapes, duplicate stubs)
159842ccb ui-panels, ui-widgets: opener visibility, muted shade, key type, unused import
081a6ac23 ui-widgets: finish the writers' calls against the contract (swatch table props, number step, link access nodes, menu, select reveal, face detail)
298079fd9 sampler: fold held modifiers from the events
8de18d4b8 ui-contract: finish the contract signatures (key_event held mods, take_discard per viewport, edge step record, item_id, ItemCx detail, unit names)
ee19084e8 keymap, kbd: use PressKind; restore DragRest; phosphor arrow-up/down as fill paths
53da45bbe plugins-w1: net change of 1 commits
df3182f2e files-w3: net change of 1 commits
e2796a8a3 files-w2: net change of 1 commits
d1fbb63a5 files-w1: net change of 1 commits
2cdb9ee52 export-w25: net change of 1 commits
6bd766654 export-w7: net change of 1 commits
09c7de6a7 export-w6: net change of 1 commits
12b620a24 export-w5: net change of 2 commits
05f254cf6 export-w2: net change of 1 commits
9784e10b3 object-dialogs-w4: net change of 1 commits
b3f92746e object-dialogs-w3: net change of 1 commits
58c6ea737 object-dialogs-w2: net change of 1 commits
ab3994954 document-panels-w6: net change of 1 commits
40e9adf9a document-panels-w5: net change of 1 commits
a795af69d document-panels-w4: net change of 1 commits
8be8078c0 document-panels-w3: net change of 1 commits
25dd950fa document-panels-w2: net change of 2 commits
23346483a document-panels-w1: net change of 2 commits
b05db6f62 object-panels-w2: net change of 1 commits
27f85f1e4 object-panels-w1: net change of 1 commits
848fd48a5 type-w5: net change of 2 commits
5b26e4fd3 type-w4: net change of 2 commits
b68ea5f65 type-w3: net change of 1 commits
be2fb0ff2 type-w2: net change of 2 commits
67c3c579a type-w1: net change of 1 commits
a4230e52d effects-w3: net change of 1 commits
4d4790ee7 appearance-w5: net change of 2 commits
43ed18ae6 appearance-w3: net change of 1 commits
da14b63ff appearance-w2: net change of 2 commits
0fca9b125 appearance-w1: net change of 1 commits
e44a6167e color-w7: net change of 1 commits
1d23ec031 color-w5: net change of 1 commits
5d9d3c28f color-w4: net change of 2 commits
ca7c44d1c color-w3: net change of 1 commits
ada9064e3 color-w2: net change of 1 commits
3f8dbe012 color-w1: net change of 1 commits
d3b6429ee widgets-w8: net change of 1 commits
55ba1aac6 widgets-w7: net change of 1 commits
73aa43489 widgets-w6: net change of 1 commits
aff9ce717 widgets-w5: net change of 1 commits
96cbb8440 widgets-w4: net change of 1 commits
01a8bbd29 widgets-w3: net change of 2 commits
387367889 widgets-w2: net change of 2 commits
11b865d3e chrome-w6: net change of 1 commits
4ed94f62c chrome-w5: net change of 2 commits
76fd7a1e5 chrome-w4: net change of 1 commits
3846cbbaa chrome-w2: net change of 2 commits
78864e28b chrome-w1: net change of 1 commits
a08d619f8 canvas-w34: net change of 1 commits
bda01e8fd canvas-w33: net change of 1 commits
d3750889f canvas-w9: net change of 1 commits
5fecd6dbd canvas-w8: net change of 1 commits
6de10ac29 canvas-w7: net change of 1 commits
8ec2c14e1 canvas-w5: net change of 1 commits
135e81764 canvas-w3: net change of 1 commits
4a367a9a5 canvas-w2: net change of 1 commits
a4e2c92fe canvas-w1: net change of 2 commits
6c0224ec4 shortcuts-w5: net change of 1 commits
8a2636cd6 shortcuts-w4: net change of 1 commits
743ed8e7f shortcuts-w3: net change of 1 commits
a481c318a shortcuts-w2: net change of 1 commits
4e2a2db3b shortcuts-w1: net change of 1 commits
652067832 menus-w4: net change of 1 commits
3a990c45d menus-w2: net change of 1 commits
6491d45d6 menus-w1: net change of 1 commits
e5e3b6a30 shell-w5: net change of 1 commits
af2629217 shell-w4: net change of 1 commits
69a9fa558 shell-w3: net change of 1 commits
30b67761f shell-w2: net change of 2 commits
5886f1ff3 shell-w1: net change of 1 commits
5bbc78021 forms-w1: net change of 1 commits
09ac7119e craft-metafile: decode WMF META_ESCAPE_ENHANCED_METAFILE fallback
5e06e0cac Contract stubs: refresh (i18n, keymap, behavior, font-catalog, platform, window, appkit, wayland, automation, dialogs, ui-contract, ui-widgets, ui-panels, ui-canvas, ui-patterns, color-ui, draw, pages, print, font-manager, text, layers, craft)
531f12d23 Add bounded Sony rtmd timecode prefix parser (#2789)
6ff51120e craft-review: handle Picked::Saved in the signature file pick
1f6029714 Contract stubs: craft-color-ui, craft-dialogs, craft-draw, craft-imaging, craft-layers
cfa2db38b Contract stubs: craft-font-manager, craft-text, craft-pages
08445f8c8 Contract stubs: craft-menu, craft-print, craft-table, craft-automation
0151995e6 Contract stubs: craft-pdf, craft-ui-patterns
f4d054950 Contract stubs: craft-ui-panels, craft-ui-canvas, craft-ui-shell
7cc766ad8 Contract stubs: craft-ui-contract, craft-ui-widgets
8bf05d8e4 Contract stubs: craft-i18n, craft-host, craft-theme, craft-keymap, craft-behavior, craft-platform, craft-window
22aa51b5f Preserve selection when text field regains focus
ad0cb48e8 craft-psd: tolerate a missing pad byte after the last image resource (#1949) (#1951)
b674d91fb stream-codecs: decode a RunLength stream prefix (#2703)
81f692810 Allow shells to center toast notifications
d18d5edb4 Open menus from keyboard and accessibility activation
f3c53d63a Allow zoom fields in explicitly sized slots
9b28b56bc craft-cdr: look up clipped objects' own chunks once (port vectorcraft#8862) (#2727)
0350c2edc sqlite: extract shared page-size, B-tree header, local-payload, and signed-int helpers (#2694)
4eeb6ae13 Support trailing text beside hosted table fields
75d179f8d craft-cdr: move VectorCraft's bounded CDR/CMX reader into craft-libs (#2697)
ec985c5e5 Leave unused Escape to the window from notices
c6c90e991 sqlite: share B-tree local payload sizing (#2696)
13e4f9d6b Process pointer releases before later Escape events
bc0a72165 Route scoped keys through the current modal focus chain
eecaef6de Allow grid group headers to opt into keyboard selection
d7d2ba44e Hand passed drags to arena background parts
e2fae69a7 Space document tabs while preserving gap drops
f2b271c54 Distinguish callout box and leader drags
503a71c00 Honor root comment deletion permissions
23d8f47fd Preserve page zoom anchor through tool host
865baab71 Resolve secondary presses on context-only parts
469a82076 Restrict wheel recipients to the active modal family
6b4c8a403 Measure document tab controls on every layout sweep
deca5dd35 Retry live preferences after custom edits
fcc757df4 Keep thumbnail strips in one column
c60d4a7fb Align notice actions to measured text rows
1442efffe Add document-aligned popup list variant
eb63acb5b Apply popup anchor width outside container padding
54329ad92 Keep the first page when a range end is lower
6ed8ebae7 Avoid repeating searches on unchanged field commits
8eacb8d7a Keep captured parts available through release passes
119a633c1 Format framework follow-up changes
76e79d59b Framework fwb11-c: port follow-ups (see runs/fwb11-c.md)
4a52a22e7 Framework fwb11-a: port follow-ups (see runs/fwb11-a.md)
0bff3fce1 Framework fwb11-b: port follow-ups (see runs/fwb11-b.md)
d7e4db780 Fix batch 10 fixture modifiers and rail icons
a1742ab01 rustfmt batch 10
103c968e2 Framework fwb10-a: port follow-ups (see runs/fwb10-a.md)
a209001e3 Framework fwb10-b: port follow-ups (see runs/fwb10-b.md)
43c6ca52f Format merged follow-up changes
fb244d195 Framework fwb9-c: port follow-ups (see runs/fwb9-c.md)
8746d220a Framework fwb9-a: port follow-ups (see runs/fwb9-a.md)
04b26b2d4 Framework fwb9-b: port follow-ups (see runs/fwb9-b.md)
5d5a30d8c Preserve dialog undo and settled file drops
ce6f9b1ab Format merged framework changes
f192f9e1a Framework fwb8-c: port follow-ups (see runs/fwb8-c.md)
6a2fd8c7a Framework fwb8-a: port follow-ups (see runs/fwb8-a.md)
3369704fe Framework fwb8-b: port follow-ups (see runs/fwb8-b.md)
e460d3a4a Framework fwb7-a: F-116 builder, F-117, F-118 (see runs/fwb7-a.md)
bdd1288eb rustfmt batch 6
1333d3c34 Framework fwb6-b: port follow-ups (see runs/fwb6-b.md)
5dd342b27 Framework fwb6-a: port follow-ups (see runs/fwb6-a.md)
465840965 Framework fwb5-a: port follow-ups (see runs/fwb5-a.md)
d2ad92a89 rustfmt batch 5 b
175b8173e Framework batch 5 b: notice actions wrap onto further rows inside the bar (F-79)
80c1ff2d4 rustfmt framework batch 4
c9fe3946e Framework batch 4 d: port follow-ups (see runs/fwb4-d.md)
895d5a46f Framework batch 4 c: port follow-ups (see runs/fwb4-c.md)
d9c0813c9 Framework batch 4 a: port follow-ups (see runs/fwb4-a.md)
c7884b336 Framework batch 4 b: port follow-ups (see runs/fwb4-b.md)
d55211163 rustfmt framework batch 3
88b867f4d Framework batch 3 d: port follow-ups (see runs/fwb3-d.md)
e015cb0a5 Framework batch 3 c: port follow-ups (see runs/fwb3-c.md)
ebc242fe3 Framework batch 3 a: port follow-ups (see runs/fwb3-a.md)
0a4db7f91 Framework batch 3 b: port follow-ups (see runs/fwb3-b.md)
019f0bc63 Don't Save answers Cmd+D on macOS and Ctrl+D elsewhere (L-243): trial follows
af0492f1e rustfmt framework batch 2
a5e668771 Framework follow-ups F-38, F-41, F-42, F-43 (app red 7, L-238, L-242, L-243): lettered dialog buttons answer Alt+letter off macOS, Don't Save Cmd/Ctrl+D; icon comments hit across their rect; PasswordDialog takes Enter; print preview message sheet is an accessibility node
55ee137c1 Framework follow-ups F-40, F-45, F-50 (right-click), F-54 (app red 7, L-241): IconicMode::Rail keeps icons beside the expanded panel and toggles it; DockPanels footer pinned under the body; a right-click on a document tab opens its menu without showing it; FindingsList collection setters
cc1a3b9ba Framework follow-ups F-37, F-39, F-44, F-46, F-49 (app red 7, L-239, L-245): arrows go to the body's tool host first (CanvasSpace::take_nudge; BodyKeys::nudge removed); Cmd+Page keys turn a page; left/right judge the current page and turn at its edge; a wheel over a plain field scrolls the view (scrolled reported); short pages centred, resize keeps the page top
b75793951 Framework follow-ups F-31, F-52, F-53, F-55, F-56 (app red 7, L-244): a right-click during an open press is an aside (PressStep::aside, PressState::context); Draw::cached_texture; Draw::role; AppCatalog::defs public; request_rename repaints settled regions
61211dabe rustfmt the framework follow-up batch
b3b4b968c Framework follow-ups F-18, F-20, F-22, F-23, F-26, F-27 (app red 5-6, L-224, L-229, L-235): AppearancePage can_favorite/size/contrast; HostedParamDialog set_param_label/set_param_shown; HostedPrint JSON sheet/sheets/notes; JSON sets recorded as edits (HostRequests::set_field); RecoveryStore::write remakes its folder; no autosave deadline while nothing waits
1a2d183fa Framework follow-ups F-24, F-25, F-28, F-33 (app red 6): the title bar's title takes no width of its own, so a fill tab strip gets the free width; relative-name drops keep their bytes (Frames::drop_with_bytes); unfocused page keys go to the window's document canvas; ScriptedFiles::written
21e77e2b5 Framework follow-ups F-29, F-30, F-32, F-34, F-35 (app red 6, L-236, L-237): Shift/Cmd page keys go to the ends; PageViewOutput::scrolled; ToolPass::hide_pointer; the arena gives a secondary press only to CONTEXT_CLICK targets (scroll bars take theirs); BoxRules::with_edge_band
3ea91d985 Framework follow-up F-21: split panes and dock group views are layout rooms of their own (public runtime::env::in_room over Env::new_room), so a fill child takes its pane, not a share of the parent body (app red 4, g5)
c5fcca763 Framework follow-up F-19 (lead, q222): an inert run declares no parts, so its parts read idle (run.rs: parted = !inert)
872b17822 Framework follow-ups F-16, F-17 (L-220, lead): #[container] forwards least_width; SyncedViews::halves takes a half that may show nothing (Option<PageViewOutput>)
f89db95a8 Framework follow-ups F-5..F-9, F-11..F-15 (L-214; F-10 withdrawn, L-216): collection menu keeps its request; one-shot FocusRequest; FontRef::family public and faces for the editor host; document_face chains subsets; CropApply cause; PlatformText::word; testing SystemReader text scale; TextEditorHost::replace reports what it kept; SplitViews::show_panes; HostedParamDialog notes and OK between passes (lead: builds, three trial fixes)
12f97ea2d F-2 trial: the separator is a row of the open list, asserted as a wider step between rows (lead)
7ee6bb979 Framework follow-ups F-1..F-4 (L-205): HostedPrint::take_settings_clicked; choice rows separated_at with a story; PrintPreviewOutput::paper_px and HostedPrint::take_paper_px; ScriptedFiles::failing_writes
6b2742f9c fw-red-5: drop the last DIAG print in product code (its trial passes); text.rs cites L-201
190dda606 red round 5 (last two framework failures, Opus): ListRow reads in a measured row, number blur draft; DIAG prints removed (runs/fw-run-6/red-last.md); committed by the lead
9266dfed4 red round 4 (fw-apps, Opus): fixes for fw-run-5 (runs/fw-run-5/fw-apps.md); committed by the lead
a999ccdf9 red round 4 (fw-pages, Opus): fixes for fw-run-5 (runs/fw-run-5/fw-pages.md); committed by the lead
d86a6015a red round 4 (fw-widgets, Opus): fixes for fw-run-5 (runs/fw-run-5/fw-widgets.md); committed by the lead
d4764b83b red round 4 (fw-lists, Opus): fixes for fw-run-5 (runs/fw-run-5/fw-lists.md); committed by the lead
0d650f406 red round 3 (fw-apps): fixes for fw-run-4 (runs/fw-run-4/fw-apps.md); committed by the lead
adc35ad1d red round 3 (fw-shell): fixes for fw-run-4 (runs/fw-run-4/fw-shell.md); committed by the lead
23a349bdd red round 3 (fw-pages): fixes for fw-run-4 (runs/fw-run-4/fw-pages.md); committed by the lead
aa0661668 red round 3 (fw-lists): fixes for fw-run-4 (runs/fw-run-4/fw-lists.md); committed by the lead
db30ca7f6 red round 3 (fw-widgets): fixes for fw-run-4 (runs/fw-run-4/fw-widgets.md); committed by the lead
818f57168 red round 2 (fw-apps): fixes for fw-run-3 (runs/fw-run-3/fw-apps.md); L-198 applied in fw-lists; committed by the lead
a4fca9b78 red round 2 (fw-shell): fixes for fw-run-3 (runs/fw-run-3/fw-shell.md); L-198 applied in fw-lists; committed by the lead
b218c823b red round 2 (fw-pages): fixes for fw-run-3 (runs/fw-run-3/fw-pages.md); L-198 applied in fw-lists; committed by the lead
9ae70fa66 red round 2 (fw-widgets): fixes for fw-run-3 (runs/fw-run-3/fw-widgets.md); L-198 applied in fw-lists; committed by the lead
f49ad550f red round 2 (fw-core): fixes for fw-run-3 (runs/fw-run-3/fw-core.md); L-198 applied in fw-lists; committed by the lead
3ca5e9ef7 red round 2 (fw-lists): fixes for fw-run-3 (runs/fw-run-3/fw-lists.md); L-198 applied in fw-lists; committed by the lead
5005f557d red round 1 (fw-apps): fixes for fw-run-2 (see runs/fw-run-2/fw-apps.md); committed by the lead
58f449f33 red round 1 (fw-shell): fixes for fw-run-2 (see runs/fw-run-2/fw-shell.md); committed by the lead
0e912e4b6 red round 1 (fw-pages): fixes for fw-run-2 (see runs/fw-run-2/fw-pages.md); committed by the lead
4633efd56 red round 1 (fw-widgets): fixes for fw-run-2 (see runs/fw-run-2/fw-widgets.md); committed by the lead
f7c52bf55 red round 1 (fw-lists): fixes for fw-run-2 (see runs/fw-run-2/fw-lists.md); committed by the lead
f28b55c66 red round 1 (fw-core): fixes for fw-run-2 (see runs/fw-run-2/fw-core.md); committed by the lead
c968deda5 pdf-workflows-w1: a measuring run may peek the rename requests (read-only, as scroll reveals), classified in the guard (lead, fw-run-1)
0971a039e pdf-workflows-w3: story captions are 'static, as Label takes them (lead, compile)
4737a02bb combine pdf-forms-workflows-fw: collect the exclusive toggle's others before emitting (owning source borrows self); auto_scroll_mark_look from style (moved by pdf-canvas-w2)
7c04a29b6 combine pdf-forms-workflows-fw: mod note_popup (pdf-comments-w3, lost to -X theirs), sync_views (pdf-workflows-w3), files (pdf-workflows-w4) (R-1)
619c0a94b pdf-forms-w4: net change of 3 commits
1e60f7ed9 pdf-forms-w3: net change of 3 commits
59d88050b pdf-forms-w2: net change of 2 commits
031b32dc9 pdf-forms-w1: net change of 4 commits
6a83eb7a0 pdf-workflows-w4: net change of 3 commits
8a8115e1c pdf-workflows-w3: net change of 3 commits
0d14e431d pdf-workflows-w2: net change of 3 commits
2454b55d0 pdf-workflows-w1: net change of 4 commits
ef4d38b97 Cargo.lock: one base64
b33d09714 pdf-shell-w2: base64 0.23, the workspace's one version (lead)
1f6640bb2 pdf-comments-w3: the story's thread has no status field once pdf-comments-w4's change lands (lead, compile)
1ddedb439 combine pdf-comments-fw: mod dock (pdf-chrome-w8's trial, missed at its combine) (R-1)
3b8c93cb5 combine pdf-comments-fw: mod rtl_paragraph (pdf-chrome-w3, missed at its combine) and mod submit_key (pdf-comments-w5) land with their files (R-1)
bb03e0d48 pdf-comments-w6: net change of 2 commits
49628dbfc pdf-comments-w5: net change of 3 commits
1221a396d pdf-comments-w4: net change of 2 commits
bac347860 pdf-comments-w3: net change of 3 commits
8faf284b9 pdf-comments-w2: net change of 4 commits
b9c4d4787 pdf-comments-w1: net change of 4 commits
315b8a939 combine pdf-edit-fw: mod format_text; lands with pdf-edit-w1's trial (R-1)
c12868925 pdf-edit-w5: net change of 2 commits
5b7a6d76f pdf-edit-w4: net change of 2 commits
b0f2f3d2c pdf-edit-w3: net change of 2 commits
29dc45a2a pdf-edit-w2: net change of 2 commits
a8ee46315 pdf-edit-w1: net change of 2 commits
1a888783f pdf-dialogs-w3: clone the pager into the row's FnMut closure (lead, compile)
a2cd90b8e pdf-dialogs-w2: clone the caption into the row's FnMut closure (lead, compile)
81e831a7c combine pdf-dialogs-fw: recovery keeps pdf-shell-w5's answer (lost to -X theirs) and calls pdf-dialogs-w2's apply_to_all
11ff86b24 pdf-dialogs-w3: net change of 2 commits
c968881b8 pdf-dialogs-w2: net change of 2 commits
76a76aa88 pdf-dialogs-w1: net change of 3 commits
74265cded pdf-chrome-w2: review fixes (review-2)
71f8e9f48 Revert owed absorb action: pdf-dialogs-w1 lands its own
39e1b2f56 combine pdf-canvas-fw: the auto-scroll mark's look moved to style::paint (pdf-canvas-w2); point cx.rs (pdf-chrome-w3) and draw.rs at it
f49d5dae3 pdf-canvas-w7: net change of 5 commits
e8fe46ae1 pdf-canvas-w6: net change of 2 commits
0c629580c pdf-canvas-w5: net change of 3 commits
b6304e0b2 pdf-canvas-w4: net change of 3 commits
ff5f067ba pdf-canvas-w3: net change of 2 commits
5bd3eb697 pdf-canvas-w2: net change of 3 commits
d0d8281e9 pdf-canvas-w1: net change of 2 commits
2f0139bbc pdf-chrome-w7: Seen's default with Rect::NOTHING, since Rect has no Default (lead, compile)
cf27faadf pdf-chrome-w2: pass the collection to key_reorder; spec_text visible in the crate (lead, compile)
f9e8433cf pdf-chrome-w8: net change of 3 commits
20bec4199 pdf-chrome-w7: net change of 1 commits
4ee975d1a pdf-chrome-w6: net change of 2 commits
c18d2c2f7 pdf-chrome-w5: net change of 4 commits
c61ce0959 pdf-chrome-w4: net change of 1 commits
4cc8033a2 pdf-chrome-w3: net change of 4 commits
b0480a9e0 pdf-chrome-w2: net change of 5 commits
2fe7ab839 pdf-chrome-w1: net change of 1 commits
a1658af0a Revert owed Span::strikethrough: pdf-chrome-w3 lands its own
dec6ee10f pdf-print-ocr-w1: label Collate and Grayscale beside their boxes inside the Copies field (look 1)
c25efe0b9 pdf-shell-w5: net change of 3 commits
a0e57a918 pdf-shell-w4: net change of 2 commits
8a2893d33 pdf-shell-w3: net change of 2 commits
be25458c1 pdf-shell-w2: net change of 2 commits
445e6ecc1 pdf-shell-w1: net change of 4 commits
545014b74 pdf-print-ocr-w1: declare pressed before the section that borrows it (compile)
e996936fe owed by pdf-dialogs-w1: absorb moves the row action (DG-C8, R-2)
32ea853de owed by pdf-chrome-w3: Span::strikethrough (asked by pdf-print-ocr-w2, R-2)
00893772d pdf-print-ocr-w1: net change of 4 commits
1e8f98ba3 pdf-print-ocr-w2: net change of 2 commits
85cedd45f pdf-print-ocr-w3: net change of 1 commits
804bae5a3 Contract stubs: browse callers of QueueOptions.order (SH-C32) and GridZoom (WF-W12), marked P3
4c0faa523 Contract stubs: phase 4 refresh
a026a0974 Icons: list_numbered in icons.toml (ED-F1), for the four pinned pack files
9ceaaa237 Icons: list-numbered files of the four pinned packs (lucide-static 1.63.0, @phosphor-icons/core 2.0.8, @carbon/icons 11.91.0, @tabler/icons 3.50.0), unchanged, for ED-F1
9f6b9e339 Contract stubs: stage 9 mop-up
931482a29 Contract stubs: craft-draw, craft-font-manager, craft-text, craft-pages
33fa6bf02 Contract stubs: craft-print, craft-review, craft-automation, craft-color-ui, craft-dialogs
ae7f1f421 Contract stubs: craft-ui-patterns
ecb210cc2 Contract stubs: craft-ui-canvas, craft-ui-shell
2d343ecbd Contract stubs: craft-ui-widgets, craft-ui-panels
aafc82051 Contract stubs: craft-ui-contract
f0275a082 Contract stubs: craft-behavior, craft-fonts-build, craft-platform, craft-window
830da93f0 Contract stubs: craft-atomic, craft-font-catalog, craft-i18n, craft-host, craft-quit, craft-settings, craft-theme, craft-web-host
```

## photocraft `dbad7de99a..e56522780` (non-merge commits)

```
f7fe0b1c8 raw: tell sensor clipping before lens shading; native-only worker test
e75657ae5 raw: answer the highlight-rebuild review (budget, CFA period, white)
dca6824c9 Object Selection: run GrabCut until it settles (#6975)
aa92b8e97 WIP: Object Selection runs GrabCut until it settles (#6975)
e6b7ac6df Fix-up for #11756: no Align Edges on the Star tool, as main's Polygon change says
0349435d5 Address CodeRabbit review: a canvas drop goes to the document the canvas shows
85324d2fe Fix-up for #11756: clippy, and keep the Pen carry-on test about carrying on
5d5f32665 Fix-up for #11756: name the bounds tuple for clippy
f2ac09d2e Fix-up for #11756: Align Edges as the reference application renders it
eda998f69 Use multilayer fixtures for successful MCP layer exports
a83110cf7 test(ui): pin the Insert-key route and the answered drag's selection (#13216)
c3a1d86ef Layers to Files: alert when the document only has a Background (#13968)
4eb8c7f31 feat(ui): editing a live shape with Direct Selection asks first (#13216)
ebc9c53ff test(ui): the live-shape question must precede a direct-select edit (#13216)
859a345b8 Keep compatible typed extensions available in Save a Copy
af3f11b32 Filter Save a Copy formats by colour mode and depth (#14170)
898f507d6 Artboards to Files background tests: fill the boards' layers directly, since edit.fill moved to the paint crate
462d6c7f9 fix(ui): keep every rasterize answer inside its own document
13f14acf3 fix(ui): keep the Patch rasterize answer inside its own document
4a136a64b Path delete test: start from an installed session
32704afb6 Path Selection Delete removes paths, not the layer; Properties eyedropper picks
b8726d6e2 raw: rebless the Windows RAW Open baseline hashes from hosted CI
400c351ac macOS menus list each system item once; arrange uses the document area
446730161 cargo xtask scorecard
fcda1c5b4 cargo fmt --all
8cd8e5e2e Engine split step 8: hand fixes after the test move (atn name_ref fixture, imports, core test helpers whose last users moved)
db03b87ab Engine split step 8: core tests that run vector commands move to the top crate's tests/it (move_tests.py output)
6d73dcee0 Engine split step 8: the vector crate (install, incomplete-docs list, layer comp recorded step, top wiring, corpus feature, core items vector uses made public, tooling)
eb982b0c8 Engine split step 8: engine_cut.py split vector (vector modules to crates/engine-vector/src)
722cb812e Split scripts: whole use trees in the trait check, re-exported traits through globs, inner attributes nest, mod.rs unit names, inline modules named from other files, assigned .ok() results count (review on #16587)
f752e6d0a fix(ui): keep the rasterize prompt off hidden and stale patch targets
ac0dfaf77 Engine tests before the vector cut: the closed-doc check's coverage floor is a share of core's commands
036f638e0 feat(canvas): Type-tool off-text right-click opens the selection menu
4b67fd4b9 Engine tests before the vector cut: the .atn table's commands are checked against the app's registry order
a54541992 Prepare dormant PhotoCraft release-ref nightly callers (#16459)
bdb914d6e Artboards to Files background tests: PNG-24 from the dialog's list
0d4b939d4 fix(ui): Patch on a Type layer opens the rasterize prompt (#12121)
190bf8212 Window-wide drops: the pane under the pointer, nothing into a hidden document (#16588)
3765708ac feat(ui-egui): the Paragraph Text Size dialog creates from a settings snapshot
88b71cb14 Add the Apple-silicon Nightly build (#16559)
c0504a9c0 Files dropped on the Home screen open again (#16588)
4b958e79e WIP: files dropped on the Home screen open again (#16588)
4653c2a2a Engine split scripts: step 8 crate map, review fixes, move_tests appends and copies only what moved tests use
bdb340c21 Engine tests before the vector cut: source cache statistics, core cache tests in core, pattern fill placement in the paint tests
5bc88ca51 Layers footer: Alt-click with a lone artboard keeps New Group
3ab64de0f Move the batch's paint-dependent core tests to the engine crate
92a5105ef Fix CMYK test initialization and canonicalize saved colors
dc7005d4a Engine split step 7: the main batch's core tests that run edit.fill move to the top crate's tests/it
f0943fb57 revert: restore scheduled release.yml dry-run on main (#16563)
99296ad6a Keep CMYK serialization in the SavedPrefs module
c1c1c51ea ci(release): publish scheduled nightly prereleases (#16551)
cce393bee Engine tests: build the session with the installed commands (crate::session)
cafbc8dba Engine tests: build the session with the installed commands (crate::session)
d096132a1 Artboards to Files background tests: clear the active layer through set_active_layer
508d8b91e Droplet play options test: build the Camera Raw settings field by field (main's CameraRaw is non-exhaustive)
3bc5136b7 nextest: heavy UI GPU tests take two slots on Windows
df8c2bd63 Cargo.lock: the paint crate follows main's craft-libs pin
cbb670962 Engine split step 7: the History Brush format checks' tests move with them to paint's history_source
43fbcaaef build: pin craft-develop for the shared sensor clip mask
8301a166d Engine split step 7: each engine crate lists its own incomplete-docs commands (handed to the install; core's param_check reads the installed set)
85bfb2761 Engine split step 7: scorecard
9277ac167 Engine split step 7: hand fixes for the moved tests (test-local copies, flate2 dev-dependency, unused imports, dead test helpers)
31b5cf9ea Image Processor test: pass the empty file list as `input`
a7fd5975a Engine split step 7: core tests that run paint commands move to the top crate's tests/it (move_tests.py and its output)
163fa9ff9 ci(release): forward ARTCRAFT_SERVICES_DISPATCH_TOKEN secret (#16535)
c0dd6c81a Check the Windows Nightly source identity and widen the profile fences
aa21256f7 Address CodeRabbit review on the restored batch
850e73c5f Engine split step 7: the paint crate (installs per crate, registry order from command_ids.txt, helpers moved down, paint-only helpers moved up, live-stroke traits, admit hook, tooling)
a8d463fab raw: rebuild sensor-clipped highlights instead of clipping after white balance
1086d71fd Engine split step 7: engine_cut.py and its paint split (paint modules to crates/engine-paint/src)
6f20a6b9f gpu, camera_raw: develop Detail on the GPU using craft_develop (#13519, #587) (#16480)
81cc69f17 Address CodeRabbit review; follow main's colour state accessors
b2e243cc5 paint: adapt mixer load swatch to the shared next API
83d04ac4d Drop a link scan taken against another document folder
446dd0e38 Canvas Size: Gray keeps the background colour in an Indexed document
53248aa86 Use the installed session fixture for FOGRA39 tests
657abde67 Canvas Size: read the params doc from the installed registry in the test
7557c44f4 Expect the picked paths to wait outside the frame in the place test
9c6e55f7d Canvas Size: Gray is linear grey 128 in a 32-bit document
02a6bee23 Show the Batch OK refusal in a Batch alert
3c5c41ac5 Keep a multi-file Place Embedded out of the selected frame
6541b5cdb test: run mcp_wait tests alone in the gate shard
f805c79c2 Cut the canvas across by the shown view's width in the place centring test
1ad43aaf2 Rectangle and Triangle menu tests: build the fixtures through the shape commands on a larger document
752c9831a Allow the pixel-aligned view centre in the place centring tests
085343bc2 Rectangle and Triangle menu tests: zoom in so the right-clicks miss the box handles
b15b1f29c Filter preview: the addressed document, layer and mask are part of the preview session
f5175404c Rectangle Align Edges tests: read a rounded rectangle's radii from its live shape
e1e1eb8f0 Actions tests: a Paint Bucket step's explicit colour replays after an .atn round trip
a3774329f Shape options bar: the tools' own extras in their own function (clippy cognitive complexity)
000a06cf0 Rectangle Align Edges tests: give finish_drag the drag's exact box
d034064da Polygon Align Edges tests: give finish_drag the drag's exact box
12ef932cc test: give panic_hunt 90s on the k8s gate
dd1c3ef78 Align Edges tests: PhotocraftApp::new gives a Box
565bd2076 Align Edges tests: PhotocraftApp::new gives a Box
e03e93e2d Fonts: tests offer a fixed catalog; installed fonts only where asked for
4c6008cfd Brush tests: a failed tools.setBrush pick of a preset with a broken tip keeps the current brush
5ce212768 Engine tests before the paint cut: failpoints feature, setups without paint commands
5dfd031e7 Gate: unreached .rs files fail in strict mode; drop unused test items
6770b80ba Mixer Brush load swatch shows the sampled tip image or the loaded colour
3098631a3 Hue/Saturation range swatches and spectrum run the engine's adjustment on one colour
6afdd3c3d Pin craft-libs to ee30b17c21 (Select and Mask previews after a document edit, #3140, #3145)
ebb0a1c46 Restored adjustment and Select and Mask tests drive the app as a user does
6097755e1 Adjustment dialogs: live CPU preview on masks, channels and modes the layer path refuses
9cf019ed3 Delete files no module compiles (widgets, point curve, brush controls, analysis_ui, old cursors, dead editors and tests, nefc tests)
e23da5f24 Tests: Notes panel, Count label size, adjustment dialog and Select and Mask tests mounted again, Command-click Auto-Select inversion
18bb80398 Notes panel empty state and counter, Import Notes outside alert, Count Label Size, Select and Mask stale-document reason, Selective Color drops stale single-colour keys
b21f59905 Tool presets: use Tool::id for the engine tool id
ec81d5e5b Engine split scripts: only crate roots and mod.rs hold children beside them; children of a cfg(test) inline module are test code
6fff6629f build: move craft-develop to craft-libs main b2d2a1e9 (craft-libs#3101) (#16485)
5cc65b5b6 Engine split scripts: imports keep their attributes; module paths from crate roots and inline #[path]
4bd262427 Build archive headers from the opened stage file
5b1214ab8 Keep the huge-canvas selection time bound at 90s
7ea8b7c8a Engine split step 6: trait imports where only CI's builds use them (Linux print, app tests, GPU trials)
62263bc7b Engine split step 6: rewrite only files a crate target compiles
cf80a283f Engine split step 6: cargo fmt
3f9eb0d59 Engine split step 6: compile fixes the script can't make
3cbd7b155 Engine split step 6: slot_access.py and its rewrite (slot fields to accessors, trait imports, doc links)
f314928e1 Engine split step 6: session slots for colour, presets, actions, Object Finder, Select Subject, print; extension traits for the upper crates' Session methods
280de1fbf Pin craft-libs at 7ab3cf8196 (a rescan with no changes keeps the catalog generation)
3ed1b1709 Record archive members under their checked stage name
3f28d1441 Keep an unshown Nightly interruption notice across a failed start
601d57603 session_state.py: report edits inside a moved definition's body
50eba6f01 Check resolved build paths and archive member names before each file operation
c684dad54 test(ui-egui): cover #14493's Use Legacy reset where tests compile
3298b955c docs/scorecard.md: regenerated (cargo xtask scorecard)
3d914bce7 Pin craft-libs next a4e08cdaf2: crop move of a turned document, docked toolbar column and grip cursor (craft-libs#3104, #3105)
859fd07b2 test(engine-core): the batch's new tests start from crate::session() after the engine split
877982cb0 fix(compose): CMYK split stand-ins keep ink Curves off whole levels
66428d818 Move the batch's Color Settings preset tests to crates/engine-core with the engine split
f9438dbda Engine split step 5: session slots, state types in core, action recording split, prefs file document, closing hooks
30bb60592 Engine split step 5: session_state.py and its rewrite (recording fields to s.recording, Clip::from_svg callers, preset store reads)
691d27ae3 Engine split step 5: action history bookkeeping to crates/engine-core/src/action_history.rs
b6241783b fix(engine): accept Action layer selectors in adjustment param checks
4921d3b5f build: move craft-develop to craft-libs main 33be356 (craft-libs#778) (#16470)
c5e0204f4 Confine Nightly packaging helpers to the private build directory
60910bb9b fix(ui-egui): strip undocumented dialog keys only after reshaping
cc716aa8e Pin craft-libs at bb7e01295d (a row pick of the current family adds no edit)
f2cb19e91 fix(engine): install the registry in schema tests and drop dialog chrome keys
6498e7ab5 fix(engine): move param_schema into engine-core after the crate split
e6e3aff72 fix(ui-egui): do not send Bitmap's input resolution as a command parameter
f0477a595 Engine split step 4: pipeline hooks
80d7c000a Engine split step 4: damage helpers to crates/engine-core/src/damage.rs
59de4b9f9 Type options bar test harness allows 64 steps: opening the font list loads row samples over several passes
58c7904cd Font menu tests find a family's row by its TreeItem role
7da733e60 Compare Image Processor output by path on Windows
e1928b6ac Choose Cancel on the shortcut set confirmation prompt
1f14e82b0 Engine test: ignore the Channel Mixer's new cmyk field in the adjustment brush test
cbaf61084 Share Brightness and Contrast reset logic with the editor events
2f8c8859d Preserve upstream shape addition history alongside the Line Tool label
08b365216 Adapt the regression fixture to the current document model
5823239d3 Pin craft-libs at c57fac1215 (font live updates on next 27e5af8151)
9f5d55c86 fix(engine): document pattern.import group in the dispatch schema
c16f9e1b8 fix(engine): document close discard and artboard extensionColor in the schema
dce2f8906 Keep Nightly profiles isolated and startup resilient
a052dd819 test: harden the three k8s gate flakes on GLES and memory pressure
d03b04593 Pick a family installed while the font menu is open on the first click
92c579481 fix(ui): the Frame tool shows the arrow over a frame (#13883)
7ad6212c9 Keep production sources in nightly Sonar analysis
85d49dc55 Cover packaging CLI dispatch and failed-output ownership
fdec30403 Report existing packaging tests to Sonar without excluding helpers
a71b1c5c3 Assert the Stable workspace configured folder spelling
e5f81d397 Return a typed error from nightly process identity setup
ce776f0cb fix(type): adapt the corrupt-size mask test to the engine's doc accessor
02193d83b style(type): drop a trailing blank line in the mask style tests
8dadc005c test(type): start the too-large mask case at the file's usual click point
7078cf579 fix(type): a new Type Mask inherits only a size type.create accepts
6d0109a7b test(type): a mask click on a layer whose shown size leaves the create range makes no mask (red)
92345d222 Magic Eraser cursor test: pin the hotspot by value (CodeRabbit on #16251)
aefd26a4e Keep early launch-profile bootstrap separate from the app event loop
523df5804 Magic Eraser: its own icon cursor
9a9f5caf8 Separate secret-free nightly compilation from required Windows signing
0ceaf2e1d Exercise successful exclusive Nightly plist publication
e8335bce1 Render app-owned Nightly Mac bundle metadata without associations
8db86f0ef Own translated nightly session notice text
753a81f63 Preserve bounded prerelease writer information in newer-file errors
9f8f4fb14 Record abnormal nightly window-session completion privately
0998d5e2a Keep nightly default agent files inside their selected profile
78bcbd0b1 Prepare private nightly isolation and channel source
ccc5998fd Test the Color Sampler limit alert for Alt+click off a point (#12219)
8c34f36ff fix(ui): open closed Info panel on sampler placement and alert on limit (#11745, #11743)
abb8b2315 Show fonts installed while PhotoCraft runs without a restart
44205e94d Pin craft-libs at db702f98 for live font catalog updates
bfe30f7eb test(compose): keep main's Size 1 review test honest about this branch's fix
fc2569c9f test(ui-egui): #12042 pin select.loadSelection as live in the type layer menu
d46ccef77 feat(canvas): Type-tool right-click off text opens the layer menu
13924e259 Reject bare ampersands in Image Processor presets
2f1f612b5 Keep XML and Run controls clear of each other
7df77e0b9 Use current egui ID constructors for XML settings
ddff72c40 Wire settings buttons into the Image Processor menu form
e399a6171 Use the shared bounded XML reader for settings
aba988ddf Take the pending settings action from egui memory
7be0322c1 Save and load Image Processor XML settings
8c8eb049b test(type): stop calling a red mask style test a guard
d890b19ec fix(type): keep a non-finite layer size out of the mask's create params
b84775b5f fix(type): a new Type Mask takes the active text layer's style
2100c0a72 Preserve PSD alignment data and fix dashed rectangle corners
2e2af528a Move Align Edges tests into the engine integration suite
51ea11151 Keep CMYK pick ranks in merged history order
8ce04cd93 Restore context argument for web font installation
2bdb519b7 Fix duplicate WASM file dialog re-exports
1d6730566 Color Settings: run the all-setups test in the global_state binary (#14229)
d46366af4 test(canvas): pin the Pen anchor row under Option and in Pixels mode
98694d224 fix(canvas): Pen right-click heads the path menu with the anchor row
4f3b6ba6c Convert to Profile FOGRA39 test: 2 % per ink, 2.5 % only for Saturation red; a NaN ink fails (#14267)
848417816 Record undo history for layers duplicated into a new document
f31aa3cd6 Color Settings: load a setup set before the first pass, Monitor Color follows the monitor (#14229)
afe1fffe0 Use the tool History label when adding to a shape
f7b46ee40 test(ui-egui): pin the Paragraph Text Size neighbours on the framework route
91f5f371a test(ui-egui): pin the Paragraph Text Size rule at the press of the gesture
ceeac4c6c Canvas Size: Gray extension colour, 128,128,128 (#14543)
e6a775927 Regenerate docs/scorecard.md
cde44927f Fix conditional action recording tests for current actions API
2813506aa Fix CMYK Fill dialog state and CI regressions after main update
4adb424ba fix(automation): treat vectorMask.currentPath as a document path
706f09cc9 fix(engine): document the magicEraser layer target
6f5615f74 fix(engine): complete schema docs for ATN keys, nulls, and clamp ranges
a546a2091 fix(engine): document importCsh path and data as alternative keys
5a37b7577 style(ui-egui): rustfmt import order in chrome and mask property panels
c331b9dae fix(engine): restore print.rs after the main merge and document gradient stop set keys
eb17414cc fix(engine): keep new-adjustment-layer options in the param schema
a61073a2d Properties: a CMYK Channel Mixer layer says why in the panel, not the status bar
3fec89a20 Keep Find and Replace Text available without type layers
78b9ed120 Channel Mixer: keep CMYK smart filters out of the RGB editor and say why
143693d29 fix(ci): remove stale color panel allowlist entry
184d73449 fix(ci): move CMYK tests after document helpers
82c386e56 Fix web compilation after craft framework migration
ba4a5db41 Convert to Profile: sRGB to Coated FOGRA39 gives the reference inks (#14267)
800c3b6dd Place: keep a frame's own centre when the view shows part of the canvas
50d8f4d6c Place Embedded: centre on the selection, else on the part of the canvas in view (#14102, #14103)
2885ee624 Place Embedded several files: queue every chosen file past the drop limits; tempdir fixture
cb90f35d1 Place Embedded several files: drop a wrong mid-transform history assertion
5b4964bc9 Place Embedded: choose several files and place them one after another (#14107)
24f34d150 Rectangle and Triangle tools: the shape and path canvas menu (#13341, #13367)
2bee45129 Polygon tool: Align Edges checkbox (#13326)
9aad9a287 Polygon tool: a Radius field after Sides rounds the next polygon (#13302)
d4edd71ec Rectangle tool: Align Edges checkbox (#13346)
e4ac9e7f5 Channel Mixer editor: one enabled scope for the slider rows; taller editor test harness
47cd663af review: query an absent readout instead of requiring a match (coderabbitai/empty label query panics)
64fa0cd36 Channel Mixer: never regenerate a monochrome CMYK mixr; mixer track colours from tokens
2d5a5060c fix(compose): keep the glow Range 1 % band inside the measured reach
e98a1d24d test(layer-style): pin the Structure title on the Satin page
dbde4a781 fix(layer-style): start a new Satin at the measured factory values
1a525c57d Batch: OK without a source or destination folder says which is missing (#13669)
c8c113457 Color Settings: a control set by script shows Custom before the next draw (#14229)
7f96ae5f1 engine: one typed parameter schema for dispatch, value checks and filter dialogs
b662e550e Match workspace Rust formatting
11371fada Match workspace Rust formatting
db534a44d Keep Image Processor results across source read failures
f9ac6aa49 Give shortcut set deletion confirmation keyboard ownership
1d8dd5a2d Retry CI after Windows runner disconnect
eb77879a0 Color Settings: reject a null preset, and load a scripted pick at once (#14229)
3f6c822b9 Fix shortcut set test for action index API
0c36b0309 Adapt to main: PSD tests import craft_psd (#13414)
755601e63 Fix artboard export order regression test
c31752c55 Color Settings: Settings menu with the 13 named setups, Custom after any change (#14229)
253eef0d2 Use an action row in the shortcut set switch test
310504a7c Use an action row in the shortcut set switch test
3012fc8ca Use an action row in the shortcut set switch test
c12dc3478 Use an action row in the shortcut set switch test
5db08b78e Artboards to Files order test: expect the <prefix><name> file names (#15170)
86a9e9ea2 Artboards to Files order test: expect the <prefix><name> file names (#15170)
f4946b654 Artboards to Files order test: expect the <prefix><name> file names (#15170)
7496bcbc3 Artboards to Files order test: expect the <prefix><name> file names (#15170)
5baeec839 ui: size the Canvas Size readout from the flat image (#14582)
b8524c882 fix(compose): Inner Shadow size 1 draws the measured pixel
5445b1639 Fix shared RAW decoder integration regressions
ee19ca874 Compare Image Processor test outputs as native paths
b03b1fd1b fix(ui-egui): the Paragraph Text Size dialog only creates text in the document it was opened for
09ccd2e9c Alt+click on the new group button groups one active layer (#14717)
4ed9fb9e3 Isolate output fixtures when testing both Image Processor fixes
d2a6a69aa Fix CI regressions in shape presets and import tests
d9a6ee824 Propagate fixture write errors to Image Processor tests
1d25e374c Cover Image Processor input validation and partial failures
0f554d9ac Report when Image Processor cannot open any source files
081bf0283 fix(compose): round the channel curve before the composite curve at 8 bit
528fb9abb Fix aligned dash phase and clarify rendering limits
5ebf7e726 Target the shortcut set Delete button in the regression test
9be48de04 Remove the unused test import
45a0613aa Use the current egui scope ID API in the Properties test
e34ab25d3 Use the current egui scope ID API for the confirmation
682bab943 Check the live preview after toggling Use Legacy
f535bf493 Reset Brightness and Contrast when toggling Use Legacy
4eaf6ef8b Confirm before deleting a custom shortcut set
41dadfd80 Round Rotate Canvas angles to hundredths of a degree
e90b3bec5 feat(type-tool): Alt+drag opens the Paragraph Text Size dialog
0c0e0ef56 fix(cmyk): merge RGB and inks atomically across windows
ae4c28d58 test(type): the hit-zone pin follows the canvas-painted pointer states
d21211e6b fix(cmyk): align eyedropper ring and preserve legacy color merges
142c2906e style(ui): format merged CMYK eyedropper code
8f27ac7fa fix(cmyk): merge independent role inks across preference saves
e028b6ae8 Artboards to Files background tests: allow JPEG's corner bleed in the engine test too
f6aeb681e Artboards to Files background tests: allow JPEG's slight bleed in the corner
669e4cf97 Droplet: say which output folder the subfolder walk leaves out
0e2f6dc4f test(ui): reference-point pointers follow main's reworked transform cursor contract
6f9c52f8c Droplet play options: fix the tests and formatting
1c7575009 Artboards to Files: Include Background in Export checkbox (#13631)
7b23e69e2 Droplet: the subfolder walk leaves out the folder the results go to
a5f7f375f Create Droplet: Play options are saved in the droplet and used by each run (#13709)
f53d493c2 fix(cmyk): preserve explicit paint inks and add mixed-color regressions
d80617367 fix(cmyk): preserve solid fill inks and parent document profile
0e62e311e fix(cmyk): handle null colors while preserving native inks
d9692fb1f fix(cmyk): preserve bucket null-color inks and document picker profile
07028527f fix(cmyk): preserve color history and cache native Info samples
b57f7d504 fix(ci): address platform-specific Clippy warnings
4114b2dbf fix(cmyk): simplify role types and address Clippy warnings
031be20d8 fix(algo): iterate directly over CSR column indices
1a06a38a4 fix(cms): use iterator for CMYK primaries in xyz
3bf4f75e1 fix(cmyk): preserve native ink separations across color tools and preferences
f7a27859e fix(cmyk): preserve document ICC context in style color pickers
7fa621f43 fix(cmyk): respect document ICC profile and preserve native inks
680d7f224 style(cmyk): fix Color Picker formatting
5fb25e4ef fix(cmyk): preserve ICC-managed ink values in color workflows
ee3ae8564 fix(cmyk): preserve native ink values across sampling and color workflows
4cd9bc5bc test(color): cover native CMYK sampling in Color Picker
7fe59f72b Smart filter Channel Mixer test: ignore the new cmyk field
d0567dd53 Format the CMYK Monochrome refusal and its UI test
9c6d8143e Channel Mixer: Monochrome is unsupported for CMYK mixers until Photoshop has a sample
2161208da Fix live line preview test for already aligned geometry
1914076bc Split Windows workspace test to satisfy Clippy
71b7cd9c9 Fix TIFF Clippy complexity and SVG depth regression test
f8c318a70 Fix duplicate TIFF tag constants blocking CI
3485a1cd6 Fix Pen tests to use current Make button labels
429a81cff Fix formatting in roundtrip test case
86875f9e7 Trigger CI rerun
345fe9c01 Use consistent half-pixel rounding across zero
2ea658d58 Change stroke width from 3.0 to 5.0
17cd7347a Document shape editing and Align Edges toolbar helpers
470c0df57 Handle PSD grid move roundoff and clarify Align Edges docs
eadbc5467 Clarify Align Edges output and snapped-render regression test
d65044872 Keep thin strokes visible in large shape previews
bcc0d3727 Fix Align Edges CI regressions
d0fa260c7 Align shape fills and persist layer alignment in PSD
b879ea906 Match Align Edges tool behavior and separate PSD stroke adjustment
482aea166 Fix PSD quantization cache regression test
2d48f095c Reduce shape options cognitive complexity
4e3fa96af Fix Pixel Align Stroke follow-up review cases
6960d1711 Fix remaining pixel alignment review issues
31a697405 Fix pixel-aligned stroke edge cases and add regression tests
87bc285ef Address remaining Pixel Align Stroke review feedback
ca85ffaa3 ci: retry checks after wasm-bindgen download failure
053be2fb4 Fix resumed Pen alignment and document-space stroke previews
ee1cfb4ad ci: rerun checks after download failure
df589b35a Add Tool::Star to shape reset options
51e954e64 Regenerate the scorecard after merging main
f426dc413 fix(ui): show the reference point's pointer in Free Transform
22d0b1f94 Fix Pixel Align Stroke review findings
194d9d4fb Implement opt-in pixel-aligned shape strokes
27dc2cec6 io: move the adjustment arm of content_blocks into adjustment_blocks
89d662187 Channel Mixer: CMYK mixes ink amounts as Photoshop does; keep Photoshop's 44-byte mixr
633baa802 Channel Mixer: save a mixer reshaped to the document mode's outputs in PSD
b931e4440 cargo fmt
4bab77a1f Fix condition for RGB mixer check in adjust_map.rs
5a444b449 Channel Mixer in CMYK documents mixes cyan, magenta, yellow and black (#8047)
```

## Measurements that prove the result

Every number below was produced on this machine on 2026-11-25, with `CARGO_BUILD_JOBS=4` and
`--test-threads 4` (one build at a time, see the skill's machine rules).

| run | command | result |
|---|---|---|
| craft-libs workspace | `cargo nextest run --workspace --no-fail-fast --test-threads 4` | **18,373 passed / 0 failed / 57 skipped**, exit 0, 1,334.8 s |
| photocraft workspace | `cargo nextest run --workspace --no-fail-fast --test-threads 4` | **15,362 passed / 0 failed / 1 flaky / 96 skipped**, exit 0, 1,281.0 s |
| corpus samples | `cargo xtask test-corpus` | exit 0, **670 passed / 0 failed / 11 ignored** |
| design lint | `cargo nextest run -p craft-design-lint` | **28 passed** |
| clippy, craft-libs repairs | `cargo clippy -p craft-captions -p craft-print -p craft-ui-contract -p craft-ui-patterns --all-targets -- -D warnings` | exit 0 |
| clippy, automation | `cargo clippy -p craft-automation --features screen-testing --all-targets -- -D warnings` | exit 0 |
| clippy, photocraft engine | `cargo clippy -p photocraft-engine --all-targets --features corpus -- -D warnings` | exit 0 |
| formatting | `cargo fmt --check` (photocraft) | exit 0 after `bfa2fbc98` |

Crate-by-crate counts measured separately, before the workspace runs: captions 203, automation 104,
print 51, ui-contract 491, ui-widgets 1,181, ui-patterns 431, global_state 28.

## What is **not** proven here

1. **Upstream's own code is not explained line by line.** 629 + 1,279 commits are listed by subject in
   this document's first half; the code they carry is `storytold`'s and is reviewed there. What is
   explained line by line is the fork's side - every file where the fork's content survives and differs
   from upstream.
2. **The Windows CI workflow** (`workspace.yml`) runs on GitHub only; its change is verified by reading.
3. **The wasm build** was checked by `cargo xtask wasm` on the merged tree in the earlier runs of the
   day, not re-run after the two `style:` commits, which touch no compiled code on `wasm32`.
4. **A release's GitHub body** is verified by reading it back and comparing with the notes file; the
   comparison is byte equality after newline normalisation, not a semantic review.

## How to re-derive any statement above

```powershell
# 1. the two merges and their upstream parents
git -C <craft-libs> log -1 --format='%H %P' 65902c049
git -C <repository>   log -1 --format='%H %P' c5af0bae9

# 2. the fork's side of a merge, per file
git -C <repo> diff <merge>^2..<merge> -- <path>

# 3. does the fork still differ at HEAD?
git -C <repo> diff upstream/<line>..HEAD --stat

# 4. the upstream inventory in this document
git -C <craft-libs> log --oneline --no-merges a4e08cdaf..9f2961402
git -C <repository>   log --oneline --no-merges dbad7de99a..e56522780
```

## The order the fork applied these changes

| # | repository | commit | what it is |
|---|---|---|---|
| 1 | craft-libs | `65902c049` | merge `upstream/next` at `9f2961402` (A.1-A.5 of document 01) |
| 2 | craft-libs | `d8e30b0a6` | upstream's workspace-test harness taken whole (C of document 01) |
| 3 | craft-libs | `9c3accef01` | the printer failure path, the English fallback, the memory access (B.1-B.6) |
| 4 | craft-libs | `f40fc6503b` | the ASS escape (B.7) |
| 5 | craft-libs | `afa985fe52` | the clippy repair (B.8) |
| 6 | craft-libs | `87ff4927bf` | the browser-POST fixture (B.9) |
| 7 | craft-libs | `89ed9dfd7` | `v1.0.6` changelog |
| 8 | photocraft | `c5af0bae9` | merge `upstream/main` at `e56522780` (A.1-A.5 of document 02) |
| 9 | photocraft | `220a5435e` | one retry for the driver abort (B.1) |
| 10 | photocraft | `3e5805d5a` | the smart-filter trial moved to `tests/global_state` (B.2) |
| 11 | photocraft | `43983c223` | `v1.3.8` changelog |
| 12 | photocraft | `68cdf8e67`, `bfa2fbc98` | rustfmt after the merge (B.3) |

The releases cut from this state: craft-libs **v1.0.6** (tag `2ca9750b5`) and PhotoCraft **v1.3.8**
(tag `838b33f3c`). The two `style:` commits above land after the photocraft tag; they change no
compiled behaviour, which is why the tagged tree and `HEAD` differ only in whitespace.
