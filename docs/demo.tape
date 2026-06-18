Output assets/demo.gif

Require python3
Require bash

Set Shell "bash"
Set FontSize 19
Set FontFamily "Menlo"
Set Width 1500
Set Height 780
Set Padding 20
Set Framerate 18
Set PlaybackSpeed 1.25
Set Theme "Dracula"
Set WindowBar Colorful
Set LetterSpacing 0
Set LineHeight 1.1
Set TypingSpeed 14ms

Type 'clear' Enter
Sleep 300ms

Type 'printf "MCP Doctor — trace, validate, replay MCP servers\n\n"' Enter
Sleep 700ms

Type 'md=./target/debug/mcp-doctor' Enter
Sleep 300ms
Type 'srv="python3 fixtures/fake_mcp_server.py"' Enter
Sleep 500ms

Type '$md smoke stdio -- $srv' Enter
Sleep 1s

Type '$md call stdio --tool echo --args "{\"text\":\"hello MCP\"}" -- $srv' Enter
Sleep 1.6s

Type 'TRACE=$(ls -td .mcpdoctor/sessions/*/trace.jsonl | head -1)' Enter
Sleep 350ms

Type '$md validate "$TRACE"' Enter
Sleep 800ms

Type 'printf "\\033c"' Enter
Sleep 500ms
Type '$md tui "$TRACE" | head -26' Enter
Sleep 1.8s

Type '$md export-repro "$TRACE" --output /tmp/mcp-doctor-replay.sh' Enter
Sleep 900ms

Type '$md replay "$TRACE" -- $srv' Enter
Sleep 2.2s
