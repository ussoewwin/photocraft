# Per-PR regression execution appendix

Each automated case uses the synthetic fixture/setup and assertions in the named checked-in regression,
linked at the reviewed head. The action is the command below. Expected: at least one matching test
executes, every assertion passes, and the behavior in the case label is retained. A zero-match run is
BLOCKED, never PASSED. After a moved-module port, locate the same test in the frozen integration
tree and retain its fixture and assertions. Manual combined cases are in the main test plan.

All cases initially NOT RUN. Record Actual, Evidence and Tester in the empty fields; the status vocabulary
is PASSED / FAILED / BLOCKED / NOT RUN / SKIPPED. Missing fixtures, unavailable adapters and unsupported
platforms are BLOCKED or SKIPPED with a reason, rather than silently passing.

## PR-3455: Undo and redo action definition edits from the Actions panel

- Fixture/setup: [`recording_and_appending_steps_can_be_undone_without_restarting_recording`](https://github.com/storytold/photocraft/blob/d84fa7955d260505378a1a596ec80d602036319d/crates/engine/src/actions_cmds.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-engine --locked recording_and_appending_steps_can_be_undone_without_restarting_recording -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-3748: control: suggest near matches for unknown command ids and methods (#3674)

- Fixture/setup: [`unknown_names_suggest_the_right_method_or_command`](https://github.com/storytold/photocraft/blob/c45d01eafe84e600bb28033a6853cc19c827b7bf/crates/automation/src/rpc.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-automation --locked unknown_names_suggest_the_right_method_or_command -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-4993: Prevent control-channel engine commands from editing behind modal dialogs

- Fixture/setup: [`engine_execute_cannot_change_document_behind_adjustment_dialog`](https://github.com/storytold/photocraft/blob/81964dc5ef77a71a2ae2a938958cf3ed77d90719/crates/ui-egui/src/control.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-ui-egui --locked engine_execute_cannot_change_document_behind_adjustment_dialog -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-5090: Actions: add editable PhotoCraft Essentials starter presets

- Fixture/setup: [`generated_defaults_are_persisted_without_unsaved_user_work`](https://github.com/storytold/photocraft/blob/61d9e8cc1aed29ed6bea0d0fd2702fa87c4f0206/apps/photocraft-web/src/preset_bridge.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-web --locked generated_defaults_are_persisted_without_unsaved_user_work -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-5176: Play CopyCat, AJ and Line and Color action sets; keep patterns across restarts; Shift Edge scale

- Fixture/setup: [`save_in_edit_contents_updates_the_smart_object`](https://github.com/storytold/photocraft/blob/c96891c298151c5cb11a55928b9e1c27911f38bd/crates/engine/src/actions_cmds.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-engine --locked save_in_edit_contents_updates_the_smart_object -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-6492: Fail control startup promptly when its port is occupied

- Fixture/setup: [`a_prebound_listener_authenticates_and_dispatches_requests`](https://github.com/storytold/photocraft/blob/657cfd503d4ce276df6b5e9f763cb3a5cf2deaeb/apps/photocraft/src/control_server.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft --locked a_prebound_listener_authenticates_and_dispatches_requests -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-7742: Actions: a fractional setting an integer field cannot hold is refused, not rounded

- Fixture/setup: [`a_fractional_setting_an_integer_field_cannot_hold_is_refused_not_rounded`](https://github.com/storytold/photocraft/blob/ab844f9cbcd1f1839ed7f286776331cb72e399aa/crates/engine/src/atn_cmds/tests.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-engine --locked a_fractional_setting_an_integer_field_cannot_hold_is_refused_not_rounded -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-8905: fix(plugins): bound a run's total plug-in memory, not just each instance

- Fixture/setup: [`parallel_bands_fit_the_total_memory_budget`](https://github.com/storytold/photocraft/blob/0fdc6259193aab32b8feab7347ddc36196f40a19/crates/plugins/tests/abi.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-plugins --locked parallel_bands_fit_the_total_memory_budget -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-3874: fix(layer-style): support Blend If Alt-split handle feathering and throttle live preview

- Fixture/setup: [`blend_if_alt_split_and_drag_behavior`](https://github.com/storytold/photocraft/blob/92488307722c3b840507827cac7ee9a88470fc8a/crates/ui-egui/src/layer_style.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-ui-egui --locked blend_if_alt_split_and_drag_behavior -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-3967: Add move and delete actions to Smart Filter rows

- Fixture/setup: [`row_menu_moves_and_deletes_the_clicked_filter_with_one_undo_step`](https://github.com/storytold/photocraft/blob/3a77519ce2cef0fb60cebd4ea796d599f68868cb/crates/ui-egui/src/smart_ui/context_tests.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-ui-egui --locked row_menu_moves_and_deletes_the_clicked_filter_with_one_undo_step -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-4023: i18n(es): add 548 missing rows, fix five filter and video terms

- Fixture/setup: Select Spanish UI; open a synthetic document and the filter/video menus affected by this PR.
- Action: Inspect Interpret/Replace Footage, Tilt-Shift, Mezzotint and Smart Sharpen; exercise labels containing parameter placeholders.
- Expected: All affected labels use the intended reviewed Spanish terminology; placeholders retain their values and no missing-key fallback appears.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-4305: Filter > Other > Frequency Separation (#1714)

- Fixture/setup: [`low_and_high_composite_back_to_the_source_at_every_depth`](https://github.com/storytold/photocraft/blob/1b74873991dbb2e9803d228a969291f09c6f1032/crates/engine/src/frequency_separation_cmds.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-engine --locked low_and_high_composite_back_to_the_source_at_every_depth -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-4711: Treat Dissolve as a per-pixel choice

- Fixture/setup: [`clipped_dissolve_is_binary_and_preserves_the_base_shape`](https://github.com/storytold/photocraft/blob/db546ff4822a011a7915b66af3834f94c05767db/crates/compose/src/tests.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-compose --locked clipped_dissolve_is_binary_and_preserves_the_base_shape -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-4992: Preserve Dissolve coverage when compositing clipped layers

- Fixture/setup: [`clipped_dissolve_is_binary_and_preserves_the_base_shape`](https://github.com/storytold/photocraft/blob/194a8976cd6c14441222b287b9b4f4841a0eb2f9/crates/compose/src/tests.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-compose --locked clipped_dissolve_is_binary_and_preserves_the_base_shape -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-7153: Smart Filters: double-clicking a Liquify filter reopens it in the Liquify workspace

- Fixture/setup: [`smart_filter_edit_after_the_stack_changed_updates_nothing`](https://github.com/storytold/photocraft/blob/6f5d929d6172cf211a070f12cde9507aca9eb30a/crates/ui-egui/src/dialogs/liquify_ui/tests.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-ui-egui --locked smart_filter_edit_after_the_stack_changed_updates_nothing -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-7903: Filter Gallery › Cutout 4/4: trace the image into flat paper shapes

- Fixture/setup: [`cutout_makes_flat_shapes_with_clean_edges`](https://github.com/storytold/photocraft/blob/5c30699087b02edcaa23397a0f7076bf503be048/crates/algo/src/artistic_fx.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-algo --locked cutout_makes_flat_shapes_with_clean_edges -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-8039: Compose: Extend Softer outer glows past Size (#7899)

- Fixture/setup: [`softer_outer_glow_keeps_a_fading_tail_past_its_size`](https://github.com/storytold/photocraft/blob/5579cf5439856fed93f41a669a3fc9b56d5813cf/crates/compose/src/effects.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-compose --locked softer_outer_glow_keeps_a_fading_tail_past_its_size -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-8191: Filter Gallery › Cutout 1/4: colour and palette stage

- Fixture/setup: [`oklab_round_trips_and_marks_the_gamut`](https://github.com/storytold/photocraft/blob/91e4cec43525ca739c05cff614701fbd3576e389/crates/algo/src/cutout/colour.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-algo --locked oklab_round_trips_and_marks_the_gamut -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-8192: Filter Gallery › Cutout 2/4: regions stage

- Fixture/setup: [`oklab_round_trips_and_marks_the_gamut`](https://github.com/storytold/photocraft/blob/412e3b7cf1f4836a5772d93391d21728dbe1f74f/crates/algo/src/cutout/colour.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-algo --locked oklab_round_trips_and_marks_the_gamut -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-8193: Filter Gallery › Cutout 3/4: edges and curves stage

- Fixture/setup: [`oklab_round_trips_and_marks_the_gamut`](https://github.com/storytold/photocraft/blob/864024609b872de3d869879f1632b951c896c6cd/crates/algo/src/cutout/colour.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-algo --locked oklab_round_trips_and_marks_the_gamut -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-8903: Box-filter minified patterns in the CPU and GPU samplers (#8831)

- Fixture/setup: [`minified_pattern_is_filtered_not_aliased`](https://github.com/storytold/photocraft/blob/d387f52f1f2f5681b2c7d1c517aa5b5992c7bc88/crates/compose/src/pattern.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-compose --locked minified_pattern_is_filtered_not_aliased -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-9472: fix(io): import a Displace smart filter that names its map file without embedding it (#9162)

- Fixture/setup: [`displace_psd_filter_with_only_a_map_path_renders_after_replace_contents`](https://github.com/storytold/photocraft/blob/4bab657cd4fb4a91ced44d3721e63a002ca5068c/crates/engine/tests/it/smart_psd.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-engine --locked displace_psd_filter_with_only_a_map_path_renders_after_replace_contents -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-9509: fix: an edit at full coverage keeps its result over huge 32-bit values

- Fixture/setup: [`full_coverage_adjustment_keeps_its_result_over_huge_hdr_values`](https://github.com/storytold/photocraft/blob/d78d7493d3f0b6e760c88de969ac810e9bc5d910/crates/compose/src/tests.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-compose --locked full_coverage_adjustment_keeps_its_result_over_huge_hdr_values -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-9512: fix(compose, gpu): adjustment layers apply Fill inside the special eight blend modes

- Fixture/setup: [`adjustment_fill_changes_the_special_blend_modes`](https://github.com/storytold/photocraft/blob/6c55561128caf5ac917eb3d174e8e7bb4bd48e23/crates/compose/src/tests.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-compose --locked adjustment_fill_changes_the_special_blend_modes -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-9584: fix(compose, gpu): Blend If hard points match Photoshop at 16 bits and for Gray

- Fixture/setup: [`blend_if_16_bit_cutoffs_follow_photoshops_steps`](https://github.com/storytold/photocraft/blob/fc0f7b1132f799b5bba7be642c9047499d95a887/crates/compose/src/tests.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-compose --locked blend_if_16_bit_cutoffs_follow_photoshops_steps -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-9589: fix(compose, gpu): an adjustment layer's This Layer range judges its result before Opacity

- Fixture/setup: [`adjustment_blend_if_judges_the_result_before_opacity`](https://github.com/storytold/photocraft/blob/da157945077190e550b53a5e14b0536f62761292/crates/compose/src/tests.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-compose --locked adjustment_blend_if_judges_the_result_before_opacity -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-9595: fix(compose, gpu): a Pass Through group with Blend If renders isolated

- Fixture/setup: [`pass_through_group_with_blend_if_is_isolated`](https://github.com/storytold/photocraft/blob/75fa71a4f55a098061b12d91508b514565ab8dcf/crates/compose/src/tests.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-compose --locked pass_through_group_with_blend_if_is_isolated -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-9651: perf(algo): row-parallel local_sharpen with fast-path protect_detail (#5426)

- Fixture/setup: [`matches_scalar_sharpen_bit_for_bit_across_layouts_edges_and_hdr`](https://github.com/storytold/photocraft/blob/67666bdf72a58066df868868cef82e5b9cc838de/crates/algo/tests/it/retouch_sharpen_exact.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-algo --locked matches_scalar_sharpen_bit_for_bit_across_layouts_edges_and_hdr -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-9703: Fix adjustment layer name clicks targeting masks

- Fixture/setup: [`adjustment_names_target_contents_instead_of_masks`](https://github.com/storytold/photocraft/blob/3605d30c38294c4d970e52f4a02f44f97e8bc1a9/crates/ui-egui/src/mask_thumbs_ui/tests.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-ui-egui --locked adjustment_names_target_contents_instead_of_masks -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-1991: feat(actions): import and export reusable Actions JSON files

- Fixture/setup: [`json_actions_round_trip_preserves_metadata`](https://github.com/storytold/photocraft/blob/202266b90613280e16ae285b53dd4e10d5f0c7ac/crates/engine/src/actions_cmds/json_file.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-engine --locked json_actions_round_trip_preserves_metadata -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-3142: Read applied Sony RAW metadata and crop ILCE-6000 images

- Fixture/setup: [`encrypted_applied_white_balance_and_black_reach_the_developer`](https://github.com/storytold/photocraft/blob/e2ed234e654e574109ffcdb9756f8e8d1b8a00b2/crates/raw/src/sony_metadata.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-raw --locked encrypted_applied_white_balance_and_black_reach_the_developer -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-3387: feat(codecs): add lossless JPEG XL export for 8/16-bit images

- Fixture/setup: [`alpha_header_correction_rejects_unexpected_or_truncated_headers`](https://github.com/storytold/photocraft/blob/58b83f42a4b39cf56751be53ec8d227efee70f68/crates/codecs/src/codecs/jxl.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-codecs --locked alpha_header_correction_rejects_unexpected_or_truncated_headers -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-3464: fix(vector): target selected saved paths and own Pen gestures

- Fixture/setup: [`appended_work_and_saved_components_preserve_existing_geometry_and_undo`](https://github.com/storytold/photocraft/blob/1eecd00b2dff4ae4c642269cec211cd06ce1c3aa/crates/engine/tests/pen_path_append.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-engine --locked appended_work_and_saved_components_preserve_existing_geometry_and_undo -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-3519: feat(export): optionally trim transparent margins from layer files

- Fixture/setup: [`layers_to_files_can_trim_transparent_margins_without_changing_the_document`](https://github.com/storytold/photocraft/blob/807de6c043a512cb947d86b944c758b3db1277c7/crates/engine/src/file_cmds/tests.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-engine --locked layers_to_files_can_trim_transparent_margins_without_changing_the_document -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-5870: fix(format): reject duplicate native layer identities

- Fixture/setup: [`duplicate_layer_ids_are_rejected_across_the_whole_tree`](https://github.com/storytold/photocraft/blob/f9de0a6abc78aa405b1ef46f5da6cba424106204/crates/format/tests/layer_identities.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-format --locked duplicate_layer_ids_are_rejected_across_the_whole_tree -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-6383: Windows releases ship a symbols zip: the release build's PDBs for profilers (#688)

- Fixture/setup: Use an existing Windows release build and its symbols artifact from the same immutable build; record both artifact identities.
- Action: Inspect the symbols ZIP and load a representative PDB with its matching executable in a debugger/profiler. Do not publish a release to run this case.
- Expected: The ZIP contains the release build PDBs and symbol identity matches the executable; missing artifacts make this case BLOCKED.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-8892: ui: larger Align icons and overflow button; Save for Web keeps its width on an invalid Matte

- Fixture/setup: Open a layered synthetic document in Move tool and Save for Web; capture initial dialog geometry.
- Action: Inspect alignment and overflow controls; enter an invalid Matte value and compare dialog width before/after.
- Expected: Alignment and overflow controls keep the reviewed larger dimensions, and invalid Matte input does not change Save for Web width.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-8897: fix(io): read linked smart objects for PSD export as regular files only

- Fixture/setup: [`reads_stop_at_the_total_limit`](https://github.com/storytold/photocraft/blob/eb06798f15516eb7e4fac24c98346fd9052e7f8e/crates/format/src/read.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-format --locked reads_stop_at_the_total_limit -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-8986: Path tools: edit the saved path selected in the Paths panel (#8478)

- Fixture/setup: [`direct_selection_drags_an_anchor_of_the_selected_saved_path`](https://github.com/storytold/photocraft/blob/390dfbbcb026b33d46c52c62360b65deca00f258/crates/ui-egui/src/saved_path_edit_tests.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-ui-egui --locked direct_selection_drags_an_anchor_of_the_selected_saved_path -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-9339: fix(export): embed requested sRGB profile for untagged sources (#9325)

- Fixture/setup: [`untagged_srgb_export_tags_without_rewriting_pixels`](https://github.com/storytold/photocraft/blob/393645ab91460af2e37cc3ba8f6921e7551f676e/crates/ui-egui/src/export_dialog.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-ui-egui --locked untagged_srgb_export_tags_without_rewriting_pixels -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-9342: fix(io, engine): Embed Color Profile writes sRGB for an untagged document (#9325)

- Fixture/setup: [`embed_color_profile_tags_an_untagged_document_srgb`](https://github.com/storytold/photocraft/blob/c46a74621d36adae9707ab8f3eb39cf6b4c302b9/crates/engine/src/web_cmds/tests.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-engine --locked embed_color_profile_tags_an_untagged_document_srgb -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-9401: fix(format): a save on macOS keeps the file's creation time (#9388)

- Fixture/setup: [`write_keeps_the_replaced_files_creation_time`](https://github.com/storytold/photocraft/blob/f241bba1adfb9c7ec5dd5ed9d86f95b63c7bc2c3/crates/automation/src/workspace.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-automation --locked write_keeps_the_replaced_files_creation_time -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-9416: Affinity reader: contiguous curve records and a cheaper object tag index

- Fixture/setup: [`lookup_keeps_first_match_for_every_object_size`](https://github.com/storytold/photocraft/blob/742b06805284d8d76d5a29ec1821193f4835fedf/crates/affinity/src/tests.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-affinity --locked lookup_keeps_first_match_for_every_object_size -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-9590: fix(io): PSD layer-effect contours keep at most 16 points

- Fixture/setup: [`psd_contours_keep_at_most_16_points_and_their_peaks`](https://github.com/storytold/photocraft/blob/a9744ed44dc782613b19a97b8c159a091f73e669/crates/io/src/effects_map.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-io --locked psd_contours_keep_at_most_16_points_and_their_peaks -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-4763: refactor(liquify): reuse native brush resizing and anchored HUD feedback

- Fixture/setup: Open Liquify on a synthetic grid; place the brush center at a known point and note the initial size.
- Action: Use the native brush-resize gesture, vary size and restore it; inspect the HUD anchor while dragging.
- Expected: Liquify reuses the native resize behavior and the HUD stays anchored to the gesture; no image edit is committed by resizing.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-4985: feat(ui): add Shift+right-click brush mode picker

- Fixture/setup: [`shift_right_mode_picker_selects_journals_and_clear_erases`](https://github.com/storytold/photocraft/blob/1dc98f80f60e993f3edf20af815391b6a7e7add7/crates/ui-egui/src/canvas/paint_mouse/tests.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-ui-egui --locked shift_right_mode_picker_selects_journals_and_clear_erases -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-5082: fix(mixer-brush): Alt-click samples the foreground colour like the Brush (#4873)

- Fixture/setup: [`mixer_load_fills_the_reservoir_from_the_canvas_and_keeps_the_foreground`](https://github.com/storytold/photocraft/blob/23bab3cc9204568aaeb012b9e1da70a2caacb879/crates/engine/src/brush_cmds/tests.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-engine --locked mixer_load_fills_the_reservoir_from_the_canvas_and_keeps_the_foreground -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-5165: Fix: brush section selection

- Fixture/setup: [`selecting_a_controls_section_keeps_its_enabled_flag_and_brush_history`](https://github.com/storytold/photocraft/blob/27bc6e4b37b701480efa69559ab13eafc52bd94c/crates/ui-egui/src/brush_panel_tests.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-ui-egui --locked selecting_a_controls_section_keeps_its_enabled_flag_and_brush_history -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-5453: Shapes: independent fill and stroke paints (solid, gradient, pattern), part 1

- Fixture/setup: [`split_cache_does_not_pin_pattern_tiles_over_its_budget`](https://github.com/storytold/photocraft/blob/e22a9baedceefd97daffe84403b23c99c56d6919/crates/compose/src/shape_split.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-compose --locked split_cache_does_not_pin_pattern_tiles_over_its_budget -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-5939: fix(paint): scale blur sigma with brush size (#5892)

- Fixture/setup: [`blur_scales_with_brush_size_and_diffuses_pixels_visibly`](https://github.com/storytold/photocraft/blob/7348e060b4be7204fb69b59079c3b9d808e9aad3/crates/engine/src/retouch_cmds/tests.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-engine --locked blur_scales_with_brush_size_and_diffuses_pixels_visibly -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-6139: Fix painting workflows, File Info and layer selection

- Fixture/setup: [`behind_brush_and_pencil_preserve_foreground_and_round_trip_history`](https://github.com/storytold/photocraft/blob/e2347f8ead85d929193044f139788b850e31c06f/crates/engine/src/brush_cmds/tests.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-engine --locked behind_brush_and_pencil_preserve_foreground_and_round_trip_history -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-7848: fix(retouch): preserve Smudge pickup with live-pixel stroke commits

- Fixture/setup: [`pressure_shape_and_smoothing_match_replay`](https://github.com/storytold/photocraft/blob/389a1d5bd67072d5581c27989080222a7065b585/crates/engine/tests/live_dab_matches_replay.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-engine --locked pressure_shape_and_smoothing_match_replay -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-9623: Pencil: keep a 1 px freehand line connected where it turns

- Fixture/setup: [`pencil_1px_freehand_line_never_breaks_where_it_turns`](https://github.com/storytold/photocraft/blob/652075f4da7fe905ad651eebc3a6c276fe1d7a5d/crates/paint/src/brush_tests.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-paint --locked pencil_1px_freehand_line_never_breaks_where_it_turns -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-3554: fix(packaging): launch bundled CLI directly from AppImage

- Fixture/setup: Run the checked-in AppRun CLI test; it builds stand-in binaries under a path with spaces.
- Action: bash packaging/linux/apprun-cli-test.sh
- Expected: Argument boundaries, MCP stdio round trip, exit status 37 and unchanged GUI entry behavior all pass the script assertions.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-4742: fix(macos): bundle photocraft-cli and resolve it from droplet scripts

- Fixture/setup: [`scripts_parse_json_and_lines`](https://github.com/storytold/photocraft/blob/b7145b5a8afb85d0eef3922931d6749436966943/crates/engine/src/automate_cmds/tests.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-engine --locked scripts_parse_json_and_lines -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-7743: packaging: the portable zip holds the files, not a folder with the files in it

- Fixture/setup: Obtain an existing portable Windows ZIP built from the frozen integration; record its artifact identity.
- Action: List root archive entries and extract into a new empty directory; launch the app from the extracted root.
- Expected: Application files are directly at ZIP root without a redundant wrapping folder; extracted layout launches correctly.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-8985: Color panel and Color Picker: clip the round cursor to the colour field

- Fixture/setup: [`field_marker_stays_inside_the_field`](https://github.com/storytold/photocraft/blob/8e695caf885b529f01f482b3f0c90cdb34df1915/crates/ui-egui/src/color_picker_ui.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-ui-egui --locked field_marker_stays_inside_the_field -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-9019: fix(xtask): reject unknown performance benchmark selections

- Fixture/setup: [`unknown_bench_is_rejected_before_updating_the_baseline`](https://github.com/storytold/photocraft/blob/364c4be1c19728aefa601ee924bc7a0cd6a09789/xtask/src/perf.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p xtask --locked unknown_bench_is_rejected_before_updating_the_baseline -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-9110: test: GPU tests honour WGPU_BACKEND, so they run on Windows with an Intel GPU (#4)

- Fixture/setup: [`injected_fault_skips_the_gpu`](https://github.com/storytold/photocraft/blob/2e716f93b2e2135944e07d821be4eb38fce13ec9/crates/gpu/tests/it/device_loss.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-gpu --locked injected_fault_skips_the_gpu -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-9436: Startup: a failed launch says why in a dialog instead of closing silently (#7643)

- Fixture/setup: [`path_names_the_attached_log_file`](https://github.com/storytold/photocraft/blob/819f2692db4302019638c57b6ea34cd1e8f4c24b/apps/photocraft/src/logging.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft --locked path_names_the_attached_log_file -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-2300: Scroll the viewport while outlining with Polygonal Lasso

- Fixture/setup: [`polygon_edge_scroll_continues_with_button_up_and_keeps_vertices_fixed`](https://github.com/storytold/photocraft/blob/ebf3777aec961d1815aa141d9987eaa645c2273f/crates/ui-egui/src/polygon_lasso_tests.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-ui-egui --locked polygon_edge_scroll_continues_with_button_up_and_keeps_vertices_fixed -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-5010: Layer Style: Photoshop's layout (three columns, OK/Cancel/New Style/Preview on the right, one-line rows) [depends on #4938]

- Fixture/setup: [`layer_style_keeps_one_size_on_every_page`](https://github.com/storytold/photocraft/blob/cc865221894e21f846d92b721cce7421ea338363/crates/ui-egui/src/dialogs.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-ui-egui --locked layer_style_keeps_one_size_on_every_page -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-5493: Import Photoshop custom shape sets into the Shapes panel

- Fixture/setup: [`csh_import_requires_inline_data_in_automation`](https://github.com/storytold/photocraft/blob/f97eb2849a68e1ed1c08c5b378770cf35cbcb72d/crates/automation/src/workspace/tests.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-automation --locked csh_import_requires_inline_data_in_automation -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-5515: Selecting subject now automatically applies to selected layer, and us…

- Fixture/setup: [`select_subject_samples_the_selected_layer_else_the_composite`](https://github.com/storytold/photocraft/blob/375d4894a119b6cd5933d53e8fe7f506d2783392/crates/engine/src/smartselect_cmds.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-engine --locked select_subject_samples_the_selected_layer_else_the_composite -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-5650: Guides: ⌥ turns a dragged guide; dropping one on either ruler deletes it (#5034)

- Fixture/setup: [`move_guide_turns_a_guide`](https://github.com/storytold/photocraft/blob/4b1bd6fb6d264f885cb9caa3b8a6578ee527bd7b/crates/engine/src/edit_cmds.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-engine --locked move_guide_turns_a_guide -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-6026: Marquee: Fixed Size selects exactly the entered width and height (fixes #6015)

- Fixture/setup: [`fixed_size_px_is_exactly_the_size`](https://github.com/storytold/photocraft/blob/248ba660576f4896f69cbb6639353e2f1d295271/crates/ui-egui/src/canvas.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-ui-egui --locked fixed_size_px_is_exactly_the_size -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-6236: Shift+Alt+right-click opens Photoshop's HUD Color Picker (#6098)

- Fixture/setup: [`shift_alt_right_press_opens_the_hud_not_the_picker`](https://github.com/storytold/photocraft/blob/ce546dd4b5451a17b843ef1e129bdb40fbfe3160/crates/ui-egui/src/color_hud.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-ui-egui --locked shift_alt_right_press_opens_the_hud_not_the_picker -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-6510: Automation: resolve configured root aliases for absolute paths (#6426)

- Fixture/setup: [`canonical_windows_prefixes_match_without_ambient_resolution`](https://github.com/storytold/photocraft/blob/8bb0a5e67474d36ae08d565251edb1ae780dd044/crates/automation/src/workspace/tests.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-automation --locked canonical_windows_prefixes_match_without_ambient_resolution -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-6621: Crop: right-click on the pending crop opens the crop menu (#6561)

- Fixture/setup: [`crop_menu_ratio_rows_fit_the_frame_like_the_options_bar`](https://github.com/storytold/photocraft/blob/ead45134d320cb1002724d5ef44f5c56588bbe1d/crates/ui-egui/src/canvas_tool_menu.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-ui-egui --locked crop_menu_ratio_rows_fit_the_frame_like_the_options_bar -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-7648: Pen: Path Operations menu, as in Photoshop (#1397)

- Fixture/setup: [`merge_shape_components_bakes_a_paths_operations`](https://github.com/storytold/photocraft/blob/d24669d412f64ab079a668b0b8520ddbc607410c/crates/engine/src/cmd/path_edit_cmds.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-engine --locked merge_shape_components_bakes_a_paths_operations -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-8074: Transform: remember selected interpolation (#8067)

- Fixture/setup: [`interpolation_selection_is_reused_for_the_next_transform`](https://github.com/storytold/photocraft/blob/290f924d0fecc3bf6bea159c0d9e86255d326a96/crates/ui-egui/src/transform_tool.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-ui-egui --locked interpolation_selection_is_reused_for_the_next_transform -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-8896: Select and Mask: Photoshop's full-window workspace (#7069, #8148)

- Fixture/setup: [`overlay_colour_and_what_it_indicates`](https://github.com/storytold/photocraft/blob/19fa473039ef11deda7c3200c8fac1de41309086/crates/ui-egui/src/canvas/select_mask_view.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-ui-egui --locked overlay_colour_and_what_it_indicates -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-9493: fix(vector): a mask feather continues the mask past the canvas edge

- Fixture/setup: [`feather_at_the_canvas_edge_continues_the_mask_past_it`](https://github.com/storytold/photocraft/blob/61a413d8df9e90ee545eb08e57c5615aa2b29cea/crates/compose/src/masks.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-compose --locked feather_at_the_canvas_edge_continues_the_mask_past_it -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-3139: Draw right-to-left layer names in visual word order

- Fixture/setup: [`text_without_right_to_left_characters_is_untouched`](https://github.com/storytold/photocraft/blob/4da79e8d16ded8b51f06c1ac64e50ccba288274e/crates/ui-egui/src/bidi_label_tests.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-ui-egui --locked text_without_right_to_left_characters_is_untouched -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-4783: Fix/4758 type extreme finite geometry 20261013s1215

- Fixture/setup: [`tysh_fallback_cannot_reintroduce_unusable_finite_or_nonfinite_transform`](https://github.com/storytold/photocraft/blob/a9d62726bed607bc540db9d09c8dc1be4f9895aa/crates/io/src/blocks.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-io --locked tysh_fallback_cannot_reintroduce_unusable_finite_or_nonfinite_transform -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-8347: Type: Roman Hanging Punctuation (trailing marks hang, don't steer the alignment)

- Fixture/setup: [`set_style_paragraph_hanging_round_trips`](https://github.com/storytold/photocraft/blob/2b1364393cfcd09380cf18a221e7cdd325fc4deb/crates/engine/src/type_cmds.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-engine --locked set_style_paragraph_hanging_round_trips -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-9457: raw: faster tone stage (Shadows, Highlights, Clarity, Texture, Dehaze), same bytes

- Fixture/setup: [`gauss_gives_the_bits_of_the_tap_loop`](https://github.com/storytold/photocraft/blob/2863c3281c2a22dade0ce92d42bf8900074f76c6/crates/raw/src/tone.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-raw --locked gauss_gives_the_bits_of_the_tap_loop -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-9478: fix(text): vertical type sets roman letters upright, one per em, as Photoshop does (#9470)

- Fixture/setup: [`baseline_direction_round_trips`](https://github.com/storytold/photocraft/blob/cc208ee0440bbdf1858565dca49667272a490c56/crates/text/src/psd.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-text --locked baseline_direction_round_trips -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-9523: Fix Type on a Path wrapping and caret positioning across closed-path seams

- Fixture/setup: [`closed_rectangle_left_edge_continues_past_the_top_left_seam`](https://github.com/storytold/photocraft/blob/728782c595e768bdb6febc7f30e4d1456d13fd36/crates/text/src/closed_path_tests.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-text --locked closed_rectangle_left_edge_continues_past_the_top_left_seam -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-9594: fix(engine): copied type keeps its size across resolutions after an edit

- Fixture/setup: [`copied_type_keeps_its_pixel_size_across_resolutions_after_an_edit`](https://github.com/storytold/photocraft/blob/46512521fec2b4b462c29d9fefd16398cb81212c/crates/engine/src/layer_copy_cmds/tests.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-engine --locked copied_type_keeps_its_pixel_size_across_resolutions_after_an_edit -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-9596: fix(io): drop a kept Txt2 once no type layers are left

- Fixture/setup: [`a_kept_txt2_goes_with_the_last_type_layer`](https://github.com/storytold/photocraft/blob/e62013cfd652fa60470a0f7a185fbc8f3d2a2c92/crates/io/src/psd_export.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-io --locked a_kept_txt2_goes_with_the_last_type_layer -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-3345: Add grouped gradient presets and selectable style icons

- Fixture/setup: [`native_gradient_icons_persist_reset_and_preserve_legacy_defaults`](https://github.com/storytold/photocraft/blob/bfb691f84896fe68543980426632153dc4b4ea69/crates/engine/src/prefs/tests.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-engine --locked native_gradient_icons_persist_reset_and_preserve_legacy_defaults -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-3461: Replay recorded layer visibility on the intended named layer

- Fixture/setup: [`recorded_eye_visibility_replays_on_another_document_and_duplicate_names`](https://github.com/storytold/photocraft/blob/5d8cb2a39ad70e31260f1f3fc2010ca132daff65/crates/engine/src/actions_cmds.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-engine --locked recorded_eye_visibility_replays_on_another_document_and_duplicate_names -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-3520: fix(ui): restore square color chips in Studio Dark theme

- Fixture/setup: [`studio_dark_keeps_square_colour_chips`](https://github.com/storytold/photocraft/blob/d36ec9a21e9c1ed70de2e0d185efd7e084644157/crates/ui-egui/src/theme.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-ui-egui --locked studio_dark_keeps_square_colour_chips -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-4035: fix(cmyk): Flatten Image and merges keep the inks of a tagged CMYK document (#3778)

- Fixture/setup: [`flatten_keeps_the_inks_of_a_single_layer_exactly`](https://github.com/storytold/photocraft/blob/15ebf8fc940031abdf772e5fdd687c8b55938639/crates/engine/tests/cmyk_flatten_profile.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-engine --locked flatten_keeps_the_inks_of_a_single_layer_exactly -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-4042: feat(canvas): rotate view with Shift + middle-mouse drag

- Fixture/setup: [`middle_drag_rotates_about_the_pointer_without_panning`](https://github.com/storytold/photocraft/blob/f05098562220a663d052ee3918fd36c8c3e17a51/crates/ui-egui/src/rotate_view.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-ui-egui --locked middle_drag_rotates_about_the_pointer_without_panning -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-4547: fix(icons): PhotoCraft icon corner radius matches other Craft apps (#4478)

- Fixture/setup: [`windows_and_linux_icons_keep_the_round_corners`](https://github.com/storytold/photocraft/blob/a08f7a2295488c2692f68902835a693322017b64/apps/photocraft/src/app_icon.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft --locked windows_and_linux_icons_keep_the_round_corners -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-4883: feat(ui): show selection and active-layer bounds in the status bar

- Fixture/setup: [`status_bounds_follow_the_actual_selection_and_active_layer`](https://github.com/storytold/photocraft/blob/ea500486fb40511d3b4782dfacbb89fbff846d8a/crates/ui-egui/src/chrome_ui.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-ui-egui --locked status_bounds_follow_the_actual_selection_and_active_layer -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-4890: Add scrollbar visibility preferences (#3262)

- Fixture/setup: [`scrollbar_visibility_round_trips_and_old_preferences_keep_the_theme_default`](https://github.com/storytold/photocraft/blob/ab84cab7355ac999fb1be226950978bafec39605/crates/engine/src/prefs/tests.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-engine --locked scrollbar_visibility_round_trips_and_old_preferences_keep_the_theme_default -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-4938: Layer Style: one dialog size on every page, and it fits short screens

- Fixture/setup: [`layer_style_keeps_one_size_on_every_page`](https://github.com/storytold/photocraft/blob/63b7c921634835259139e4d3e7b9d3c396998b4d/crates/ui-egui/src/dialogs.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-ui-egui --locked layer_style_keeps_one_size_on_every_page -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-5270: Layer Style: Gradient Overlay takes any number of colour stops (#5177)

- Fixture/setup: [`gradient_overlay_takes_explicit_stops`](https://github.com/storytold/photocraft/blob/9db1c7ae1d48ce4f31622b5f9029c70c590842f8/crates/engine/src/layer_style.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-engine --locked gradient_overlay_takes_explicit_stops -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-5744: fix(ui): keep detached documents in one interactive window

- Fixture/setup: [`torn_off_document_is_not_drawn_again_in_the_main_window`](https://github.com/storytold/photocraft/blob/50b1f6734799cf7b5d45dd84868976afd31aa711/crates/ui-egui/src/canvas.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-ui-egui --locked torn_off_document_is_not_drawn_again_in_the_main_window -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-6387: Video Frames to Layers: every page of a multi-page TIFF (part of #6379)

- Fixture/setup: [`video_frames_to_layers_takes_every_tiff_page`](https://github.com/storytold/photocraft/blob/8bfbbf816025942b74130adf16fb07265817b691/crates/engine/src/video_cmds/tests.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-engine --locked video_frames_to_layers_takes_every_tiff_page -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-6449: Hand tool: Cmd held zooms in and Alt zooms out, as in Photoshop

- Fixture/setup: [`cmd_and_alt_with_the_hand_tool_zoom_in_and_out_while_held`](https://github.com/storytold/photocraft/blob/25c38a0b69b7bff31badcdbe2ef9f2ff6aba1c63/crates/ui-egui/src/hold_keys/tests.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-ui-egui --locked cmd_and_alt_with_the_hand_tool_zoom_in_and_out_while_held -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-7255: fix(engine): guard trim and duplicate merged against documents exceeding render limits (#7223)

- Fixture/setup: [`trim_and_duplicate_merged_on_too_large_document_fail_gracefully_without_panic`](https://github.com/storytold/photocraft/blob/1b810052bc3a876c1eaf1a8766870ffdba9f8a80/crates/engine/src/image_cmds.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-engine --locked trim_and_duplicate_merged_on_too_large_document_fail_gracefully_without_panic -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-8931: feat(algo): describe the Gaussian blur as data for other devices

- Fixture/setup: [`the_description_is_the_cpus_blur`](https://github.com/storytold/photocraft/blob/d42dd4c6302c77c1f9dc46b7b53ffa5eb0047a13/crates/algo/src/separable.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-algo --locked the_description_is_the_cpus_blur -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-9004: fix(ui): enlarge Move tool alignment icons for legibility

- Fixture/setup: Open a synthetic layered document with Move tool; use the supported themes and display scales.
- Action: Inspect Move alignment icons and compare screenshots to the reviewed icon geometry in this PR.
- Expected: Icons use the enlarged reviewed geometry, remain legible and do not clip or shift adjacent controls.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-9015: fix(xtask): reject leading zeroes in numeric prerelease identifiers

- Fixture/setup: [`rejects_numeric_prerelease_identifiers_with_leading_zeroes`](https://github.com/storytold/photocraft/blob/afb4364a8795c42e6353762891b4890d878f9a68/xtask/src/version.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p xtask --locked rejects_numeric_prerelease_identifiers_with_leading_zeroes -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-9172: Gradient Fill layer: default to Foreground to Transparent (#8708)

- Fixture/setup: [`new_gradient_fill_layer_defaults_to_foreground_to_transparent`](https://github.com/storytold/photocraft/blob/8ebb04999d25790f71f9bd88f7a268a6a2754aec/crates/engine/src/gradient_fill_cmds/tests.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-engine --locked new_gradient_fill_layer_defaults_to_foreground_to_transparent -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-9282: feat(ui): Preset menus for Levels, Exposure, Black & White and Channel Mixer

- Fixture/setup: [`adjustment_preset_menus_load_presets_and_read_custom_after_an_edit`](https://github.com/storytold/photocraft/blob/20803d1f15900ed6ecc7b5c79b4c68f6c3d6dcce/crates/ui-egui/src/adjust_dialog.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-ui-egui --locked adjustment_preset_menus_load_presets_and_read_custom_after_an_edit -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-9354: fix(ui): Full Screen Mode needs a document (#9349)

- Fixture/setup: [`screen_mode_button_is_inert_without_a_document`](https://github.com/storytold/photocraft/blob/41eb1de5cff17f5f5b9c22913e48b71565b56a2a/crates/ui-egui/src/panels.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-ui-egui --locked screen_mode_button_is_inert_without_a_document -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-9418: Color Lookup: honour the .3dl Mesh header's cube edge and output depth (#9078, #9073)

- Fixture/setup: [`three_dl_mesh_three_point_shaper_is_not_a_node`](https://github.com/storytold/photocraft/blob/da4b838e54f6073cda4798fa7b63b3b7214aaf93/crates/cms/src/lutfile.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-cms --locked three_dl_mesh_three_point_shaper_is_not_a_node -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-9481: fix(ui): new layer effects start from the Make Default values

- Fixture/setup: [`new_effects_start_from_the_made_default`](https://github.com/storytold/photocraft/blob/7937adff961027154d795b8905e821244ae4f929/crates/ui-egui/src/layer_style.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-ui-egui --locked new_effects_start_from_the_made_default -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-9504: fix(ui): Keyboard Shortcuts records ⌘Z instead of undoing the capture

- Fixture/setup: [`cmd_z_is_recorded_while_capturing_and_undoes_otherwise`](https://github.com/storytold/photocraft/blob/993c9ed167c6acd9bc8f87c6cff6e2c7abba17a8/crates/ui-egui/src/shortcut_capture_tests.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-ui-egui --locked cmd_z_is_recorded_while_capturing_and_undoes_otherwise -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-9517: fix(ui): Layer Style scrolls on a short window so OK stays on screen

- Fixture/setup: [`layer_style_buttons_stay_on_a_short_window`](https://github.com/storytold/photocraft/blob/81b434c6149732eadc94e0465fea81bd075facd4/crates/ui-egui/src/dialogs.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-ui-egui --locked layer_style_buttons_stay_on_a_short_window -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-9597: fix(ui): the layer list doesn't move with no layer selected

- Fixture/setup: [`layer_rows_do_not_move_when_no_layer_is_selected`](https://github.com/storytold/photocraft/blob/9429df26b371c04c2afa65228a60ded739b5fbc2/crates/ui-egui/src/layer_pct_slider_tests.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-ui-egui --locked layer_rows_do_not_move_when_no_layer_is_selected -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-9598: fix(ui): Liquify's Twirl Counterclockwise gets a mirrored twirl icon

- Fixture/setup: [`twirl_counterclockwise_has_its_own_mirrored_icon`](https://github.com/storytold/photocraft/blob/47aea89e9220b996f0d5a0d2bb8b5938100b4ad1/crates/ui-egui/src/liquify_ui.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-ui-egui --locked twirl_counterclockwise_has_its_own_mirrored_icon -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 

## PR-9606: fix(ui): the Color panel remembers its options across restarts

- Fixture/setup: [`color_panel_options_survive_a_restart`](https://github.com/storytold/photocraft/blob/6cc516a3d20120b678040050d95ccadc8d1dfd3f/crates/ui-egui/src/color_panel_tests.rs); use its checked-in synthetic inputs and setup unchanged.
- Action: `cargo test -p photocraft-ui-egui --locked color_panel_options_survive_a_restart -- --nocapture`
- Expected: matching regression executes and every checked-in assertion passes; verify the named behavior in the frozen integrated tree.
- Status: NOT RUN
- Actual: 
- Evidence: 
- Tester: 
