# GTK3 and GLib 0.22 compatibility sources

The Linux desktop lock keeps GTK3 and the WebKitGTK 4.1 system library.
The overlays let the pinned Tauri 2.12 family select the published GTK3
0.19 and GLib 0.22 Rust bindings. GLib 0.22 includes the fix for
[RUSTSEC-2024-0429](https://rustsec.org/advisories/RUSTSEC-2024-0429.html).

The package-local licenses and original release metadata accompany each
overlay. The GTK3 adaptation follows these upstream changes:

| Component | Upstream change | Source commit |
| --- | --- | --- |
| Tauri and runtime | [Tauri #16170](https://github.com/tauri-apps/tauri/pull/16170) | `899ca2105ffd06293013b6d2bff4416987d99010` |
| Tao | [Tao #1332](https://github.com/tauri-apps/tao/pull/1332) | `08660027c723e09a45b10badf60a73a661c94a95` |
| Wry | [Wry #1843](https://github.com/tauri-apps/wry/pull/1843) | `327242ae73301c5e34dcc158e31ae453e8afe837` |
| WebKitGTK wrapper and sys | [WebKitGTK #167](https://github.com/tauri-apps/webkit2gtk-rs/pull/167) | `f5c3825c3f21c44ce346a5aa42e8c33a762f9f72` |
| Muda | [Muda #403](https://github.com/tauri-apps/muda/pull/403) | `920ee6fd027ee778bd9867b5ab7005f7a130ff03` |
| Tray Icon | [Tray Icon #369](https://github.com/tauri-apps/tray-icon/pull/369) | `af76479d5ef9f52fbc6a7bf127b0edd8164d72ff` |
| AppIndicator and sys | [AppIndicator #53](https://github.com/tauri-apps/libappindicator-rs/pull/53) | `f2617d92288e2f0a2f998d3cd46c003e3a5c6d93` |
