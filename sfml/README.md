crusty-chip-sfml
================

A chip8 interpreter written in Rust (SFML frontend)

## Controls ##

### Keypad ###
```
1234
QWER
ASDF
ZXCV
```

### Meta ###

Key combination | Effect
----------------|-----------------
P               | Pause
.               | Cycle advance
Ctrl+R          | Restart
F1-F10          | Load states 1-10
Shift + F1-F10  | Save states 1-10

When paused, crusty-chip-sfml prints debugging information to stdout.
This combined with cycle advance can be used to debug the interpreter or CHIP-8 programs.
