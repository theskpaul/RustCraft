These decisions should be made explicitly rather than guessed by the coding agent.

1. **Reference version:** 26.3 // Which exact Minecraft Java Edition release defines gameplay parity?
2. **World compatibility:** No // Is matching world generation for identical seeds required, or is matching general behavior sufficient initially?
3. **Save compatibility:** No MCJE import. Worlds must be exportable/importable between RustCraft installs (RustCraft's own format). // Is importing or exporting existing Minecraft Java world saves a requirement?
4. **Network compatibility:** No // Is compatibility with the official Java Edition protocol desired, or will RustCraft use its own protocol?
5. **Modding:** Yes // Should RustCraft support scripting, plugins, or data-driven modifications?
6. **Audio:** Which original audio production and asset pipeline will be used?
7. **Distribution:** Is the initial target Linux only, or should Windows support be planned from the start?
8. **Visual identity:** What original name, UI design, textures, models, and sound identity should the finished game use?

Do not block Milestone 1 on every one of these decisions. The first renderer can be developed while the reference-version decision is finalized.
