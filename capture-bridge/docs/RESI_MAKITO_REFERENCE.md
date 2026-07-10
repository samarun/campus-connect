# Resi and Makito Feature Reference

This note records the product patterns that should influence Campus Connect Capture Bridge without copying another vendor's branding, wording, or proprietary UI.

## Sources Reviewed

- Resi Multisite product and support references: https://resi.io/multisite/ and https://support.pushpay.com/s/article/get-started-with-resi-multisite
- Haivision Makito X4 Encoder web interface: https://doc.haivision.com/MakitoX4Enc/1.9/exploring-the-web-interface
- Makito X4 stream, video encoder, audio encoder, network service, and security/account documentation under the same Haivision guide.

## Product Patterns To Keep

- Appliance-style dashboard: show state, active profile, selected format, stream target, input, and preflight status up front.
- Grouped sidebar: separate operator workflow, campus/decoder workflow, capture inputs, streaming controls, and administration.
- Stream list view: show status, stream name, protocol, destination, selected content, route/path, statistics, settings, start/stop affordances.
- Encoder list view: separate video and audio encoders, with status, input, format, codec, bitrate, stats, and settings.
- Decoder-friendly operation: expose campus outputs, fullscreen display routing, countdown/cue, pause, buffer, and volunteer-safe controls.
- Event workflow: Sunday service and rehearsal events should be selectable without requiring the operator to rebuild settings every time.
- Network/services page: show only required services and their interface/port; warn against opening unnecessary rack services.
- Security page: separate admin/operator/campus roles, local-first API defaults, API token requirements, audit logging, and session controls.
- Preview thumbnails/local preview: use local preview as a fast confirmation of the selected camera, OBS virtual camera, or capture-card source.
- Presets/profiles: keep configuration reusable, but avoid duplicate preset grids when a dropdown already provides selection.

## Campus Connect Mapping

- Resi-style multisite reliability maps to stream relay SRT relay, local recording fallback, event rehearsals, and future LAN/cloud fallback logic.
- Makito-style stream and encoder rows map to FFmpeg/GStreamer plans for OBS virtual camera, DeckLink, v4l2, AVFoundation, and future AJA SDK adapters.
- Decoder controls map to Tauri-managed receiver windows today; later they can map to a rack daemon or dedicated decoder hardware.
- Operator-safe UI remains the default. Advanced settings stay present, but the first path should be profile selection, preflight, start, fullscreen outputs, and monitor.

## Not Copied

- Do not copy Resi, Pushpay, Haivision, or Makito branding.
- Do not reproduce proprietary layouts pixel-for-pixel.
- Use Campus Connect language, Campus Connect color/style, and Campus Connect-specific campus workflows.
