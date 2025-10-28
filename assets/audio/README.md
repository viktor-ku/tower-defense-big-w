Audio assets guide

This game uses bevy_kira_audio for playback and expects sound effects (SFX) under assets/audio/sfx using snake_case file stems.

Supported formats
- wav, flac, mp3, ogg (in this priority order)

How resolution works
- For each SFX stem, the engine tries assets/audio/sfx/<stem>.{wav,flac,mp3,ogg} in that order and uses the first file that exists.
- If no file exists for a stem, the game will still run; that sound will simply be silent.

Current SFX stems (snake_case)
- tower_bow_release
- tower_crossbow_release
- wave_start
- wave_start_boss
- player_footstep_01
- tower_place
- tower_place_invalid
- tower_upgrade
- tower_sell

Place your files like this
- assets/audio/sfx/tower_bow_release.wav
- assets/audio/sfx/wave_start_boss.ogg
…any of the four supported extensions are fine.

Event mapping (what plays when)
- Tower shots (spatialized): tower_bow_release or tower_crossbow_release
  - Trigger: when any tower fires
  - Position: tower world position (stereo pan + distance attenuation)
- Player footsteps (spatialized): player_footstep_01
  - Trigger: while moving, periodic cadence
  - Position: player world position
- Wave start (screen-space): wave_start
  - Trigger: start of a normal wave
- Boss wave start (screen-space): wave_start_boss
  - Trigger: start of boss waves (every 10th)
- Building actions (spatialized):
  - tower_place: a tower was placed
  - tower_place_invalid: invalid placement attempt
  - tower_upgrade: tower upgraded (reserved; when upgrades exist)
  - tower_sell: tower sold

Spatialization
- Listener: the main 3D camera (tagged with AudioListener)
- Model: simple 2D top-down stereo pan and distance attenuation

Channels and volumes
- SFX play on SfxChannel with an overall volume of master * sfx (see AudioVolumes resource)
- Music/UI/Ambience channels are reserved for future use

Extending with new sounds
1) Add a new loader module under src/audio/sfx using the existing files as reference. Each module exports:
   - STEM: &str with the snake_case name
   - load(asset_server) -> Handle<AudioSource>
2) Re-export it in src/audio/sfx/mod.rs
3) Wire an event and play it from src/audio/mod.rs or the relevant gameplay system
4) Drop your audio file in assets/audio/sfx using the same STEM and any supported extension

Notes
- Assets are optional: missing files won’t crash the game
- Keep names in snake_case to stay consistent with code and files

