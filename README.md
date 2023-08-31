<h1 align="center">
  <a href="https://lapce.dev" target="_blank">
  <img src="extra/images/logo.png" width=200 height=200/><br>
  Lapce
  </a>
</h1>

Lapce (IPA: /læps/) is written in pure Rust with a UI in [Floem](https://github.com/lapce/floem) (which is also written in Rust). It is designed with [Rope Science](https://xi-editor.io/docs/rope_science_00.html) from the [Xi-Editor](https://github.com/xi-editor/xi-editor) which makes for lightning-fast computation, and leverages [Wgpu](https://github.com/gfx-rs/wgpu) for rendering. More information about the features of Lapce can be found on the [main website](https://lapce.dev) and user documentation can be found on [GitBook](https://docs.lapce.dev/).

This is my attempt to implement real-time collaborative editing in Lapce.


# RUN THE SOURCE CODE
- git clone https://github.com/lapce/lapce.git ~/lapce
- cd ~/lapce
- cargo build --release
will save the executable in --> ./target/release/lapce <--
- cargo run 
will save the executable in --> ./target/debug/lapce <--
